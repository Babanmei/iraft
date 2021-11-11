use iraft::conf::Config;
use futures::executor::block_on;
use iraft::server::RaftServer;
use std::time::Duration;
use iraft::message::{Message, Event, Address};
use iraft::driver::MVCC;
use simplelog;
use log::{LevelFilter, Log};
use simplelog::SimpleLogger;

#[macro_use]
extern crate log;
#[async_std::main]
async fn main() -> std::io::Result<()> {
    let args = std::env::args().nth(1);
    let _ = SimpleLogger::init(LevelFilter::Debug, simplelog::Config::default());

    let cfg = match args {
        Some(arg) => Config::new(arg.as_str()).unwrap(),
        None => Config::default(),
    };
    debug!("config: {:?}", cfg);

    let x  = MVCC{};
    let trs = RaftServer::new(cfg, Box::new(x)).await;


    let (tx, rx) = futures::channel::mpsc::unbounded();
    std::thread::spawn(move || {
        for i in 2..10 {
            std::thread::sleep(Duration::from_secs(5));
            let m = Message { term: 11, from:Address::Local, to: Address::Peers, event: Event::None };
            //tx.unbounded_send(m);
        }
    });
    trs.serve(rx).await;
    Ok(())
}
