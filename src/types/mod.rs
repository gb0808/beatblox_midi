pub trait Util {
    fn to_float(&self) -> f32;

    fn cmp(&self, b: &Self) -> f32;

    fn get_overflow(&self, precesion: &Self) -> PrecisionOverflow;
}

/// A type alias to help with readability.
pub type DurationType = (NoteDuration, NoteDurationModifier);

/// A container for helping with parsing notes with a precision value. The first item is the 
/// clipped note duration and the second value is the overflow duration.
pub type PrecisionOverflow = (f32, f32);

impl Util for DurationType {
    /// Converts a note duration to a float in order to help with comparison.
    fn to_float(&self) -> f32 {
        let mut duration_mod: f32 = 1.0;
        if self.1 == NoteDurationModifier::Dotted {
            duration_mod = 1.5;
        } else if self.1 == NoteDurationModifier::DoubleDotted {
            duration_mod = 1.75;
        }
        
        match self.0 {
            NoteDuration::WHOLE => return 64.0 * duration_mod,
            NoteDuration::HALF => return 32.0 * duration_mod,
            NoteDuration::QUARTER => return 16.0 * duration_mod,
            NoteDuration::EIGHTH => return 8.0 * duration_mod,
            NoteDuration::SIXTEENTH => return 4.0 * duration_mod,
            NoteDuration::THIRTYSECOND => return 2.0 * duration_mod,
            NoteDuration::SIXTYFOURTH => return 1.0 * duration_mod,
            NoteDuration::NaN => return 0.0,
        }
    }

    /// Compares note durations.
    /// 
    /// Returns a positive number if `self` is greater than `b`. Returns a negative number 
    /// if `self` is less than `b`. Returns zero if `self` is equivalent to `b`.
    fn cmp(&self, b: &DurationType) -> f32 {
        return self.to_float() - b.to_float()
    }

    /// A helper function for parsing notes that can not be evenly divied into the note precision.
    fn get_overflow(&self, precision: &DurationType) ->  PrecisionOverflow {
        let cliped_note = self.to_float() as u16 / precision.to_float() as u16;
        let overflow = self.to_float() % precision.to_float();
        return (cliped_note as f32, overflow);
    }
}

/// An array containing the beat lengths for all possible note durations.
pub const POSSIBLE_NOTE_LENGTHS: [f32; 21] = [
    0.0625, 0.09375, 0.109375, 0.125, 0.1875, 0.21875, 
    0.25, 0.375, 0.4375, 0.5, 0.75, 0.875, 1.0, 1.5, 
    1.75, 2.0, 3.0, 3.5, 4.0, 6.0, 7.0
];

/// Represents a note duration.
#[derive(Clone, PartialEq, Eq)]
pub enum NoteDuration { 
    WHOLE, 
    HALF, 
    QUARTER, 
    EIGHTH, 
    SIXTEENTH, 
    THIRTYSECOND, 
    SIXTYFOURTH,
    NaN,
}

/// Modifiers that may be added onto a note duration.
#[derive(Clone, PartialEq, Eq)]
pub enum NoteDurationModifier {
    None,
    Dotted,
    DoubleDotted,
}

/// Represents the content of a midi track.
#[derive(Clone)]
pub struct Track {
    /// The name of the track.
    pub name: String,
    /// A vector of all the notes played in the track.
    pub notes: Vec<NoteWrapper>
}

/// A wrapper for a musical note.
#[derive(Clone)]
pub enum NoteWrapper {
    PlainNote(Note),
    ModifiedNote(NoteModifier),
    Rest(Note),
}

/// Simulates a beatblox modifier being placed on a note.
#[derive(Clone)]
pub enum NoteModifier {
    TiedNote(Vec<NoteWrapper>),
    Chord(Vec<NoteWrapper>),
}

/// The basic representation of a note.
#[derive(Clone)]
pub struct Note {
    pub value: u8,
    pub duration: DurationType,
    pub velocity: u8,
}

/// A musical time signature.
#[derive(Clone, Copy)]
pub struct TimeSignature {
    /// The number of beats in a measure.
    pub beat_count: u8,
    /// The beat division.
    pub beat_type: u8,
    /// The time at which the time signature first occurs in the piece.
    /// 
    /// This allows for the handling of time signature changes.
    pub time_of_occurance: u32,
}

