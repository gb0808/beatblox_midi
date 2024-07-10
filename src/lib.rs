use midly::Smf;
use std::fs;

mod parsing;
mod types;

#[derive(Clone)]
pub struct Midi {
    bmp: u32,
    time_signatures: Vec<types::TimeSignature>,
    ticks_per_beat: f32,
    tracks: Vec<types::Track>,
}

impl Midi {
    pub fn parse(dir: String) -> Midi {
        let contents = fs::read(dir).unwrap();
        let smf = Smf::parse(&contents).unwrap();
        let mut midi = Midi::new(&smf);
        parsing::load_tracks(&mut midi, &smf);
        return midi;
    }

    pub fn print(&self) {
        println!("BPM: {}", self.bmp);
        for track in &self.tracks {
            println!("=============== {} ===============", track.name);
            for note in &track.notes {
                types::print_note_wrapper(note);
            }
        }
    }

    fn new(smf: &midly::Smf) -> Midi {
        Midi {
            bmp: parsing::get_bpm(&smf.tracks[0]),
            time_signatures: parsing::get_time_signature(&smf.tracks[0]),
            ticks_per_beat: parsing::get_ticks_per_beat(&smf.header),
            tracks: Vec::new(),
        }
    }
}
