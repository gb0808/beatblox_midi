/// An array containing the beat lengths for all possible note durations.
pub const POSSIBLE_NOTE_LENGTHS: [f32; 18] = [
    0.125, 0.1875, 0.21875, 0.25, 0.375, 0.4375, 
    0.5, 0.75, 0.875, 1.0, 1.5, 1.75, 2.0, 3.0, 
    3.5, 4.0, 6.0, 7.0
];

/// The defualt note precision for parsing through files.
pub const DEFAULT_DURATION_PRECISION: DurationType = DurationType {
    duration: NoteDuration::THIRTYSECOND,
    modifier: NoteDurationModifier::None,
};

/// Represents a note duration.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NoteDuration { 
    WHOLE, 
    HALF, 
    QUARTER, 
    EIGHTH, 
    SIXTEENTH, 
    THIRTYSECOND, 
    NaN,
}

impl NoteDuration {
    /// Converts the enum to a string.
    pub fn to_string(&self) -> &str {
        match self {
            NoteDuration::WHOLE => return "whole note",
            NoteDuration::HALF => return "half note", 
            NoteDuration::QUARTER => return "quarter note", 
            NoteDuration::EIGHTH => return "eighth note", 
            NoteDuration::SIXTEENTH => return "sixteenth note", 
            NoteDuration::THIRTYSECOND => return "thirtysecond note", 
            NoteDuration::NaN => return "unknown note",
        }
    }

    fn shift(&self, shift: u8) -> Self {
        let mut temp = self.clone();
        if shift > 2 {
            for _ in 2..shift {
                temp = temp.shift_down();
            }
        } else if shift < 2 {
            for _ in shift..2 {
                temp = temp.shift_up();
            }
        }
        return temp;
    }

    fn reverse_shift(&self, shift: u8) -> Self {
        let mut temp = self.clone();
        if shift > 2 {
            for _ in 2..shift {
                temp = temp.shift_up();
            }
        } else if shift < 2 {
            for _ in shift..2 {
                temp = temp.shift_down();
            }
        }
        return temp;
    }

    fn shift_up(&self) -> Self {
        match self {
            NoteDuration::WHOLE => return NoteDuration::NaN,
            NoteDuration::HALF => return NoteDuration::WHOLE, 
            NoteDuration::QUARTER => return NoteDuration::HALF, 
            NoteDuration::EIGHTH => return NoteDuration::QUARTER, 
            NoteDuration::SIXTEENTH => return NoteDuration::EIGHTH, 
            NoteDuration::THIRTYSECOND => return NoteDuration::SIXTEENTH, 
            NoteDuration::NaN => return NoteDuration::NaN,
        }
    }

    fn shift_down(&self) -> Self{
        match self {
            NoteDuration::WHOLE => return NoteDuration::HALF,
            NoteDuration::HALF => return NoteDuration::QUARTER, 
            NoteDuration::QUARTER => return NoteDuration::EIGHTH, 
            NoteDuration::EIGHTH => return NoteDuration::SIXTEENTH, 
            NoteDuration::SIXTEENTH => return NoteDuration::THIRTYSECOND, 
            NoteDuration::THIRTYSECOND => return NoteDuration::NaN, 
            NoteDuration::NaN => return NoteDuration::NaN,
        }
    }
}

/// Modifiers that may be added onto a note duration.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NoteDurationModifier {
    None,
    Dotted,
    DoubleDotted,
}

impl NoteDurationModifier {
    /// Converts the enum to a string.
    pub fn to_string(&self) -> &str {
        match self {
            NoteDurationModifier::None => return "",
            NoteDurationModifier::Dotted => return "dotted",
            NoteDurationModifier::DoubleDotted => return "double dotted",
        }
    }
}

/// A struct to help with readability.
#[derive(Clone)]
pub struct  DurationType {
    pub duration: NoteDuration,
    pub modifier: NoteDurationModifier,
}

impl DurationType {
    pub fn quantize(&self, beat_type: u8, precision_beats: f32) -> Self {
        let beats = self.get_beat_count(beat_type);
        if beats < precision_beats {
            return Self::beat_type_map(precision_beats, beat_type);
        }
        let qualtized_beats = beats - (beats % precision_beats);
        return Self::beat_type_map(qualtized_beats, beat_type);
    }

