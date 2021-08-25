use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::time::Duration;

use anyhow::Result;
use async_std::{
    net::{TcpListener, TcpStream},
    prelude::*,
};
use futures::{AsyncBufReadExt, FutureExt, sink::SinkExt, StreamExt};
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures::channel::mpsc;

use crate::config::Config;
use crate::memory_store::MemoryStore;
use crate::message::{Event, Message};
use crate::node::RaftNode;
use futures::io::BufReader;

const TICK: Duration = Duration::from_millis(10000);

pub struct RaftServer {
    node: RaftNode<MemoryStore>,
    node_rx: UnboundedReceiver<Message>,
    conf: Config,
}

impl RaftServer {
    pub fn new(conf: Config) -> RaftServer {
        //通道的两头, 接收方给RaftNode(当前节点),
        // 当此node需要发送消息给peer,
        // 从rx发送消息,在event_loop中的rx收到消息再发送出去
        let (node_tx, node_rx) = mpsc::unbounded();
        let store = MemoryStore {};
        RaftServer {
            node: RaftNode::new(conf.id.clone(), store, node_tx),
            node_rx,
            conf,
        }
    }

    //此函数处理三个功能:
    // 1, 作为server角色, 监听接收其他节点的消息
    // 2, 作为client角色, 发送消息给其他节点
    // 3, 作为整个server, 接收外部client的请求
    pub async fn serve(self, client_rx: UnboundedReceiver<Message>) -> Result<()> {

        //1, 接收其他Node的TCP请求, 以server的角色
        let (tcp_in_tx, tcp_in_rx) = mpsc::unbounded();
        let addr = self.conf.listen_raft.clone();
        let (task, receive) = RaftServer::tcp_receive(addr, tcp_in_tx).remote_handle();
        async_std::task::spawn(task);

        //2,
        let (tcp_out_tx, tcp_out_rx) = mpsc::unbounded();
        let (task, send) = RaftServer::tcp_sender(self.conf.peers.clone(), tcp_out_rx).remote_handle();
        async_std::task::spawn(task);

        //集中处理所有请求, 节点之间以及client的请求
        //用channel链接此函数与send,receive两函数
        let (task, event_loop) = self
            .event_loop(tcp_in_rx, tcp_out_tx, client_rx)
            .remote_handle();
        async_std::task::spawn(task);

        futures::join!(event_loop, receive, send);

        Ok(())
    }

    async fn event_loop(
        self,
        tcp_in_rx: UnboundedReceiver<Message>, //其他node请求的接收通道
        tcp_out_tx: UnboundedSender<Message>, //本节点向其他节点的发送通道
        //来自客户端的请求接收通道(发送端在外部逻辑处理处), 如查询请求
        client_rx: UnboundedReceiver<Message>,
    ) -> Result<()> {
        let mut client_rx = UnboundedReceiver::from(client_rx);
        let mut node_rx = UnboundedReceiver::from(self.node_rx);
        let mut tcp_in_rx = UnboundedReceiver::from(tcp_in_rx);
        let mut tcp_out_tx = UnboundedSender::from(tcp_out_tx);

        let mut tick = async_std::stream::interval(TICK);
        let mut node = self.node;
        loop {
            futures::select! {
                _ = tick.next().fuse() => node = node.tick()?,
                //处理其他node发送过来的消息
                msg = tcp_in_rx.next().fuse() => match msg{
                    Some(msg) => *&(node).step(msg)?,
                    None => (),
                },
                //接收从RaftNode过来的消息, 转发到send函数处理
                msg = node_rx.next().fuse() => match msg {
                    Some(msg) => {tcp_out_tx.unbounded_send(msg)?},
                    None =>(),
                },
                //接收client发来的消息
                msg = client_rx.next().fuse() => match msg {
                    Some(msg) => {println!("from client:{:?}", msg); ()},
                    None=> (),
                }
            }
        }
    }

    /// 监听其他节点消息
    async fn tcp_receive(addr: String, out_rx: UnboundedSender<Message>) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            let mut stream = stream.unwrap();
            let out_rx = out_rx.clone();
            async_std::task::spawn(connection_loop(out_rx, stream));
        }
        Ok(())
    }


    /// 此node向其他节点的消息处理逻辑
    async fn tcp_sender(
        peers: HashMap<String, String>,
        out_tx: UnboundedReceiver<Message>,
    ) -> Result<()> {
        //此node向外部node发送的消息会来自此通道
        let mut out_tx = UnboundedReceiver::from(out_tx);
        let mut peer_txs = HashMap::new();

        for (id, addr) in peers.into_iter() {
            let (tx, rx) = mpsc::unbounded();
            peer_txs.insert(id, tx);
            async_std::task::spawn(
                send_message_to_peer(addr, rx)
            );
        }

        while let Some(msg) = out_tx.next().await {
            let to: Vec<String> = peer_txs.keys().cloned().collect();
            for id in to {
                let send = peer_txs.get_mut(&id).unwrap();
                send.unbounded_send(msg);
            }
        }
        Ok(())
    }
}

async fn send_message_to_peer(addr: String, rx: UnboundedReceiver<Message>) -> Result<()> {
    let mut rx = UnboundedReceiver::from(rx);
    loop {
        match async_std::net::TcpStream::connect(&addr).await {
            Ok(mut socket) => {
                println!("success connection: {}", &addr);
                while let Some(msg) = rx.next().await {
                    //TODO ser msg
                    socket.write_all(&vec![1, 2, 3]).await.unwrap();
                    socket.flush().await;
                }
            }
            Err(e) => println!("{:?}", e),
        }
        std::thread::sleep(Duration::from_secs(5));
    }
}

async fn connection_loop(out_rx: UnboundedSender<Message>, mut stream: TcpStream) -> Result<()> {

    let mut buffer = [0; 1024];
    loop {
        let nbytes = stream.read(&mut buffer).await?;
        if nbytes == 0 {
            return Ok(());
        }
        println!("{:?}", &buffer[..nbytes]);

        let mut out_rx = UnboundedSender::from(out_rx.clone());
        let m = Message{term: 11, event: Event::None};
        out_rx.unbounded_send(m);
    }
    Ok(())
}