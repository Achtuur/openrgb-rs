use std::error::Error;

use openrgb::OpenRgbClientWrapper;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // connect to local server
    let mut client = OpenRgbClientWrapper::connect_to(("localhost", 6742)).await?;
    // set client name
    client.set_name("my client").await?;

    // print protocol version
    println!(
        "connected using protocol version {}",
        client.get_protocol_version()
    );

    Ok(())
}