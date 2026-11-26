# Render controls and progress

`av render TIMELINE.json --out FILE.webm [--cpu N] [--mem BYTES] [--progress]`
accepts options in any order after the timeline. Repeated `--progress` flags are
accepted by both commands; a token in an option's value position is kept as a
value, even when it is `--progress`. `--mem` accepts integer bytes
or uppercase K/M/G suffixes (1024 powers). Default memory admission is 8 GiB.
CPU must be 1 through `available_parallelism()`. Requests in the timeline are
clamped by reducing parallel chunks before per-chunk threads. Audio reserves one
CPU slot, leaving N−1 for video; `--cpu 1` runs audio then video serially.
A memory ceiling reserves the conservative audio peak alongside the video peak
and reduces chunks; audio plus one chunk must fit or `encode_memory_budget` refuses the job.
Invalid values refuse with `cpu_limit` or `mem_limit`. Without a CPU ceiling,
video retains its legacy 2× available CPU admission alongside one audio thread.
The stdout receipt includes `limits` (a null CPU means no caller CPU ceiling),
`encode_requested` and `encode_effective`.

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
audio, video (render only), mux, done. Audio and video events may interleave:
mastering runs alongside the full video encode. Encoded video packets are
spooled to a temporary file, keeping only one batch in memory. Both stages join
before mux starts. On either error the other stage runs to completion and joins;
no output is published. The spool is removed on success and failure.
Workers only increment atomic frame counters. A dedicated
writer samples them roughly every 500 ms and writes all progress stderr lines.
Start/end events use a bounded 64-event queue with non-blocking sends; a full
queue drops events, counted as `events_dropped` on done. Video frame counts are totals across
all chunks; `frames_out` counts composed output frames and `frames_encoded`
counts encoder packets, excluding repeat packets. Chunk start/end events carry
zero-based `chunk` and `chunks_total`. Audio boundaries include `audio_pass`
(codec correction pass, starting at zero); mux end includes output `bytes`.
The final result has a separate reserved slot. With a responsive stderr reader,
the last AVP line is done/end, with `ok` true only for exit status zero.
Completion waits at most one second for the writer to flush it, then silently
gives up. On success, a blocked stderr reader delays only the writer, never
encoding or command completion beyond that final timeout. On failure, existing
`av:` diagnostics still write directly to fd 2 and can block completion;
the one-second bound applies only to the progress writer. Under backpressure, boundaries
and the done line may therefore be absent; surviving events retain queue order
and counts remain monotonic.
`receipt` names stdout, the existing receipt destination; callers saving it
choose the actual file path. On error stdout may have no receipt. No new receipt
file is created. Progress does not change output bytes or receipt fields.

The receipt's `stages` reports input, audio, video and mux `wall_s` and `cpu_s`.
Video wall time includes writing the packet spool; mux includes reading it.
Overlapping wall times must not be added to estimate elapsed time. Audio CPU is
thread CPU; video CPU is joined process CPU minus input and audio CPU, including
encoder worker CPU and orchestration. `audio.passes` totals all limiter attempts;
`audio.codec_passes` counts corrections, `audio.codec_attempts` includes the initial
encode/decode/meter cycle. `audio.mastering_cost` has each attempt's limiter count,
per-limiter CPU times, mastering CPU/wall seconds and codec CPU/wall seconds.
Each video chunk uses one codec pass (no two-pass video encode).
Stage entries also expose `audio.limiter_passes`, `audio.codec_attempts`,
`video.codec_passes_per_chunk` and `video.codec_attempts` (the chunk count).
A mastering-refusal receipt retains its existing top-level audio fields and
adds these stage entries, with zero mux cost because mux did not run.

Memory admission includes twelve stereo f64 programme buffers, envelope and
packet/index allowance (224 bytes/output frame), three buffers for the largest
native source window and 32 MiB scratch, plus every concurrent video chunk's
existing estimate. This conservative estimate also applies with `--cpu 1`.
It is admission accounting, not an OS-enforced RSS limit. Packet spooling uses
temporary disk space alongside the output; disk errors fail without publishing.
