use midly::Smf;
use std::fs;

mod parsing;
pub mod types;

/// The Midi structure is a netsblox-friendly representation of the parsed midi file.
#[derive(Clone)]
pub struct Midi {
    /// The initial tempo of the piece.
    bmp: u32,
    /// A list of time signatures that occur in the piece.
    time_signatures: Vec<types::TimeSignature>,
    /// Number of ticks in each beat.
    ticks_per_beat: f32,
    /// A list of tracks in the midi file.
    tracks: Vec<types::Track>,
}
impl Midi {
    /// Parses through a midi file found at `dir` and returns a `Midi` object.
    pub fn parse(dir: String) -> Midi {
        let precision = (types::NoteDuration::SIXTYFOURTH, types::NoteDurationModifier::None);
        return Midi::parse_with_precision(dir, precision);
    }

    /// Parses through a midi file found at 'dir' and returns a `Midi` object.
    /// 
    /// The `precision` parameter allows the user to set the degree of precision they would like
    /// when parsing. Any notes shorter than the value specified in the `precision` parameter
    /// will be grouped as a chord.
    pub fn parse_with_precision(dir: String, precision: types::DurationType) -> Midi {
        let contents = fs::read(dir).unwrap();
        let smf = Smf::parse(&contents).unwrap();
        let mut midi = Midi::new(&smf);
        parsing::load_tracks(&mut midi, &smf, &precision);
        return midi;
    }

    /// Pretty prints the contents of the `Midi` object.
    pub fn print(&self) {
        println!("BPM: {}", self.bmp);
        for track in &self.tracks {
            println!("=============== {} ===============", track.name);
            for note in &track.notes {
                types::print_note_wrapper(note);
            }
        }
    }

    /// Private constructor for a midi object.
    /// 
    /// Initially, the `tracks` field is empty and tracks must manually be loaded in with
    /// the funtion `parssing::load_tracks(...)`
    fn new(smf: &midly::Smf) -> Midi {
        Midi {
            bmp: parsing::get_bpm(&smf.tracks[0]),
            time_signatures: parsing::get_time_signature(&smf.tracks[0]),
            ticks_per_beat: parsing::get_ticks_per_beat(&smf.header),
            tracks: Vec::new(),
        }
    }
}