    /// Maps a number of beats to a `DurationType`.
    pub fn beat_type_map(beats: f32, beat_type: u8) -> DurationType {
        match beats {
            7.0 => DurationType {
                duration: NoteDuration::WHOLE.shift(beat_type),
                modifier: NoteDurationModifier::DoubleDotted,
            },
            6.0 => DurationType {
                duration: NoteDuration::WHOLE.shift(beat_type),
                modifier: NoteDurationModifier::Dotted,
            },
            4.0 => DurationType {
                duration: NoteDuration::WHOLE.shift(beat_type),
                modifier: NoteDurationModifier::None,
            },
            3.5 => DurationType {
                duration: NoteDuration::HALF.shift(beat_type),
                modifier: NoteDurationModifier::DoubleDotted,
            },
            3.0 => DurationType {
                duration: NoteDuration::HALF.shift(beat_type),
                modifier: NoteDurationModifier::Dotted,
            },
            2.0 => DurationType {
                duration: NoteDuration::HALF.shift(beat_type),
                modifier: NoteDurationModifier::None,
            },
            1.75 => DurationType {
                duration: NoteDuration::QUARTER.shift(beat_type),
                modifier: NoteDurationModifier::DoubleDotted,
            },
            1.5 => DurationType {
                duration: NoteDuration::QUARTER.shift(beat_type),
                modifier: NoteDurationModifier::Dotted,
            },
            1.0 => DurationType {
                duration: NoteDuration::QUARTER.shift(beat_type),
                modifier: NoteDurationModifier::None,
            },
            0.875 => DurationType {
                duration: NoteDuration::EIGHTH.shift(beat_type),
                modifier: NoteDurationModifier::DoubleDotted,
            },
            0.75 => DurationType {
                duration: NoteDuration::EIGHTH.shift(beat_type),
                modifier: NoteDurationModifier::Dotted,
            },
            0.5 => DurationType {
                duration: NoteDuration::EIGHTH.shift(beat_type),
                modifier: NoteDurationModifier::None,
            },
            0.4375 => DurationType {
                duration: NoteDuration::SIXTEENTH.shift(beat_type),
                modifier: NoteDurationModifier::DoubleDotted,
            },
            0.375 => DurationType {
                duration: NoteDuration::SIXTEENTH.shift(beat_type),
                modifier: NoteDurationModifier::Dotted,
            },
            0.25 => DurationType {
                duration: NoteDuration::SIXTEENTH.shift(beat_type),
                modifier: NoteDurationModifier::None,
            },
            0.21875 => DurationType {
                duration: NoteDuration::THIRTYSECOND.shift(beat_type),
                modifier: NoteDurationModifier::DoubleDotted,
            },
            0.1875 => DurationType {
                duration: NoteDuration::THIRTYSECOND.shift(beat_type),
                modifier: NoteDurationModifier::Dotted,
            },
            0.125 => DurationType {
                duration: NoteDuration::THIRTYSECOND.shift(beat_type),
                modifier: NoteDurationModifier::None,
            },
            _ => DurationType {
                duration: NoteDuration::NaN,
                modifier: NoteDurationModifier::None,
            },
        }
    }

    /// A helper function that returns the number of beats in this Duration type.
    pub fn get_beat_count(&self, beat_type: u8) -> f32 {
        let duration = self.duration.reverse_shift(beat_type);
        let mod_factor: f32;
        match self.modifier {
            NoteDurationModifier::DoubleDotted => mod_factor = 1.75,
            NoteDurationModifier::Dotted => mod_factor = 1.5,
            NoteDurationModifier::None => mod_factor = 1.0,
        }
        match duration {
            NoteDuration::WHOLE => 4.0 * mod_factor,
            NoteDuration::HALF => 2.0 * mod_factor, 
            NoteDuration::QUARTER => 1.0 * mod_factor, 
            NoteDuration::EIGHTH => 0.5 * mod_factor, 
            NoteDuration::SIXTEENTH => 0.25 * mod_factor, 
            NoteDuration::THIRTYSECOND => 0.125 * mod_factor, 
            NoteDuration::NaN => 0.0,
        }
    }
}