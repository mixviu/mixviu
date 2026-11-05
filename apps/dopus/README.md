# Dopus Mix evaluation

Twin-pane Mix scene adapted from the earlier Dopus comparison prototype.
`dopus.mix` owns application state and behaviour, `core.mix` owns file and
selection operations, and `scene.mix` defines the widgets. The Viu client
renders it in a normal Wayland window, with embedded Mix on its worker thread.
compd supplies the same native decorations as the Quoin audio example.
The later native-model/table optimisation branches are not included.

Run in a MixOS graphical session with a local scene host:

```
/opt/mixos/bin/viu-dopus-client /path/to/apps/dopus
```

`DOPUS_DEMO_ROOT` optionally sets the demo root; the default is
`$XDG_DATA_HOME/mixviu/dopus-demo` (or the corresponding home data directory).
The first run creates example folders/files. Later runs preserve changes.
`open.mix` launches another client window. `client.json` declares the behaviour
entry point, scene/citizen, title, app ID, size and absence of an audio backend.
The generic client defaults still run the Quoin fixture when no manifest exists.

Select a row to activate its pane. Use Open folder, Parent, Back and
Expand/collapse to navigate. Sort by name, size or modification time; toggle
hidden files per pane. Type a name before New folder or Rename. Copy/Move
target the other pane and refuse existing destinations. Failures appear in
the status line. Paths remain confined to the demo root.

This is a small-folder, single-selection UI evaluation: listings refuse more
than 200 flattened rows. It has no native virtual table, drag-and-drop, file
preview, delete or host-home integration. Its Mix core retains selection
range support for command tests; the basic list UI does not claim keyboard/
modifier parity with the full Rust Dopus product. Multi-file operations are
synchronous and may partially complete on a later per-file failure.

Check the retained filesystem oracle with `mix core-test.mix`; it creates a
unique temporary directory and exercises navigation, selections, real copy/
move/rename, overwrite refusal and root confinement. Source origin/adaptations
and original hashes are recorded in `provenance/dopus-prototype.json`.
