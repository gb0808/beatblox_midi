use beatblox_midi::parsing::duration::DurationType;
use beatblox_midi::parsing::duration::NoteDuration;
use beatblox_midi::parsing::duration::NoteDurationModifier;

#[test]
fn get_beat_count_1() {
    let duration = DurationType {
        duration: NoteDuration::QUARTER,
        modifier: NoteDurationModifier::None,
    };
    let beats = duration.get_beat_count(2);
    assert_eq!(1.0, beats);
}

#[test]
fn get_beat_count_2() {
    let duration = DurationType {
        duration: NoteDuration::QUARTER,
        modifier: NoteDurationModifier::None,
    };
    let beats = duration.get_beat_count(3);
    assert_eq!(2.0, beats);
}

#[test]
fn get_beat_count_3() {
    let duration = DurationType {
        duration: NoteDuration::QUARTER,
        modifier: NoteDurationModifier::Dotted,
    };
    let beats = duration.get_beat_count(2);
    assert_eq!(1.5, beats);
}

#[test]
fn get_beat_count_4() {
    let duration = DurationType {
        duration: NoteDuration::EIGHTH,
        modifier: NoteDurationModifier::Dotted,
    };
    let beats = duration.get_beat_count(3);
    assert_eq!(1.5, beats);
}

#[test]
fn get_beat_count_5() {
    let duration = DurationType {
        duration: NoteDuration::EIGHTH,
        modifier: NoteDurationModifier::None,
    };
    let beats = duration.get_beat_count(2);
    assert_eq!(0.5, beats);
}

#[test]
fn get_beat_count_6() {
    let duration = DurationType {
        duration: NoteDuration::SIXTEENTH,
        modifier: NoteDurationModifier::None,
    };
    let beats = duration.get_beat_count(2);
    assert_eq!(0.25, beats);
}

#[test]
fn get_beat_count_7() {
    let duration = DurationType {
        duration: NoteDuration::THIRTYSECOND,
        modifier: NoteDurationModifier::None,
    };
    let beats = duration.get_beat_count(2);
    assert_eq!(0.125, beats);
}