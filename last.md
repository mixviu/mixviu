# Desk4 integration close-out

First set: **ACCEPTED**, tested source `9643f9235819d33fc8e398ea0abfcc29d84192cf`. Four release engines on CBC2 with SHA256SUMS; full test stages, Monitor release revalidation, P2 furniture, Track A and direct Dopus/Email 2,000-row comparison pass. Exact hashes, numerical results and CBC records: [desk4-integration.md](docs/ports/desk4-integration.md).

Ced d4e19db is merged at `4f2c0fd518336e2b2bb74aac9c3305fb9a2be8f0`. CBC3 is building that source; Ced release-byte suite and Monitor quick check remain pending. No install, VT or desktop work.

First set cbc2:/build/es-artifacts/9643f9235819d33fc8e398ea0abfcc29d84192cf/SHA256SUMS:

```text
2dd3b73fb913c32c9106275e63c5a4455966824a5bb81573e6f639d5abe43ca3  mix
4795719e5b8ff1f8a53e37c431349ad462b85ab7afb26e53f8f4f2757f2ee138  mixd
f32cc6cfb3162955f67a9938a56dcc9c564ee699be861e55e26c4f1db602e205  viud
f7c4f97ca82c989340ba7060ad1130aa164de3b0d986dc9ae7680db8d7796a7b  viu
```

Branch lab/gold-parity; tested integration head 4f2c0fd518336e2b2bb74aac9c3305fb9a2be8f0 (Ced validation pending; receipt HEAD is the commit containing this file); accepted first artifacts cbc2:/build/es-artifacts/9643f9235819d33fc8e398ea0abfcc29d84192cf/ with hashes above, SHA256SUMS 2becc4885ba92e1e2ec9164a664911fc01c6f0ea5fa31eff269c64959e4a8869; first gates PASS; CBC 20261107-064343-f05d5cfb-a1, 20261107-075258-c6071acd-a1, 20261107-075901-fe888f44-a1; final build 20261107-080318-8bce8633 pending.
