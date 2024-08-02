use beatblox_midi::Midi;
use beatblox_midi::parsing::duration::DEFAULT_DURATION_PRECISION;
use beatblox_midi::parsing::duration::DurationType;
use beatblox_midi::parsing::duration::NoteDuration;
use beatblox_midi::parsing::duration::NoteDurationModifier;

#[ignore]
#[test]
fn parse() {
    let dir = String::from("tests/test_files/test-2.mid");
    let midi = Midi::parse(dir);
    midi.print();
}

#[ignore]
#[test]
fn parse_precision() {
    let dir = String::from("tests/test_files/test-4.mid");
    let precision = DurationType {
        duration: NoteDuration::EIGHTH,
        modifier: NoteDurationModifier::None,
    };
    let midi = Midi::parse_with_precision(dir, precision, false);
    midi.print();
}

#[test]
fn parse_tuplet() {
    let dir = String::from("tests/test_files/test-3.mid");
    let midi = Midi::parse_with_precision(dir, DEFAULT_DURATION_PRECISION, true);
    midi.print();
}
