Optional desktop families, loaded at runtime from the installation's
`share/fonts`. These are data, not linked code. No font discovery or system
font installation is needed. See `provenance/deck-fonts.json` for source
versions and file hashes; the adjacent OFL texts cover redistribution.

Select `Liberation Sans`, `Noto Sans Mono` or `Noto Sans` using a design
typography role or scene text `font` and `weight`. Apps can use
`toolkit::fonts::optional::family` to select a named egui family for a weight.
CSS Fonts 4 weight matching searches the bundled faces: sans 400/500 uses
Regular and 600/700 Bold; mono 400 uses Regular, 500 Medium, 600/700 Bold.
Noto Sans Regular is the required glyph fallback (not observed in the current
17-slide PDF union). Content settings keep their existing family choices.
Inter and JetBrains Mono stay the desktop defaults. These files are copied
into every gold apps bundle and covered by its SHA-256 manifest.
