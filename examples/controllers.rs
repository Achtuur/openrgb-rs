use std::error::Error;

use openrgb::{Color, OpenRgbClientWrapper};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // connect to local server
    let client = OpenRgbClientWrapper::connect().await?;

    let controllers = client.get_all_controllers().await?;
    for c in controllers {
        println!("controller {}: {:#?}", c.id(), c.name(),);
        c.set_controllable_mode().await?;
        c.update_all_leds(Color::new(255, 0, 255))
        .await?;
    }

    Ok(())
}
