use serde_json::to_vec;
use tokio::main;
use twelf::{config, Layer};
use zmtp::sockets;

#[config]
struct Conf {
    /// The IP address of the stdout-sink node to contact
    host: String,
    /// The port of the stdout-sink node to contact
    port: u16,
}

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Will generate global arguments for each of your fields inside your configuration struct
    let app = clap::Command::new("zmtp").args(&Conf::clap_args());

    // Init configuration with layers, each layers override only existing fields
    let config = Conf::with_layers(&[
        Layer::Toml("ipc-chan.toml".into()),
        Layer::Env(Some("IPC_CHAN_".to_string())),
        Layer::Clap(app.get_matches()),
    ])?;
    println!("Connecting to tcp://{}:{}...", config.host, config.port);
    let mut s = sockets::Zmtp::connect(config.host.as_str(), config.port).await?;
    println!("{:?}", s.version());
    println!("{:?}", s.next_frame().await);
    s.send_frame(
        zmtp::packets::null::Command::Ready {
            socket_type: Vec::from(&b"REQ"[..]),
            identity: Some(Vec::from(&b"fcaddet"[..])),
        }
        .into(),
    )
    .await?;
    s.send_frame(zmtp::packets::null::Frame::Empty).await?;
    let msg = to_vec(&String::from("Hi!")).unwrap();
    s.send_frame(msg.into()).await?;
    println!("{:?}", s.next_frame().await);
    Ok(())
}
