use std::{fs, time::Duration};

use crate::midi::Midi;
use joycon_rs::{prelude::*, result::JoyConResult};
use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};

const MINUTE_PER_HOUR: u64 = 60;
const MILI_PER_HOUR: u64 = MINUTE_PER_HOUR as u64 * 1000;
const MICRO_PER_QUARTER: u64 = 60_000_000;

pub struct Song<'a>(Smf<'a>);

impl<'a> Song<'a> {
    pub fn new(file: String) -> Self {
        let data = fs::read(file).unwrap();
        let smf = Smf::parse(&data).unwrap();

        Self(smf.make_static())
    }

    pub fn play(self, mut driver: SimpleJoyConDriver) -> JoyConResult<()> {
        let Smf { tracks, header } = self.0;
        let mut division: u64 = 0;

        if let Timing::Metrical(time) = header.timing {
            division = time.as_int().into();
        }

        for track in &tracks {
            let mut tempo: u64 = 1;

            for TrackEvent { delta, kind } in track {
                match kind {
                    TrackEventKind::Meta(meta) => {
                        if let MetaMessage::Tempo(t) = meta {
                            tempo = MICRO_PER_QUARTER / t.as_int() as u64;
                        }
                    }
                    TrackEventKind::Midi { message, .. } => {
                        if message.is_note() {
                            let delta: u64 = delta.as_int().into();
                            let time = delta * MILI_PER_HOUR / (division * tempo);
                            std::thread::sleep(Duration::from_millis(time));
                        }

                        match message {
                            MidiMessage::NoteOn { key, .. } => {
                                let key: f32 = key.as_int().into();
                                let rumble = Rumble::new(key, 0.3);
                                driver.rumble((Some(rumble), Some(rumble)))?;
                            }
                            MidiMessage::NoteOff { .. } => {
                                driver.rumble((Some(Rumble::stop()), Some(Rumble::stop())))?;
                            }
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
        }

        Ok(())
    }
}
