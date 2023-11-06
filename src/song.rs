use std::{path::Path, time::Duration};

use crate::midi::Midi;
use joycon_rs::{prelude::*, result::JoyConResult};
use rimd::{Event, MetaCommand, Status, Track, TrackEvent, SMF};

const MINUTE_PER_HOUR: u64 = 60;
const MILI_PER_HOUR: u64 = MINUTE_PER_HOUR as u64 * 1000;
const MICRO_PER_QUARTER: u64 = 60_000_000;

pub struct Song(SMF);

impl Song {
    pub fn new(file: String) -> Self {
        let smf = SMF::from_file(Path::new(&file)).unwrap();

        Self(smf)
    }

    pub fn play(self, mut driver: SimpleJoyConDriver) -> JoyConResult<()> {
        let SMF {
            division, tracks, ..
        } = self.0;

        for Track { events, .. } in tracks {
            let mut tempo: u64 = 1;

            for TrackEvent { vtime, event, .. } in events.iter() {
                match event {
                    Event::Meta(meta) => {
                        if let MetaCommand::TempoSetting = meta.command {
                            tempo = MICRO_PER_QUARTER / meta.data_as_u64(3);
                        }
                    }
                    Event::Midi(message) => {
                        let status = message.status();

                        if status.is_note() {
                            let time = vtime * MILI_PER_HOUR / (division as u64 * tempo);
                            std::thread::sleep(Duration::from_millis(time));
                        }

                        match status {
                            Status::NoteOn => {
                                let rumble = Rumble::new(message.data[1] as f32, 0.3);
                                driver.rumble((Some(rumble), Some(rumble)))?;
                            }
                            Status::NoteOff => {
                                driver.rumble((Some(Rumble::stop()), Some(Rumble::stop())))?;
                            }
                            _ => (),
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
