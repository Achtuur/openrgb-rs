use std::error::Error;

use openrgb::OpenRgbClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // connect to local server at 127.0.0.1:6742
    let mut client = OpenRgbClient::connect().await?;

    client.set_name("My Rust Client").await?;
    println!(
        "connected using protocol version {}",
        client.get_protocol_version()
    );

    Ok(())
}
