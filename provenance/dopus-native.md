# Dopus comparison source closure

The original reference is MixOS at
8779f33284b7d3f1178c3d57a699efb9b0867b22, matching the retained reference
application's build stamp. The reference application remains installed and
unchanged; it is never passed off as a new Viu engine.

`dopus-native.json` records the immutable imported bytes and destinations.
The original renderer-free Dopus core belongs to the explicitly requested
Rust comparator. The Mix application independently owns its navigation,
selection and filesystem behaviour. It does not call that core as its policy.

The minimal `files` closure consists of chooser data contracts and their tests;
corpus, database, indexing and transport modules are excluded. The initial
atomic/error candidates were removed after closure inspection: their unused
streaming-transfer API required additional BLAKE3 machinery. Dopus config now
uses the existing tested config::atomic::replace primitive, with its stronger
directory/symlink checks. Original import hashes remain recorded as removed;
the reference source remains preserved. Its manifest declares only required
dependencies. The native thumbnail library retains all
of its coherent worker/cache/discovery/PNG modules and tests. Manifest paths,
workspace versions and explicit pins are adapted to this repository.

All Rust source retains its SPDX licence and copyright. The root MIT and
Apache-2.0 licences cover these imports. MixOS is a named source library;
historical comments in imports do not set MixViu strategy. Imported worker
and filesystem limitations must be measured and reported in the comparison.
