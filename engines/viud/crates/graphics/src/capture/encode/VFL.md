# Video frame log, version 1

VFL is an intermediate lossless capture, not a delivery codec. A take is a
directory named `video.vfl`. During capture the Bus recorder uses a private
sibling `video.vfl.<random>.part`; successful stop publishes it with a single
no-replace directory rename. Interrupted and failed takes retain the staging
directory for recovery. Explicit discard removes only that take's directory.
Directories are 0700; every file is created 0600 with exclusive creation and
`libc::O_NOFOLLOW`. Existing destinations, including dangling symlinks, fail.

## Metadata

`manifest.json` is immutable UTF-8 JSON, written and synced before frames:
`{version:1,width,height,fps,format:"bgra8",encoder:"vfl-lz4",complete:false,key_interval}`.
Dimensions are positive; width × height × 4 is at most 256 MiB. FPS is an
integer 1–60. Pixels are row-major BGRA bytes, top to bottom, with no stride.
Source RGB is converted on the writer thread; padding pixels are zero.

`origin.json` is written before the first record:
`{t0_us,clock:"monotonic"}`. The origin is the compositor's first capture
timestamp, in microseconds. `frames.jsonl` has `{frame,t_us}` per output slot;
its times are the exact constant-rate timeline `floor(frame*1000000/fps)`.
`markers.jsonl` has `{seq,label,data,t_us,frame}`: marker times are compositor
monotonic timestamps relative to the origin, and frame is the last scheduled
slot. Sequence numbers start at zero. A marker acknowledgement means queued;
durability is guaranteed only by successful stop. A full queue refuses a
marker explicitly; it does not silently discard it.

`complete.json` is written and synced last:
`{version:1,t0_us,frames,dropped,chunks,complete:true}`. It completes the
manifest without overwriting its recoverable initial copy. Its frame count
includes repeats replacing dropped slots. `dropped` includes queue overruns
and compositor readback-budget skips, without counting a slot twice.

## Chunks and records

Chunks are `chunk-000000.bin`, `chunk-000001.bin`, etc., in numeric order.
There is no file header. Records never cross chunks. The writer rotates before
a record would exceed 64 MiB; a single larger record occupies its own chunk.
All integers are unsigned little endian. Each record is:

| Offset | Bytes | Meaning |
|---|---:|---|
| 0 | 4 | Payload length, excluding the 17-byte header |
| 4 | 4 | IEEE CRC32 of frame number bytes, kind byte, and payload |
| 8 | 8 | Frame slot, starting at zero and increasing by one |
| 16 | 1 | Kind: 0 key, 1 delta, 2 repeat |
| 17 | length | Payload |

Key payloads are raw LZ4 **blocks**, without an LZ4 frame header or prepended
size. Decompress to exactly width × height × 4 bytes. Delta payloads decompress
to the same size, then XOR byte-for-byte with the previous decoded frame.
Repeat has length zero and reuses the previous pixels. The first record must
be key. Every `fps*10` slots is key, including unchanged images; all other
identical images are repeat. A missing submitted slot repeats the last image;
before any accepted image, missing slots use a zero-filled frame.

Each sealed chunk has a `chunk-NNNNNN.json` receipt:
`{bytes,crc32,next_frame}`. CRC covers **every byte** of the binary chunk,
including headers. `next_frame` is the next slot after its last record.
Chunks are synced before their receipts. Recovery can read a chunk without
its receipt using the per-record CRCs.

## Reader checks and recovery

Reject unknown versions, encoders, formats, invalid dimensions/FPS, unknown
kinds, non-contiguous chunks or frame numbers, a delta/repeat before key,
non-empty repeat, invalid LZ4 or decompressed size. Bound allocations from
dimensions, never from an unchecked payload length. Before decompressing,
check the complete record exists and its CRC matches. Verify each available
chunk receipt's length, CRC and next slot. Complete takes must also match
`complete.json` counts and contain both metadata files and sidecars.

Missing/invalid `complete.json` means interrupted, never a successful take.
Scan complete CRC-valid records in order and stop at the first torn or corrupt
record; never skip it and resume a delta chain. Earlier complete records remain
usable, including those in the unsealed last chunk. A missing origin prevents
wall-clock marker alignment, but does not prevent pixel/timeline recovery.
Sidecars are advisory during recovery and may have a torn or missing tail;
binary records determine recovered frame count. Interrupted drop totals are
unknown, not zero. Do not trust a completion file until all checks pass.

Incompatible layouts require a new version. Unknown JSON fields may be
ignored. Shared constants live in `libraries/media/src/lib.rs`; R2 owns the
production reader and verifier. No reader is shipped by this writer change.

## Scheduling

One writer thread owns conversion, compression and all frame/marker disk I/O.
The channel holds eight messages, with frame admission limited to four
occupied slots so markers have reserved capacity; admission uses
`try_send`. A full queue increments the dropped counter and releases that
frame allocation. The writer fills slot gaps and the dropped tail with repeats,
preserving CFR without retaining an unbounded frame backlog. The compositor
keeps the latest source `Arc` so a subsequent tick can retry changed pixels.
Stop drains and syncs on a finaliser thread. Disk errors fail the take and
retain the staged directory. Shutdown does not wait for disk on encoder drop.
