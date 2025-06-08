mod midi;
mod song;

use std::ops::Deref;

use clap::Parser;
use joycon_rs::{joycon::joycon_features::JoyConFeature, prelude::*};
use song::Song;

use crate::midi::Midi;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Path to the midi file
    path: String,
    /// Which channel to be played.
    #[arg(long, default_value_t = 0)]
    channel: u8,
}

#[tokio::main]
async fn main() -> JoyConResult<()> {
    let args = Args::parse();

    let manager = JoyConManager::get_instance();
    let devices = {
        let lock = manager.lock();

        match lock {
            Ok(manager) => manager.new_devices(),
            Err(_) => unreachable!(),
        }
    };

    devices
        .into_iter()
        .inspect(|d| {
            let lock = d.lock();
            let device = match lock {
                Ok(device) => device,
                Err(e) => e.into_inner(),
            };
            let hid_device: Result<&HidDevice, JoyConError> = device.deref().try_into();

            if let Ok(hid_device) = hid_device {
                println!("{:?}", hid_device.get_product_string())
            }
        })
        .try_for_each::<_, Result<(), JoyConError>>(|d| {
            let mut driver = SimpleJoyConDriver::new(&d)?;
            driver.enable_feature(JoyConFeature::Vibration)?;

            if let Ok(midi) = Midi::new(&args.path, args.channel) {
                let song = Song::new(driver, midi);

                tokio::spawn(async move {
                    song.play().await?;

                    Ok::<(), JoyConError>(())
                });
            }

            Ok(())
        })?;

    Ok(())
}
