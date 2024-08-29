# beatblox_midi
This crate helps with parsing data from .mid files and returns a format compatible with netsblox.

[crate.io page](https://crates.io/search?q=beatblox_midi)

## Version Info
0.3.0

Is able to parse both dupal and triple beats and can parse to different precisions.

## Update Log

0.3.0 - Triplet parsing now works. Sixtyfourth duration was removed. Only eight note triplets can be
handled.

0.2.2 - Parsing now works with "fuzzy" data.

0.2.1 - Optimizations to parsing and better beat accuracy.

## TODO

- [x] Note velocity detection
- [x] Chord detection
- [x] Precision param in Midi::parse
- [x] Tuplet detection
- [ ] Quarter / sixteenth note triplet detection
- [ ] Handle time signature changes
