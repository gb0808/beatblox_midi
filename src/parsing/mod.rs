use super::Midi;
use super::types;
use num_traits;

struct RawNoteData {
    value: u8,
    beats: f32,
    velocity: u8,
}

pub fn get_ticks_per_beat(header: &midly::Header) -> f32 {
    let midly::Header { format: _, timing } = header;
    if let midly::Timing::Metrical(x) = timing {
        let ticks_per_beat: u16 = (*x).into();
        return ticks_per_beat as f32;
    }
    panic!("Timing format not supported");
}

pub fn get_bpm(track: &Vec<midly::TrackEvent>) -> u32 {
    for event in track {
        if let midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(tempo)) = event.kind {
            let microseconds_per_beat: u32 = tempo.into();
            return microseconds_per_beat / 1000000 * 60;
        }
    }
    return 0;
}

pub fn get_time_signature(track: &Vec<midly::TrackEvent>) -> Vec<types::TimeSignature> {
    let mut time_signatures: Vec<types::TimeSignature> = Vec::new();
    let mut cur_time: u32 = 0;
    for event in track {
        let delta_t: u32 = event.delta.into();
        cur_time += delta_t;
        if let midly::TrackEventKind::Meta(message) = event.kind {
            if let midly::MetaMessage::TimeSignature(numerator, denominator, _, _) = message {
                time_signatures.push(types::TimeSignature {
                    beat_count: numerator,
                    beat_type: num_traits::pow(2, denominator.into()),
                    time_of_occurance: cur_time,
                });
            }
        }
    }
    return time_signatures;
}

pub fn load_tracks(midi: &mut Midi, smf: &midly::Smf) {
    let tmp = midi.clone();
    for track in &smf.tracks {
        midi.tracks.push(parse_track(&tmp, track));
    }
}

fn parse_track(midi: &Midi, track: &Vec<midly::TrackEvent>) -> types::Track {
    types::Track { 
        name: get_name(track), 
        notes: get_notes(midi, track),
    }
}

fn get_name(track: &Vec<midly::TrackEvent>) -> String {
    for event in track {
        if let midly::TrackEventKind::Meta(midly::MetaMessage::InstrumentName(s)) = event.kind {
            let raw_string: Vec<u8> = s.to_vec();
            return String::from_utf8(raw_string).unwrap();
        }
    }
    return String::from("");
}

fn get_notes(midi: &Midi, track: &Vec<midly::TrackEvent>) -> Vec<types::NoteWrapper> {
    let mut notes = Vec::new();
    let raw_note_data = get_raw_note_data(track, midi.ticks_per_beat);
    let time_signature = midi.time_signatures[0];

    for note in raw_note_data {
        let duration_type = types::beat_type_map(note.beats, time_signature.beat_type as f32);
        if duration_type.0 == types::NoteDuration::NaN {
            let tied_note = get_tied_note(note, time_signature.beat_type as f32);
            notes.push(types::NoteWrapper::ModifiedNote(tied_note));
        } else {
            notes.push(types::note_wrapper_builder(note.value, duration_type, note.velocity));
        }
    }
    
    return notes;
}

fn get_raw_note_data(track: &Vec<midly::TrackEvent>, ticks_per_beat: f32) -> Vec<RawNoteData> {
    let mut status: bool = false;
    let mut cur_note_value: u8 = 0;
    let mut cur_time: u32 = 0;
    let mut note_on_time: u32 = 0;
    let mut note_off_time: u32 = 0;
    let mut data: Vec<RawNoteData> = Vec::new();

    for event in track {
        let delta_t: u32 = event.delta.into();
        cur_time += delta_t;

        if let midly::TrackEventKind::Midi { channel: _, message } = event.kind {
            if let midly::MidiMessage::NoteOn {key, vel: _ } = message {
                if status == false {
                    cur_note_value = key.into();
                    note_on_time = cur_time;
                    status = true;
                    if note_on_time - note_off_time != 0 {
                        data.push(RawNoteData { 
                            value: 255, 
                            beats: (note_on_time - note_off_time) as f32 / ticks_per_beat,
                            velocity: 0,
                        });
                    }
                } 
            }
            else if let midly::MidiMessage::NoteOff { key, vel: _ } = message {
                if status == true && cur_note_value == key {
                    data.push(RawNoteData { 
                        value: cur_note_value, 
                        beats: (cur_time - note_on_time) as f32 / ticks_per_beat,
                        velocity: 0,
                    });
                    note_off_time = cur_time;
                    status = false;
                }
            }
        }
    }

    return data;
}

fn get_tied_note(note: RawNoteData, beat_type: f32) -> types::NoteModifier {
    let mut notes: Vec<types::NoteWrapper> = Vec::new();
    let mut remaining_beats: f32 = note.beats;
    while remaining_beats > 0.0 {
        let nested_beat_value = get_nested_beat_value(remaining_beats);
        let new_duration = types::beat_type_map(nested_beat_value, beat_type as f32);
        remaining_beats -= nested_beat_value;
        notes.push(types::note_wrapper_builder(note.value, new_duration, note.velocity));
    }
    return types::NoteModifier::TiedNote(notes);
}

fn get_nested_beat_value(beats: f32) -> f32 {
    for i in 1..types::POSSIBLE_NOTE_LENGTHS.len() {
        if types::POSSIBLE_NOTE_LENGTHS[i] > beats {
            return types::POSSIBLE_NOTE_LENGTHS[i - 1];
        }
    }
    return types::POSSIBLE_NOTE_LENGTHS[types::POSSIBLE_NOTE_LENGTHS.len() - 1];
}