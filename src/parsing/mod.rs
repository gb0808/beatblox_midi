pub mod duration;
pub mod symbols;

use duration::NoteDuration;
use crate::Midi;
use crate::parsing::duration::DurationType;
use crate::parsing::duration::POSSIBLE_NOTE_LENGTHS;
use crate::parsing::symbols::NoteModifier;
use crate::parsing::symbols::NoteWrapper;
use crate::parsing::symbols::TimeSignature;
use std::collections::VecDeque;

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
    key: u8,
    onset: u32,
    vel: u8,
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
/// 
/// The `triplet` parameter indicated if the user wants to scan for triplets. Scanning for
/// triplets requires extra resources.
pub fn load_tracks(midi: &mut Midi, smf: &midly::Smf, precision: &DurationType, triplet: bool) {
    let tmp = midi.clone();
    for track in &smf.tracks {
        midi.tracks.push(parse_track(&tmp, track, precision, triplet));
    }
}

/// A helper function to build the `Track Object`.
fn parse_track(
    midi: &Midi, 
    track: &Vec<midly::TrackEvent>, 
    precision: &DurationType,
    triplet: bool
) -> Track {
    Track { 
        name: get_name(track), 
        notes: get_notes(midi, track, precision, triplet),
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
    precision: &DurationType,
    triplet: bool
) -> Vec<NoteWrapper> {
    let beat_type = midi.time_signatures[0].beat_type;
    let precision_beat = precision.get_beat_count(beat_type);
    let divisions = if triplet { 
        4.0 / precision_beat / 2.0 * 1.5 
    } else { 
        1.0 / precision_beat
    };
    let quantized_note_data = quantize(midi, track, divisions);

    let mut possible_triplets = VecDeque::new();
    if triplet {
        possible_triplets = get_triplets(&quantized_note_data);
    }

    let mut complete_beat_grid = Vec::new();
    for (mut beat_grid, _) in quantized_note_data {
        complete_beat_grid.append(&mut beat_grid);
    }

    let mut notes = Vec::new();
    let mut beat_count = 0;
    let mut i = 0;
    let mut length = 0;
    let mut cur_note: &Vec<(u8, u8)> = &Vec::new();
    while i < complete_beat_grid.len() {
        if i % divisions as usize == 0 {
            beat_count += 1;
            if possible_triplets.len() != 0 && possible_triplets[0] == beat_count {
                let x = i + divisions as usize;
                let beat_data = &Vec::from(&complete_beat_grid[i..x]);
                notes.push(gen_triplet(beat_data, beat_type));
                possible_triplets.pop_front();
                i += divisions as usize;
                length = 0;
                continue;
            }
        }
        if complete_beat_grid[i].len() != 0 {
            if length != 0 {
                let beat_length = length as f32 / divisions;
                println!("{} / {} = {}", length, divisions, beat_length);
                notes.push(gen_wrapper(cur_note, beat_length, beat_type));
            }
            length = 0;
            cur_note = &complete_beat_grid[i];
        }
        length += 1;
        i += 1;
    }

    return notes;
}

/// This function finds all the triplets in a piece of music and returns a vector containing what
/// beats they are on.
/// 
/// Precondition: the note data must have already been quantized.
fn get_triplets(quantized_note_data: &Vec<(Vec<Vec<(u8, u8)>>, u8)>) -> VecDeque<u32> {
    let mut triplets = VecDeque::new();
    for i in 0..quantized_note_data.len() {
        if is_possible_triplet(&quantized_note_data[i]) {
            triplets.push_back(i as u32 + 1);
        }
    }
    return triplets;
}

/// Determines if a group of notes can be a triplet.
/// 
/// `beat_data` is a vector of all the subdivisions of the current beat. Each element in the vector
/// is another vector containing the key and velocity of the notes that start on that subdivision.
fn is_possible_triplet(beat_data: &(Vec<Vec<(u8, u8)>>, u8)) -> bool {
    let (beat_grid, note_count) = beat_data;
    if *note_count != 3 {
        return false;
    }

    let mut beat_length: [u8; 3]= [0, 0, 0];
    let mut i = 0;
    for data_point_index in 0..3 {
        beat_length[data_point_index] += 1;
        i +=1;
        while i < beat_grid.len() && beat_grid[i].len() == 0 {
            beat_length[data_point_index] += 1;
            i += 1;
        }
    }
    beat_length.sort();

    return beat_length[2] - beat_length[0] <= 2 && beat_length[2] as usize > beat_grid.len() / 4;
}

/// This function generates a note wrapper for a triplet. The `duration` for the note will be
/// the appropriate dupal counterpart. For example, eight note triplets will be stored as eigth 
/// notes in a triplet wrapper.
fn gen_triplet(beat_data: &Vec<Vec<(u8, u8)>>, beat_type: u8) -> NoteWrapper {
    let mut triplet = Vec::new();
    for div in beat_data {
        if div.len() > 0 {
            triplet.push(gen_wrapper(div, 0.5, beat_type));
        }
    }
    return NoteWrapper::ModifiedNote(NoteModifier::Triplet(triplet));
}

/// This function generates a note wrapper for a given note or set of notes.
/// 
/// If `cur_note` as a length of 1, the NoteWrapper is that of a single note. Otherwize, a chord is
/// generated made up of all the entries in `cur_note`.
/// 
/// `cur_note.len()` must be greater than 0.
fn gen_wrapper(cur_note: &Vec<(u8, u8)>, beat_length: f32, beat_type: u8) -> NoteWrapper {
    let mut chord = Vec::new();
    for note_data in cur_note {
        let value = note_data.0;
        let velocity = note_data.1;
        if value != 255 { 
            chord.push(parse_note_data((value, velocity), beat_length, beat_type));
        }
    }
    if chord.len() == 0 {
        let duration = DurationType::beat_type_map(beat_length, beat_type);
        return NoteWrapper::build_note_wrapper(255, duration, 0);
    } else if chord.len() == 1 {
        return chord[0].clone();
    }
    return NoteWrapper::ModifiedNote(NoteModifier::Chord(chord));
} 

/// A helper function for building a `NoteWrapper`.
fn parse_note_data((value, velocity): (u8, u8), beat_length: f32, beat_type: u8) -> NoteWrapper {
    let duration = DurationType::beat_type_map(beat_length, beat_type);
    if duration.duration == NoteDuration::NaN {
        return NoteWrapper::ModifiedNote(get_tied_note((value, beat_length, velocity), beat_type));
    } else {
        return NoteWrapper::build_note_wrapper(value, duration, velocity);
    }
}

/// This snaps all of the notes found in `track` to a grid. 
/// 
/// The function returns a vector of tuplets (representing beats) made up of a vector and a number. 
/// The vector in the tuplet represents the grid of subdivisions for each beat and the number shows
/// how many unique onsets are in that beat.
fn quantize(
    midi: &Midi, 
    track: &Vec<midly::TrackEvent>, 
    divisions: f32
) -> Vec<(Vec<Vec<(u8, u8)>>, u8)> {
    let mut notes = Vec::new();

    let mut ticks_per_beat = midi.ticks_per_beat;
    let mut scalar = 1;
    if midi.ticks_per_beat % 12.0 != 0.0 {
        scalar = 12;
        ticks_per_beat *= 12.0;
    }

    let mut flag = true;
    let mut raw_note_data = get_raw_note_data(track, ticks_per_beat, scalar);
    if raw_note_data.len() == 0 {
        return Vec::new();
    }

    let mut cur_beat = ticks_per_beat as u32;
    let mut note = raw_note_data.pop_front().unwrap();
    while flag {
        let mut beat_container = vec![Vec::new(); divisions as usize];
        let mut note_count = 0;
        while note.onset < cur_beat {
            let onset = note.onset - (cur_beat - ticks_per_beat as u32);
            let position = (onset as f32 * (1.0 / ticks_per_beat) * divisions).floor() as usize;
            beat_container[position].push((note.key, note.vel));
            note_count += 1;
            if raw_note_data.is_empty() {
                flag = false;
                break;
            }
            note = raw_note_data.pop_front().unwrap();
        }
        cur_beat += ticks_per_beat as u32;
        notes.push((beat_container, note_count));
    }

    if notes[0].0[0].len() == 0 {
        notes[0].0[0].push((255, 0));
        notes[0].1 += 1;
    }

    return notes;
}

/// Gets the raw note data in a midi track.
fn get_raw_note_data(
    track: &Vec<midly::TrackEvent>, 
    ticks_per_beat: f32, 
    scalar: u32
) -> VecDeque<RawNoteData> {
    let mut cur_time: u32 = 0;
    let mut cur_velocity: u8 = 0;
    let mut note_on_time: u32 = 0;
    let mut note_off_time: u32 = 0;
    let mut data: VecDeque<RawNoteData> = VecDeque::new();

    for event in track {
        let delta_t: u32 = event.delta.into();
        cur_time += delta_t * scalar;

        if let midly::TrackEventKind::Midi { channel: _, message } = event.kind {
            if let midly::MidiMessage::NoteOn {key: _, vel } = message {
                cur_velocity = vel.into();
                note_on_time = cur_time;
                if note_on_time - note_off_time >= (ticks_per_beat *  0.125).ceil() as u32 {
                    data.push_back(RawNoteData {
                        key: 255,
                        onset: note_off_time,
                        vel: 0,
                    });
                }
            }
            else if let midly::MidiMessage::NoteOff { key , vel: _ } = message {
                data.push_back(RawNoteData {
                    key: key.into(),
                    onset: note_on_time,
                    vel: cur_velocity,
                });
                note_off_time = cur_time;
            }
        }
    }

    return data;
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