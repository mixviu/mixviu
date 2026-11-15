# Video frame log, versions 1 and 2

The current writer emits version 2. Version 1 recordings use the same layout
but permit kinds 0–2 only. Version 2 adds kind 3 rectangle XOR deltas. Readers
must select supported kinds by version; a version-1 reader must reject version
2, not interpret its new payload as a whole-frame delta.

VFL is an intermediate lossless capture, not a delivery codec. A take is a
directory named `video.vfl`. During capture the Bus recorder uses a private
sibling `video.vfl.<random>.part`; successful stop publishes it with a single
no-replace directory rename. Interrupted and failed takes retain the staging
directory for recovery. Explicit discard removes only that take's directory.
The start reply names the staging directory as `recovery_path`; failed stop
replies retain that field so callers can locate an interrupted take.
Directories are 0700; every file is created 0600 with exclusive creation and
`libc::O_NOFOLLOW`. Existing destinations, including dangling symlinks, fail.

## Metadata

`manifest.json` is immutable UTF-8 JSON, written and synced before frames:
`{version:2,width,height,fps,format:"bgra8",encoder:"vfl-lz4",complete:false,key_interval}`.
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
`{version:2,t0_us,frames,dropped,chunks,complete:true,error:null,writer:{...}}`. It completes the
manifest without overwriting its recoverable initial copy. Its frame count
includes repeats replacing dropped slots. `dropped` counts sampled changed
source updates that were superseded before capture (or abandoned at stop),
plus newly captured changed images refused by the writer queue. Deferring
the same change until a later tick is not itself a drop. An unchanged tick
is a repeat even if its queue message is omitted and healed from slot numbers.
This is not a count of all client commits between sampling ticks or a claim
to know the pixels of an uncaptured source state.
Legacy version-1 takes from the previous recorder used an aggregate of
readback-budget skipped ticks and queue rejection slots for `dropped`; do not
interpret that historical aggregate as version-2 source-update losses.

A gracefully failed or abandoned take drains and seals its complete records
but writes `interrupted.json` instead of `complete.json`. It has the same
counts and origin, `complete:false`, and `error` (a string for a known capture
failure, otherwise null). This is recovery evidence, never a success receipt.

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
| 16 | 1 | Kind: 0 key, 1 whole-frame delta, 2 repeat, 3 rectangle delta (v2) |
| 17 | length | Payload |

CRC32 is the reflected IEEE polynomial 0xEDB88320, initial/final XOR
0xFFFFFFFF (the CRC of ASCII `123456789` is 0xCBF43926). Readers must bound a
compressed payload length to `raw_bytes + raw_bytes/255 + 32` before allocation.

Key payloads are raw LZ4 **blocks**, without an LZ4 frame header or prepended
size. Decompress to exactly width × height × 4 bytes. Delta payloads decompress
to the same size, then XOR byte-for-byte with the previous decoded frame.
Repeat has length zero and reuses the previous pixels. The first record must
be key. Every `fps*10` slots is key, including unchanged images; all other
identical images are repeat. A missing submitted slot repeats the last image;
before any accepted image, missing slots use a zero-filled frame.

Kind 3's payload begins with four little-endian u32 pixel coordinates:
`x, y, width, height` (16 bytes), followed by one raw LZ4 block. Decompress
that block to exactly `width*height*4` bytes of tightly packed, top-down BGRA
XOR pixels, then XOR those rows into the previous frame at `(x,y)`. Pixels
outside that rectangle remain unchanged. The rectangle is non-empty and
must fit inside the canvas, with checked coordinate arithmetic. Length must
be at least 16 before reading coordinates. The normal record CRC covers
both coordinates and compressed data.

The writer derives the bounding rectangle from the actual XOR pixels, rather
than trusting compositor damage. It chooses kind 3 when that rectangle covers
less than three quarters of the canvas; large or dispersed changes retain
kind 1. Accepted identical frames remain kind 2. Compositor damage rectangles
are not an input to this format or an assumption a reader may make.

Each sealed chunk has a `chunk-NNNNNN.json` receipt:
`{bytes,crc32,next_frame}`. CRC covers **every byte** of the binary chunk,
including headers. `next_frame` is the next slot after its last record.
Chunks are synced before their receipts. Recovery can read a chunk without
its receipt using the per-record CRCs.
Sealing also syncs the take directory. Setup syncs the take directory and its
parent, and the origin entry is synced before the first record, so sealed
chunks remain discoverable after power loss before finalisation.

## Reader checks and recovery

Reject unknown versions, encoders, formats, invalid dimensions/FPS, unknown
kinds (including kind 3 in version 1), non-contiguous chunks or frame numbers,
any delta/repeat before key,
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
ignored. The writer core and constants live in `libraries/media/vfl/src/lib.rs`;
this spec belongs to that same crate. R2 owns the
production reader and verifier. No reader is shipped by this writer change.

## Scheduling

viud's adapter has one writer thread; the shared core owns conversion,
compression and all frame/marker file I/O, without owning any thread or Bus.
The channel holds eight messages, with frame admission limited to four
occupied slots so markers have reserved capacity; admission uses
`try_send`. A full queue increments `queue_overruns`; only a newly captured
changed image increments `dropped_queue`. The writer fills slot gaps and the dropped tail with repeats,
preserving CFR without retaining an unbounded frame backlog. The compositor
keeps the latest source `Arc` so a subsequent tick can retry changed pixels.
Stop drains and syncs on a finaliser thread. Disk errors fail the take and
retain the staged directory. Shutdown does not wait for disk on encoder drop.

Status/stop distinguish `dropped_capture`, `dropped_queue`, `readback_deferred`
and `pending_ticks`; `dropped` is the sum of the first two. The `writer` object
reports queue overruns/wait total/max, conversion time, XOR/comparison/LZ4
total/mean/max time, and file-write total/mean/max time (including CRC/JSON
append work). `setup_us` covers initial file setup, `sync_us`/`sync_max_us`
cover chunk sealing; sealing time is also part of a write that rotates a chunk.
Timing fields are microseconds, accumulated on the worker, not disk work on
the compositor. Record counts and key/delta/rectangle payload byte totals
(excluding 17-byte record headers) show the resulting storage cost. Fatal
writer errors fail the take explicitly; a missing completion receipt never
promises that its queued tail was stored.

Active output sampling follows that output's submitted scene changes, excluding
capture and backend continuation requests. Empty renders caused by another
output do not advance it. Paused outputs have no submitted-frame clock and use
the conservative scene-request clock instead, with capture requests excluded;
unrelated changes on another output can still cause an extra paused readback.
Switching between those clocks forces a fresh sample. Neither clock is a pixel
hash; the writer determines repeats from the actual accepted pixels.
