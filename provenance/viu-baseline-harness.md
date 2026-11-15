# Baseline headless harness

`tests/desktop/nested_host_lib.mix` adapts the bounded start/stop/readiness
interface of `tests/desktop/nested_host_lib.mix` from `markc/mixos`, revision
`884fdd8ef2c548cc42017b9170593ec67ed30e57`.

Source file SHA-256:
`af373c0b767ef73048bb29276277d5092836d0423325cb801ace49ad4ea7a317`.

The source is MIT OR Apache-2.0, copyright 2026 Mark Constable; its notice
is retained. This adaptation brings only private headless Weston and nested
viud lifecycle helpers. It uses this repository's current private broker
helper, current environment names and strict compositor settings. It keeps
run evidence, omits the historical compatibility/mesh/probe machinery, and
never connects to a live display. The original source repository is not a
runtime dependency.
