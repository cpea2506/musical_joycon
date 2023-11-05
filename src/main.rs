use joycon_rs::prelude::*;
use rimd::{Event, MetaCommand, Status, SMF};
use std::convert::TryInto;
use std::ops::Deref;
use std::path::Path;
use std::time::Duration;

const MICRO_PER_QUARTER: u64 = 60_000_000;

fn play_song(mut driver: SimpleJoyConDriver) -> JoyConResult<()> {
    let smf = SMF::from_file(&Path::new("src/midi/overworld.mid")).unwrap();

    for track in &smf.tracks {
        let mut tempo: u64 = 1;

        for event in track.events.iter() {
            match &event.event {
                Event::Meta(meta) => {
                    if let MetaCommand::TempoSetting = meta.command {
                        tempo = MICRO_PER_QUARTER / meta.data_as_u64(3);
                    }
                }
                Event::Midi(message) => {
                    let status = message.status();

                    if matches!(status, Status::NoteOff | Status::NoteOn) {
                        let time = event.vtime * 60_000 / (smf.division as u64 * tempo);
                        std::thread::sleep(Duration::from_millis(time));
                    }

                    match status {
                        Status::NoteOn => {
                            dbg!(message.data[1]);
                            let mut note: i32 = 5 * (message.data[1] as i32 - 60) + 120;

                            if note > 250 || note < 120 {
                                note = (note % 250) + 120;
                            }

                            note -= note % 2;
                            println!("{}", note);

                            let rumble = Rumble::new(note as f32, 1.);
                            driver.rumble((Some(rumble), Some(rumble)))?;
                        }
                        Status::NoteOff => {
                            let rumble =
                                Rumble::new(message.data[1] as f32, message.data(2) as f32);
                            driver.rumble((Some(rumble), Some(rumble)))?;
                            // driver.rumble((Some(Rumble::stop()), Some(Rumble::stop())))?;
                        }
                        _ => (),
                    }
                }
            }
        }
    }

    Ok(())
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
            let driver = SimpleJoyConDriver::new(&d)?;
            play_song(driver)?;

            Ok(())
        })?;

    Ok(())
}
