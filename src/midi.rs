use midly::MidiMessage;

pub trait Midi {
    fn is_note(&self) -> bool;
}

impl Midi for MidiMessage {
    /// Check if the midi message status is `NoteOn` or `NoteOff`.
    fn is_note(&self) -> bool {
        matches!(
            self,
            MidiMessage::NoteOff { .. } | MidiMessage::NoteOn { .. }
        )
    }
}
