use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use std::{collections::HashMap, error::Error, fs};

use crate::{DEFAULT_BPM, DEFAULT_TBP, MICROSECONDS_PER_MINUTE, ZERO_U7};

#[derive(Debug)]
pub struct NoteEvent {
    /// The absolute start timestamp of the event in MIDI ticks.
    pub start_tick: u64,
    /// The absolute end timestamp of the event in MIDI ticks.
    pub end_tick: u64,
    /// MIDI note number (0–127). Middle C is 60.
    pub key: u8,
    /// MIDI velocity (1–127). Zero is reserved for `NoteOff`.
    pub velocity: u8,
    /// MIDI channel (0-15).
    pub channel: Option<u8>,
}

#[derive(Debug)]
pub struct Midi {
    /// Ticks per beat.
    pub tpb: u16,
    /// Beat per minute.
    pub bpm: f64,
    /// List of `NoteEvent`.
    pub note_events: Vec<NoteEvent>,
}

impl Midi {
    pub fn new(path: &str, channel: u8) -> Result<Self, Box<dyn Error>> {
        let data = fs::read(path)?;
        let smf = Smf::parse(&data)?;

        let mut bpm = DEFAULT_BPM;
        let mut note_events = Vec::new();
        let mut active_notes = HashMap::new();

        for track in &smf.tracks {
            let mut ticks = 0u64;

            for event in track {
                ticks += event.delta.as_int() as u64;

                match event.kind {
                    TrackEventKind::Midi {
                        message,
                        channel: midi_channel,
                    } if midi_channel == channel => match message {
                        MidiMessage::NoteOn { key, vel } if vel > ZERO_U7 => {
                            active_notes.insert((channel, key), (ticks, vel.as_int()));
                        }
                        MidiMessage::NoteOff { key, .. }
                        | MidiMessage::NoteOn { key, vel: ZERO_U7 } => {
                            if let Some((start_tick, velocity)) =
                                active_notes.remove(&(channel, key))
                            {
                                note_events.push(NoteEvent {
                                    start_tick,
                                    end_tick: ticks,
                                    key: key.as_int(),
                                    velocity,
                                    channel: Some(channel),
                                });
                            }
                        }
                        _ => {}
                    },
                    TrackEventKind::Meta(MetaMessage::Tempo(ubp)) => {
                        bpm = MICROSECONDS_PER_MINUTE / ubp.as_int() as f64;
                    }
                    _ => {}
                }
            }
        }

        let tpb = match smf.header.timing {
            Timing::Metrical(t) => t.as_int(),
            _ => DEFAULT_TBP,
        };

        Ok(Self {
            tpb,
            bpm,
            note_events,
        })
    }
}
