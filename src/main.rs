mod midi;
mod song;

use std::ops::Deref;

use clap::Parser;
use joycon_rs::prelude::*;
use song::Song;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Path to the midi file
    path: String,
}

fn main() -> JoyConResult<()> {
    let manager = JoyConManager::get_instance();
    let (managed_devices, new_devices) = {
        let lock = manager.lock();
        match lock {
            Ok(manager) => (manager.managed_devices(), manager.new_devices()),
            Err(_) => unreachable!(),
        }
    };

    managed_devices
        .into_iter()
        .chain(new_devices)
        .inspect(|d| {
            let lock = d.lock();
            let device = match lock {
                Ok(device) => device,
                Err(e) => e.into_inner(),
            };
            let hid_device: JoyConResult<&HidDevice> = device.deref().try_into();
            if let Ok(hid_device) = hid_device {
                println!("{:?}", hid_device.get_product_string())
            }
        })
        .try_for_each::<_, JoyConResult<()>>(|d| {
            let args = Args::parse();
            let driver = SimpleJoyConDriver::new(&d)?;
            let song = Song::new(args.path);

            song.play(driver)?;

            Ok(())
        })?;

    Ok(())
}
