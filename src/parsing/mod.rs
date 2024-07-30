pub mod duration;
pub mod symbols;

use duration::NoteDuration;
use crate::Midi;
use crate::parsing::duration::DurationType;
use crate::parsing::duration::POSSIBLE_NOTE_LENGTHS;
use crate::parsing::symbols::NoteModifier;
use crate::parsing::symbols::NoteWrapper;
use crate::parsing::symbols::TimeSignature;

/// Represents the content of a midi track.
#[derive(Clone)]
pub struct Track {
    /// The name of the track.
    pub name: String,
    /// A vector of all the notes played in the track.
    pub notes: Vec<NoteWrapper>
}

/// Represents a raw note data taken from the midi file.
#[derive(Clone, Copy)]
struct RawNoteData {
    value: u8,
    start_beat: f32,
    velocity: u8,
}

/// Gets the number of ticks in each beat.
pub fn get_ticks_per_beat(header: &midly::Header) -> f32 {
    let midly::Header { format: _, timing } = header;
    if let midly::Timing::Metrical(x) = timing {
        let ticks_per_beat: u16 = (*x).into();
        return ticks_per_beat as f32;
    }
    panic!("Timing format not supported");
}

/// Gets the tempo of a midi file.
pub fn get_bpm(track: &Vec<midly::TrackEvent>) -> u32 {
    for event in track {
        if let midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(tempo)) = event.kind {
            let microseconds_per_beat: u32 = tempo.into();
            return microseconds_per_beat / 1000000 * 60;
        }
    }
    return 0;
}

/// Returns all time signatures in the midi file.
pub fn get_time_signature(track: &Vec<midly::TrackEvent>) -> Vec<TimeSignature> {
    let mut time_signatures: Vec<TimeSignature> = Vec::new();
    let mut cur_time: u32 = 0;
    for event in track {
        let delta_t: u32 = event.delta.into();
        cur_time += delta_t;
        if let midly::TrackEventKind::Meta(message) = event.kind {
            if let midly::MetaMessage::TimeSignature(numerator, denominator, _, _) = message {
                time_signatures.push(TimeSignature {
                    beat_count: numerator,
                    beat_type: denominator.into(),
                    time_of_occurance: cur_time,
                });
            }
        }
    }
    return time_signatures;
}

/// Loads all the tracks in a midi file.
/// 
/// `midi` holds the newly created `Midi` object.
/// 
/// `smf` holds the `midly::Smf` object being used to parse through the midi file.
/// 
/// The `precision` parameter allows the user to set the degree of precision they would like
/// when parsing. Any notes shorter than the value specified in the `precision` parameter
/// will be grouped as a chord.
pub fn load_tracks(midi: &mut Midi, smf: &midly::Smf, precision: &DurationType) {
    let tmp = midi.clone();
    for track in &smf.tracks {
        midi.tracks.push(parse_track(&tmp, track, precision));
    }
}

/// A helper function to build the `Track Object`.
fn parse_track(midi: &Midi, track: &Vec<midly::TrackEvent>, precision: &DurationType) -> Track {
    Track { 
        name: get_name(track), 
        notes: get_notes(midi, track, precision),
    }
}

/// Gets the name of a midi track.
fn get_name(track: &Vec<midly::TrackEvent>) -> String {
    for event in track {
        if let midly::TrackEventKind::Meta(midly::MetaMessage::InstrumentName(s)) = event.kind {
            let raw_string: Vec<u8> = s.to_vec();
            return String::from_utf8(raw_string).unwrap();
        }
    }
    return String::from("");
}

/// Gets all the notes in a midi track. 
/// 
/// Does this by formatting the raw midi data.
fn get_notes(
    midi: &Midi, 
    track: &Vec<midly::TrackEvent>, 
    precision: &DurationType
) -> Vec<NoteWrapper> {
    let mut notes = Vec::new();
    let raw_note_data = get_raw_note_data(track, midi.ticks_per_beat);
    let beat_type = midi.time_signatures[0].beat_type;
    let precision_beats = precision.get_beat_count(beat_type);
    let quantized_notes = quantize(&raw_note_data, precision_beats);

    let mut cur_note: Vec<(u8, u8)> = Vec::new();
    let mut beat_length = precision_beats;
    for i in 0..quantized_notes.len() {
        if quantized_notes[i].len() != 0 {
            if cur_note.len() == 1 {
                let value = cur_note[0].0;
                let velocity = cur_note[0].1;
                notes.push(parse_note_data((value, velocity), beat_length, beat_type));
            } else if cur_note.len() != 0{
                let mut chord = Vec::new();
                for note_data in cur_note.clone() {
                    let value = note_data.0;
                    let velocity = note_data.1;
                    chord.push(parse_note_data((value, velocity), beat_length, beat_type));
                }
                notes.push(NoteWrapper::ModifiedNote(NoteModifier::Chord(chord)));
            }
            cur_note = quantized_notes[i].clone();
            beat_length = precision_beats;
        } else {
            beat_length += precision_beats;
        }
    }
    
    return notes;
}

