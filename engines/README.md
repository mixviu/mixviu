# Engine boundary

The intended hosts are mix (shell/CLI), mixd (server host), viu (application
and shell UI runtime) and viud (compositor authority). The source for all four
belongs in this repository as it is needed and surveyed.

No engine code is seeded here yet. In particular, splitting mix into mix and
mixd is future work. Do not create empty crates or copy the old multi-call
application/service tree to make this directory look complete.

Reusable native facilities live in libraries/. Mix application bundles live
in apps/. The first implementation will use the existing compositor while
proving the shared Viu runtime and Mix behaviour boundary.
