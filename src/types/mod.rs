use num_traits;

pub const POSSIBLE_NOTE_LENGTHS: [f32; 21] = [
    0.0625, 0.09375, 0.109375, 0.125, 0.1875, 0.21875, 
    0.25, 0.375, 0.4375, 0.5, 0.75, 0.875, 1.0, 1.5, 
    1.75, 2.0, 3.0, 3.5, 4.0, 6.0, 7.0
];

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

#[derive(Clone)]
pub enum NoteDurationModifier {
    None,
    Dotted,
    DoubleDotted,
}

#[derive(Clone)]
pub struct Track {
    pub name: String,
    pub notes: Vec<NoteWrapper>
}

#[derive(Clone)]
pub enum NoteWrapper {
    PlainNote(Note),
    ModifiedNote(NoteModifier),
    Rest(Note),
}

#[derive(Clone)]
pub enum NoteModifier {
    TiedNote(Vec<NoteWrapper>),
}

#[derive(Clone)]
pub struct Note {
    pub value: u8,
    pub duration: (NoteDuration, NoteDurationModifier),
    pub velocity: u8,
}

#[derive(Clone, Copy)]
pub struct TimeSignature {
    pub beat_count: u8,
    pub beat_type: u8,
    pub time_of_occurance: u32,
}

pub fn beat_type_map(beats: f32, beat_type: f32) -> (NoteDuration, NoteDurationModifier) {
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

pub fn note_wrapper_builder(
    value: u8, duration: (NoteDuration, NoteDurationModifier), velocity: u8) -> NoteWrapper {
    if value == 255 {
        return NoteWrapper::Rest(Note {value: value, duration: duration, velocity: velocity});
    }
    return NoteWrapper::PlainNote(Note {value: value, duration: duration, velocity: velocity});
}

pub fn print_note_wrapper(note: &NoteWrapper) {
    match note {
        NoteWrapper::PlainNote(n) => {
            let duration_str = note_duration_str(&n.duration.0);
            let mod_str = note_duration_mod_str(&n.duration.1);
            println!("Note: {} for duration {} + {}", n.value, duration_str, mod_str);
        },
        NoteWrapper::Rest(r) => {
            let duration_str = note_duration_str(&r.duration.0);
            let mod_str = note_duration_mod_str(&r.duration.1);
             println!("Rest for duration {} + {}", duration_str, mod_str);
        },
        NoteWrapper::ModifiedNote(v) => {
            let NoteModifier::TiedNote(t) = v;
            println!("====Tied Notes====");
            for n in t { 
                print_note_wrapper(n); 
            }
            println!("==================");
        },
    }
}

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

fn note_duration_mod_str(note_duration_modifier: &NoteDurationModifier) -> &str {
    match note_duration_modifier {
        NoteDurationModifier::None => return "none",
        NoteDurationModifier::Dotted => return "dotted",
        NoteDurationModifier::DoubleDotted => return "double dotted",
    }
}