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
    beats: f32,
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
    let mut chord_notes: Vec<NoteWrapper> = Vec::new();
    let mut beat_marker = 0.0;
    let raw_note_data = get_raw_note_data(track, midi.ticks_per_beat);
    let beat_type = midi.time_signatures[0].beat_type;
    let quantized_notes = quantize(&raw_note_data, precision, beat_type);

    for note_data in quantized_notes {
        if note_data.0 != beat_marker {
            if chord_notes.len() == 1 {
                notes.push(chord_notes[0].clone());
            } else {
                notes.push(NoteWrapper::ModifiedNote(NoteModifier::Chord(chord_notes)));
            }
            chord_notes = Vec::new();
            beat_marker = note_data.0;
        } 
        chord_notes.push(note_data.1);
    }   
    
    return notes;
}

fn quantize(
    raw_note_data: &Vec<RawNoteData>, 
    precision: &DurationType,
    beat_type: u8
) -> Vec<(f32, NoteWrapper)> {
    let mut notes = Vec::new();
    if raw_note_data.len() == 0 {
        return notes;
    }
   
    let mut cur_beat = 0.0;
    let precision_beats = precision.get_beat_count(beat_type);
    let mut cur_beat_marker = precision_beats;
    let mut prev_beat_marker = 0.0;
    let cur_note_index = 0;
    let cur_note = raw_note_data[cur_note_index];

    for note in raw_note_data {
        let value = note.value;
        let duration = DurationType::beat_type_map(note.beats, beat_type);
        let velocity = note.velocity;

        if duration.duration == NoteDuration::NaN {
            notes.push((
                prev_beat_marker, 
                NoteWrapper::ModifiedNote(get_tied_note(&cur_note, beat_type, precision)),
            ));
        } else {
            notes.push((
                prev_beat_marker, 
                NoteWrapper::build_note_wrapper(
                    value, duration.quantize(beat_type, precision_beats), velocity)
            ));
        }

        cur_beat += note.beats;
        if cur_beat >= cur_beat_marker {
            prev_beat_marker = cur_beat_marker;
        }
        while cur_beat >= cur_beat_marker {
            cur_beat_marker += precision_beats;
        }
    }
    return notes;
}

/// Gets the raw note data in a midi track.
fn get_raw_note_data(track: &Vec<midly::TrackEvent>, ticks_per_beat: f32) -> Vec<RawNoteData> {
    let mut status: bool = false;
    let mut cur_note_value: u8 = 0;
    let mut cur_velocity: u8 = 0;
    let mut cur_time: u32 = 0;
    let mut note_on_time: u32 = 0;
    let mut note_off_time: u32 = 0;
    let mut data: Vec<RawNoteData> = Vec::new();

    for event in track {
        let delta_t: u32 = event.delta.into();
        cur_time += delta_t;

        if let midly::TrackEventKind::Midi { channel: _, message } = event.kind {
            if let midly::MidiMessage::NoteOn {key, vel } = message {
                if status == false {
                    cur_velocity = vel.into();
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
                        velocity: cur_velocity,
                    });
                    note_off_time = cur_time;
                    status = false;
                }
            }
        }
    }

    return data;
}

/// Returns a tuple with the fist value containing a `NoteModifier` and the second value containing
/// the overflow. 
fn get_tied_note(note: &RawNoteData, beat_type: u8, precision: &DurationType) -> NoteModifier {
    let precision_beats = precision.get_beat_count(beat_type);
    let mut notes: Vec<NoteWrapper> = Vec::new();
    let mut remaining_beats: f32 = note.beats;
    while remaining_beats > 0.0 {
        let nested_beat_value = get_nested_beat_value(remaining_beats, precision_beats);
        let new_duration = DurationType::beat_type_map(nested_beat_value, beat_type);
        remaining_beats -= nested_beat_value;
        notes.push(NoteWrapper::build_note_wrapper(
            note.value, new_duration.quantize(beat_type, precision_beats), note.velocity));
        if nested_beat_value == 0.0 {
            break;
        }
    }
    return NoteModifier::TiedNote(notes);
}

/// A helper function for parsing tied notes.
fn get_nested_beat_value(beats: f32, precision_beats: f32) -> f32 {
    let mut nested_beat_value = 0.0;
    for i in 0..POSSIBLE_NOTE_LENGTHS.len() {
        let fit = POSSIBLE_NOTE_LENGTHS[i] % precision_beats == 0.0;
        if POSSIBLE_NOTE_LENGTHS[i] <= beats && fit {
            nested_beat_value = POSSIBLE_NOTE_LENGTHS[i];
        }
    }
    return nested_beat_value;
}