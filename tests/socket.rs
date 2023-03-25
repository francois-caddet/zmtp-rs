use tokio::test;

use ipc_chan::{Config, Sink};
use zmtp::{sockets, Result};

static HOST: &str = "localhost";

#[test]
pub async fn null_connect() -> Result<()> {
    let host = HOST.to_string();
    let port = 51515u16;
    let cfg = Config {
        host,
        port: port.into(),
    };
    let _sink = Sink::from_config(cfg).unwrap();
    let _s = sockets::Zmtp::connect(HOST, port).await?;
    Ok(())
}