/// Maps a raw beat value to a `NoteDuration`. 
pub fn beat_type_map(beats: f32, beat_type: f32) -> DurationType {
    match beats * num_traits::pow(beat_type, 2) {
        112.0 => return (NoteDuration::WHOLE, NoteDurationModifier::DoubleDotted),
        96.0 => return (NoteDuration::WHOLE, NoteDurationModifier::Dotted),
        64.0 => return (NoteDuration::WHOLE, NoteDurationModifier::None),
        56.0 => return (NoteDuration::HALF, NoteDurationModifier::DoubleDotted),
        48.0 => return (NoteDuration::HALF, NoteDurationModifier::Dotted),
        32.0 => return (NoteDuration::HALF, NoteDurationModifier::None),
        28.0 => return (NoteDuration::QUARTER, NoteDurationModifier::DoubleDotted),
        24.0 => return (NoteDuration::QUARTER, NoteDurationModifier::Dotted),
        16.0 => return (NoteDuration::QUARTER, NoteDurationModifier::None),
        14.0 => return (NoteDuration::EIGHTH, NoteDurationModifier::DoubleDotted),
        12.0 => return (NoteDuration::EIGHTH, NoteDurationModifier::Dotted),
        8.0 => return (NoteDuration::EIGHTH, NoteDurationModifier::None),
        7.0 => return (NoteDuration::SIXTEENTH, NoteDurationModifier::DoubleDotted),
        6.0 => return (NoteDuration::SIXTEENTH, NoteDurationModifier::Dotted),
        4.0 => return (NoteDuration::SIXTEENTH, NoteDurationModifier::None),
        3.5 => return (NoteDuration::THIRTYSECOND, NoteDurationModifier::DoubleDotted),
        3.0 => return (NoteDuration::THIRTYSECOND, NoteDurationModifier::Dotted),
        2.0 => return (NoteDuration::THIRTYSECOND, NoteDurationModifier::None),
        1.75 => return (NoteDuration::SIXTYFOURTH, NoteDurationModifier::DoubleDotted),
        1.5 => return (NoteDuration::SIXTYFOURTH, NoteDurationModifier::Dotted),
        1.0 => return (NoteDuration::SIXTYFOURTH, NoteDurationModifier::None),
        _ => return (NoteDuration::NaN, NoteDurationModifier::None),
    }
}

/// A helper function to create a `NoteWrapper` object.
pub fn note_wrapper_builder(value: u8, duration: DurationType, velocity: u8) -> NoteWrapper {
    if value == 255 {
        return NoteWrapper::Rest(Note {value: value, duration: duration, velocity: velocity});
    }
    return NoteWrapper::PlainNote(Note {value: value, duration: duration, velocity: velocity});
}

/// Pretty prints a `NoteWrapper` object.
pub fn print_note_wrapper(note: &NoteWrapper) {
    match note {
        NoteWrapper::PlainNote(n) => {
            let duration_str = note_duration_str(&n.duration.0);
            let mod_str = note_duration_mod_str(&n.duration.1);
            print!("Note: {} | ", n.value);
            print!("Duration: {} {} | ", mod_str, duration_str);
            println!("Velocity: {}", n.velocity);
        },
        NoteWrapper::Rest(r) => {
            let duration_str = note_duration_str(&r.duration.0);
            let mod_str = note_duration_mod_str(&r.duration.1);
            println!("Rest | Duration:{} {}", mod_str, duration_str);
        },
        NoteWrapper::ModifiedNote(v) => {
            if let NoteModifier::TiedNote(t) = v {
                println!("====Tied Notes====");
                for n in t { 
                    print_note_wrapper(n); 
                }
                println!("==================");
            } else if let NoteModifier::Chord(c) = v {
                println!("====Chord====");
                for n in c { 
                    print_note_wrapper(n); 
                }
                println!("=============");
            }
        },
    }
}

/// Converts a `NoteDuration` to a `&str`.
fn note_duration_str(note_duration: &NoteDuration) -> &str {
    match note_duration {
        NoteDuration::WHOLE => return "whole note",
        NoteDuration::HALF => return "half note", 
        NoteDuration::QUARTER => return "quarter note", 
        NoteDuration::EIGHTH => return "eighth note", 
        NoteDuration::SIXTEENTH => return "sixteenth note", 
        NoteDuration::THIRTYSECOND => return "thirtysecond note", 
        NoteDuration::SIXTYFOURTH => return "sixtyfourth note",
        NoteDuration::NaN => return "unknown note",
    }
}

/// Converts a `NoteDurationModifier` to a `&str`.
fn note_duration_mod_str(note_duration_modifier: &NoteDurationModifier) -> &str {
    match note_duration_modifier {
        NoteDurationModifier::None => return "",
        NoteDurationModifier::Dotted => return "dotted",
        NoteDurationModifier::DoubleDotted => return "double dotted",
    }
}