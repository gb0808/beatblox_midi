use crate::parsing::duration::DurationType;

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

impl NoteWrapper {
    /// A helper function to create a `NoteWrapper` object.
    pub fn build_note_wrapper(value: u8, duration: DurationType, velocity: u8) -> Self {
        if value == 255 {
            return NoteWrapper::Rest(Note {value: value, duration: duration, velocity: velocity});
        }
        return NoteWrapper::PlainNote(Note {value: value, duration: duration, velocity: velocity});
    }

    /// Pretty prints a `NoteWrapper` object.
    pub fn print(&self) {
        match self {
            NoteWrapper::PlainNote(n) => {
                let duration_str = n.duration.duration.to_string();
                let mod_str = n.duration.modifier.to_string();
                print!("Note: {} | ", n.value);
                print!("Duration: {} {} | ", mod_str, duration_str);
                println!("Velocity: {}", n.velocity);
            },
            NoteWrapper::Rest(r) => {
                let duration_str = r.duration.duration.to_string();
                let mod_str = r.duration.modifier.to_string();
                println!("Rest | Duration: {} {}", mod_str, duration_str);
            },
            NoteWrapper::ModifiedNote(v) => {
                if let NoteModifier::TiedNote(t) = v {
                    println!("====Tied Notes====");
                    for n in t { 
                        n.print(); 
                    }
                    println!("==================");
                } else if let NoteModifier::Chord(c) = v {
                    println!("++++++Chord+++++++");
                    for n in c { 
                        n.print(); 
                    }
                    println!("++++++++++++++++++");
                } else if let NoteModifier::Triplet(tr) = v {
                    println!("-----Triplet------");
                    for n in tr { 
                        n.print(); 
                    }
                    println!("------------------");
                }
            },
        }
    }
}

/// Simulates a beatblox modifier being placed on a note.
#[derive(Clone)]
pub enum NoteModifier {
    TiedNote(Vec<NoteWrapper>),
    Chord(Vec<NoteWrapper>),
    Triplet(Vec<NoteWrapper>),
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