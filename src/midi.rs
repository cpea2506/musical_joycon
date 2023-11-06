use rimd::Status;

pub trait Midi {
    fn is_note(&self) -> bool;
}

impl Midi for Status {
    /// Check if the midi message status is `NoteOn` or `NoteOff`.
    fn is_note(&self) -> bool {
        matches!(self, Status::NoteOff | Status::NoteOn)
    }
}
