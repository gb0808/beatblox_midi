use beatblox_midi::parsing::duration::DurationType;
use beatblox_midi::parsing::duration::NoteDuration;
use beatblox_midi::parsing::duration::NoteDurationModifier;

#[test]
fn beat_type_map_1() {
    let control = DurationType {
        duration: NoteDuration::QUARTER,
        modifier: NoteDurationModifier::None,
    };
    let duration = DurationType::beat_type_map(1.0, 2);
    assert_eq!(control.duration, duration.duration);
    assert_eq!(control.modifier, duration.modifier);
}

#[test]
fn beat_type_map_2() {
    let control = DurationType {
        duration: NoteDuration::EIGHTH,
        modifier: NoteDurationModifier::Dotted,
    };
    let duration = DurationType::beat_type_map(0.75, 2);
    assert_eq!(control.duration, duration.duration);
    assert_eq!(control.modifier, duration.modifier);
}

#[test]
fn beat_type_map_3() {
    let control = DurationType {
        duration: NoteDuration::EIGHTH,
        modifier: NoteDurationModifier::None,
    };
    let duration = DurationType::beat_type_map(1.0, 3);
    assert_eq!(control.duration, duration.duration);
    assert_eq!(control.modifier, duration.modifier);
}

#[test]
fn beat_type_map_4() {
    let control = DurationType {
        duration: NoteDuration::HALF,
        modifier: NoteDurationModifier::None,
    };
    let duration = DurationType::beat_type_map(1.0, 1);
    assert_eq!(control.duration, duration.duration);
    assert_eq!(control.modifier, duration.modifier);
}

#[test]
fn beat_type_map_5() {
    let control = DurationType {
        duration: NoteDuration::QUARTER,
        modifier: NoteDurationModifier::None,
    };
    let duration = DurationType::beat_type_map(2.0, 3);
    assert_eq!(control.duration, duration.duration);
    assert_eq!(control.modifier, duration.modifier);
}