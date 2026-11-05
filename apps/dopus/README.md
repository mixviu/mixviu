# Dopus three-way lab (in progress)

This checkpoint compares the retained original Rust application with a Mix
application and a new Rust comparator using the same reusable Viu widgets.
The complete source/command inventory is the feature oracle; incomplete work
must not be described as a full port. Existing evaluation demos remain intact.

`rust-core` is the renderer-free original Dopus application model, imported
for the explicitly requested Rust comparator. It is never used to implement
Mix application policy. Reusable chooser and thumbnail closures live
under libraries; provenance/dopus-native.json pins their source and hashes.
The original installed application is solely a named comparison reference.

No production promotion or replacement follows from this lab.
