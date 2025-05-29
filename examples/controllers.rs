use std::error::Error;

use openrgb::{Color, OpenRgbClientWrapper};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // connect to local server
    let mut client = OpenRgbClientWrapper::connect().await?;

    let controllers = client.get_all_controllers().await?;
    for c in controllers {
        println!("controller {}: {:#?}", c.id(), c.name(),);
        c.set_controllable_mode().await?;
        // c.update_zone(0, Color {r: 255, g: 0, b: 0}).await?;
        c.update_all_leds(Color {
            r: 255,
            g: 255,
            b: 255,
        })
        .await?;
        // c.update_led(0, Color {r: 255, g: 0, b: 0}).await?;
    }

    Ok(())
}
