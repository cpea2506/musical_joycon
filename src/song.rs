use crate::{
    MAX_VELOCITY, MICROSECONDS_PER_MINUTE, MIDI_NOTE_TO_FREQUENCIES, NOTE_NAMES, midi::Midi,
};
use joycon_rs::prelude::*;
use std::time::Duration;
use tokio::time::{Instant, sleep_until};

pub struct Song {
    driver: SimpleJoyConDriver,
    midi: Midi,
}

impl Song {
    pub fn new(driver: SimpleJoyConDriver, midi: Midi) -> Self {
        Self { driver, midi }
    }

    fn note_name(note: u8) -> String {
        let pitch_class = note % 12;
        let octave = note / 12 - 1; // MIDI note 0 = C-1

        format!("{}{}", NOTE_NAMES[pitch_class as usize], octave)
    }

    fn ticks_to_micros(&self, ticks: u64) -> u64 {
        let ubp = MICROSECONDS_PER_MINUTE / self.midi.bpm;

        ((ticks as f64 * ubp) / self.midi.tpb as f64).round() as u64
    }

    /// Plays the song.
    pub async fn play(mut self) -> JoyConResult<()> {
        let start_time = Instant::now();

        for event in &self.midi.note_events {
            let start_offset = self.ticks_to_micros(event.start_tick);
            let end_offset = self.ticks_to_micros(event.end_tick);
            let duration_us = end_offset.saturating_sub(start_offset);

            let frequency = MIDI_NOTE_TO_FREQUENCIES[event.key as usize];
            let amplitude = (event.velocity as f32 / MAX_VELOCITY).powf(2.0);

            println!(
                "Note ({}, {:?}), (channel {}) -> freq: {:.2} Hz, amp: {:.2}, duration: {}μs",
                Self::note_name(event.key),
                event,
                match event.channel {
                    Some(channel) => channel,
                    None => unreachable!(),
                },
                frequency,
                amplitude,
                duration_us
            );

            sleep_until(start_time + Duration::from_micros(start_offset)).await;

            self.driver.rumble((
                Some(Rumble::new(frequency, amplitude)),
                Some(Rumble::new(frequency, amplitude)),
            ))?;

            sleep_until(start_time + Duration::from_micros(end_offset)).await;

            self.driver
                .rumble((Some(Rumble::stop()), Some(Rumble::stop())))?;
        }

        Ok(())
    }
}