fn parse_note_data((value, velocity): (u8, u8), beat_length: f32, beat_type: u8) -> NoteWrapper {
    let duration = DurationType::beat_type_map(beat_length, beat_type);
    if duration.duration == NoteDuration::NaN {
        return NoteWrapper::ModifiedNote(get_tied_note((value, beat_length, velocity), beat_type));
    } else {
        return NoteWrapper::build_note_wrapper(value, duration, velocity);
    }
}

fn quantize(raw_note_data: &Vec<RawNoteData>, precision_beats: f32) -> Vec<Vec<(u8, u8)>> {
    if raw_note_data.len() == 0 {
        return Vec::new();
    }
    let total_precision_beats = get_total_precision_beats(raw_note_data, precision_beats);
    let mut notes = vec![Vec::new(); total_precision_beats];
    for note in raw_note_data {
        notes[(note.start_beat / precision_beats) as usize].push((note.value, note.velocity));
    }
    return notes;
}

/// Gets the raw note data in a midi track.
fn get_raw_note_data(track: &Vec<midly::TrackEvent>, ticks_per_beat: f32) -> Vec<RawNoteData> {
    let mut cur_time: u32 = 0;
    let mut cur_velocity: u8 = 0;
    let mut note_on_time: u32 = 0;
    let mut note_off_time: u32 = 0;
    let mut data: Vec<RawNoteData> = Vec::new();

    for event in track {
        let delta_t: u32 = event.delta.into();
        cur_time += delta_t;

        if let midly::TrackEventKind::Midi { channel: _, message } = event.kind {
            if let midly::MidiMessage::NoteOn {key: _, vel } = message {
                cur_velocity = vel.into();
                note_on_time = cur_time;
                if note_on_time - note_off_time != 0 {
                    data.push(RawNoteData {
                        value: 255,
                        start_beat: note_off_time as f32 / ticks_per_beat,
                        velocity: 0,
                    });
                }
            }
            else if let midly::MidiMessage::NoteOff { key , vel: _ } = message {
                data.push(RawNoteData {
                    value: key.into(),
                    start_beat: note_on_time as f32 / ticks_per_beat,
                    velocity: cur_velocity,
                });
                note_off_time = cur_time;
            }
        }
    }

    return data;
}

fn get_total_precision_beats(raw_note_data: &Vec<RawNoteData>, precision_beats: f32) -> usize {
    let total_beats = raw_note_data[raw_note_data.len() - 1].start_beat;
    let mut total_precision_beats = 0;
    let mut cur_beat = 0.0;
    while cur_beat <= total_beats {
        cur_beat += precision_beats;
        total_precision_beats += 1;
    }
    return total_precision_beats;
}

fn get_tied_note((value, duration, velocity): (u8, f32, u8), beat_type: u8) -> NoteModifier {
    let mut notes: Vec<NoteWrapper> = Vec::new();
    let mut remaining_beats: f32 = duration;
    while remaining_beats > 0.0 {
        let nested_beat_value = get_nested_beat_value(remaining_beats);
        let new_duration = DurationType::beat_type_map(nested_beat_value, beat_type);
        remaining_beats -= nested_beat_value;
        notes.push(NoteWrapper::build_note_wrapper(value, new_duration, velocity));
    }
    return NoteModifier::TiedNote(notes);
}

/// A helper function for parsing tied notes.
fn get_nested_beat_value(beats: f32) -> f32 {
    for i in 1..POSSIBLE_NOTE_LENGTHS.len() {
        if POSSIBLE_NOTE_LENGTHS[i] > beats {
            return POSSIBLE_NOTE_LENGTHS[i - 1];
        }
    }
    return POSSIBLE_NOTE_LENGTHS[POSSIBLE_NOTE_LENGTHS.len() - 1];
}