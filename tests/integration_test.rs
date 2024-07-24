use beatblox_midi;
use beatblox_midi::parsing::duration::DurationType;
use beatblox_midi::parsing::duration::NoteDuration;
use beatblox_midi::parsing::duration::NoteDurationModifier;

#[test]
fn print() {
    let dir = String::from("tests/test_files/test-2.mid");
    let midi = beatblox_midi::Midi::parse_with_precision(dir, DurationType {
        duration: NoteDuration::EIGHTH,
        modifier: NoteDurationModifier::None,
    });
    midi.print();
}