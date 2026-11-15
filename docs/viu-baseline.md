# Viu baseline

Run the collector on the same Linux node as its targets, as a user permitted
to read their `/proc` files. It requires Mix and `getconf`; `readelf` is
optional for ELF build IDs. No Rust rebuild is needed for process measurements.

```text
mix tools/bench/viu_baseline.mix --output baseline.json 1234 5678
mix tools/bench/viu_baseline.mix --seconds 60 --output baseline.json gallery
mix tools/bench/viu_baseline.mix --seconds 2 --workload scroll --node 42 --steps 100 --output scroll.json gallery
```

The default window is 60 seconds. Service names resolve through the local
Bus's read-only `noded.list` inventory, including its reported source SHA.
Numeric PIDs need no Bus; when local discovery succeeds they also use the
first registered service for that PID to request frame stats. JSON contains before/after raw counters and monotonic
timestamps; stdout contains a Markdown summary. Output paths are caller-owned;
take care before replacing an existing file. Captures can contain local paths
and service names: review them before publishing.

`idle` sends no input. `scroll` first waits the idle window, then finds the
explicit AccessKit node in `ui.tree`, scrolls its rectangle centre through
`ui.scroll`, and reads stats afterwards. Scroll deliberately changes the
application's scroll position; the operator must select a suitable fixture.
The target table's row count belongs to the fixture, not the collector.
Use a 2,000-row table for comparable table runs. Definitions live in
`tools/bench/workloads.conf.mix`, parsed as strict data. `--workloads FILE`
adds named variants of the supported idle and scroll primitives without
changing the tool. The JSON records the effective definition, including a
`--steps` override. The measured interval includes the workload and its Bus
round trips, so compare equal definitions and equal fixtures.

| Field | Meaning |
|---|---|
| `memory_kib.Rss` | Resident process mappings, including shared pages |
| `Pss` | Shared resident pages apportioned between their users |
| `Private_Dirty` | Dirty pages private to this process |
| `Swap` | Swapped mappings, as reported by smaps_rollup |
| `cpu_seconds` | Delta of stat utime + stime divided by CLK_TCK |
| `cpu_percent_one_core` | CPU seconds / actual monotonic interval × 100; can exceed 100 |
| `context_switches_per_second` | Sum of per-thread voluntary/involuntary counter deltas / interval |
| `thread_context_switches_per_second` | The same rates for each TID |
| `status.Threads` | Kernel thread count at each sample |
| `executable_bytes` | Executable file's logical byte length, excluding libraries and assets |
| `build_id`, `git_sha` | ELF build ID when readelf is available; registry-reported source SHA for service targets |

Reads fail on missing counters, denied permissions, vanished processes or
threads, changed thread sets, or process start-time changes. No zero is
invented for an unreadable target. On a multithreaded process, the leader's
status counters alone would undercount switches; the summary uses all threads.
Counters are sampled sequentially, not atomically. Thread churn requires a
retry after the process settles. Context switches are a wake-up proxy, not
a count of hardware interrupts, timer expirations or scheduler wake events.

## ui.stats

`<app>.ui.stats` is a read-only drive command, advertised beside the other
`ui.*` commands. It returns `available:false` before any observed frame or
when instrumentation is unsupported. A failed Bus call is represented as
`available:false` with its return code. Undiscovered PID targets record frame
statistics as unavailable; service targets are preferred when a process owns
several Bus services.

For observed surfaces it returns cumulative `frames_drawn`, `sample_count`,
`cpu_frame_ms` (`median`, `p95`, `max`), `last_frame_ms`, `surface_pixels`,
`scale`, `backend`, and `adapter`. The ring retains the last 256 completed
frames; the median averages the middle pair for even sample counts, and p95
uses nearest rank. These describe that ring, not the difference
between two reads. Subtract before/after `frames_drawn` for interval counts.
Reads preserve the ring and counts. Viewports have separate storage.

App windows use eframe's existing CPU-side frame measurement (elapsed host
work minus measured present waits); layer surfaces measure layout through
successful submission/presentation. These are CPU-side elapsed durations,
not per-thread CPU accounting, GPU timestamps, or compositor frame times.
Skipped paints are excluded. Wgpu backend and adapter names come from the
adapter actually selected at runtime. Glow and browser hosts do not currently
publish these statistics. Idle time is represented by `last_frame_ms`, not
by adding fake zero-duration frames. Recording and reading arm no timer and
request no repaint; Bus delivery itself still needs a logic wake.
Native Viu hosts answer root/layer stats directly from registered contexts,
without waking the UI owner. Multiple layer outputs return a `surfaces` map
keyed by output name; `{output:"name"}` selects one. Removed outputs disappear
from that registry. Other apps using the drive library answer through their
existing logic wake. Neither route schedules a frame.

This cannot see GPU memory, input-to-photon latency, compositor/scanout delay,
shared-library disk size, or total system energy. Software rendering under
the CBC harness validates the path; it is not a desktop performance baseline.

## Headless checks

On a build worker, build `viu`, `viud`, and `mixd`, then run
`mix tools/bench/proc_test.mix` and `mix tools/bench/headless_test.mix`.
`MIX_TEST_BIN` can select another binary directory. The smoke test uses the
chooser test's `tests/desktop/nested_host_lib.mix` interface: an owned broker,
headless Weston, nested viud, and a real gallery window entirely inside that
private headless session. It retains JSON and logs in its printed run directory.
The missing harness interface is restored as a bounded native subset, informed
by the earlier MIT/Apache-2.0 nested-host harness; it uses this repository's
current private broker helper. No host display, VT or desktop session is used.
