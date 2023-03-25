use futures::TryFutureExt;
use serde_json::{from_slice, to_vec};
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
    let msg = to_vec(&String::from("Hi!")).unwrap();
    s.send_frame(msg.into())
        .map_ok(|msg| {
            if let zmtp::packets::null::Frame::Message(msg) = msg {
                println!("REP: {}", from_slice::<String>(&msg).unwrap())
            } else {
                ()
            }
        })
        .err_into()
        .await
}
