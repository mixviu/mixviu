# Render controls and progress

`av render TIMELINE.json --out FILE.webm [--cpu N] [--mem BYTES] [--progress]`
accepts options in any order after the timeline. `--mem` accepts integer bytes
or uppercase K/M/G suffixes (1024 powers). Default memory admission is 8 GiB.
CPU must be 1 through `available_parallelism()`. Requests in the timeline are
clamped by reducing parallel chunks before per-chunk threads. A memory ceiling
reduces chunks; one chunk must fit or `encode_memory_budget` refuses the job.
Invalid values refuse with `cpu_limit` or `mem_limit`. Without limits the old
admission rules remain. The stdout receipt includes `limits` (a null CPU means
the legacy 2× available CPU admission), `encode_requested` and `encode_effective`.

`av audio TIMELINE.json --out FILE.webm [--wav FILE.wav] [--cpu N] [--mem BYTES] [--progress]`
accepts CPU and memory ceilings with the same syntax and validation as render.
Both are no-ops for audio: its work is single-threaded and it has no memory
admission ceiling. In particular, zero bytes is valid syntax but render refuses
it when one chunk cannot fit. It also accepts `--progress` and `--wav`.

With `--progress`, stderr carries flushed `AVP ` lines alongside diagnostics:

```text
AVP {"v":1,"t":0.001,"stage":"input","event":"start"}
AVP {"v":1,"t":0.502,"stage":"video","event":"progress","frames_out":15,"frames_encoded":2,"frames_total":120}
AVP {"v":1,"t":1.234,"stage":"done","event":"end","ok":true,"receipt":"stdout","frames_out":120,"frames_encoded":8,"frames_total":120}
```

`t` is monotonic seconds since command start. Stages first appear as input,
audio, video (render only), mux, done. Video and mux overlap because batches
are muxed as they finish. Video end marks the last batch joining, before its
mux accounting. Workers only increment atomic frame counters. A dedicated
writer samples them roughly every 500 ms and writes all progress stderr lines.
Start/end events use a bounded 64-event queue with non-blocking sends; a full
queue drops events, counted as `events_dropped` on done. Video frame counts are totals across
all chunks; `frames_out` counts composed output frames and `frames_encoded`
counts encoder packets, excluding repeat packets. Chunk start/end events carry
zero-based `chunk` and `chunks_total`. Audio boundaries include `audio_pass`
(codec correction pass, starting at zero); mux end includes output `bytes`.
The final result has a separate reserved slot. With a responsive stderr reader,
the last line is done/end, with `ok` true only for exit status zero.
Completion waits at most one second for the writer to flush it, then silently
gives up. A blocked stderr reader delays only the writer, never encoding or
command completion beyond that final timeout. Under backpressure, boundaries
and the done line may therefore be absent; surviving events retain queue order
and counts remain monotonic.
`receipt` names stdout, the existing receipt destination; callers saving it
choose the actual file path. On error stdout may have no receipt. No new receipt
file is created. Progress does not change output bytes or receipt fields.
