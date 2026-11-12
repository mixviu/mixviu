# Round 2 shared-crate comparison

Frozen origin/main: `8779f33284b7d3f1178c3d57a699efb9b0867b22`.
`git ls-tree origin/main libs/inputd-core libs/inputd-schema` returns no entries.
The source crates are under `services/inputd/crates/`; comparison below uses
`git show origin/main:services/inputd/crates/<crate>/<file>` for every file.
No source working-tree file was read or changed.

| File | Comparison |
| --- | --- |
| `inputd-core/Cargo.toml` | Manifest adaptation (diff below) |
| `inputd-core/src/interceptor.rs` | Byte-identical |
| `inputd-core/src/lib.rs` | Byte-identical |
| `inputd-core/src/tests.rs` | Byte-identical |
| `inputd-schema/Cargo.toml` | Manifest adaptation (diff below) |
| `inputd-schema/src/action.rs` | Byte-identical |
| `inputd-schema/src/lib.rs` | Byte-identical |
| `inputd-schema/src/scope.rs` | Byte-identical |

All six Rust source/test files match byte for byte, including all 26 core
tests and 14 schema tests. There are no resolver, interceptor, user-key-rule,
wire-name, JSON-shape or test omissions/differences. The only manifest-file difference is
`package.metadata.mixos` becoming `package.metadata.mixviu`; dependencies,
lints and inherited package fields are identical. Workspace dependency
resolution changes source locations, not the implementation or protocol. No second implementation is imported or deleted by this
port relative to MixViu main. The old reserved-key disagreement with PR #68
remains as documented in docs/inputd/port.md.

```diff
diff --git a/frozen/inputd-core/Cargo.toml b/libraries/inputd-core/Cargo.toml
index 922d0824..2a889384 100644
--- a/frozen/inputd-core/Cargo.toml
+++ b/libraries/inputd-core/Cargo.toml
@@ -9,7 +9,7 @@ repository.workspace = true
 publish = false
 description = "inputd's pure core: keymap resolution, the user-key rule, rebind admission and press-latched key disposition"
 
-[package.metadata.mixos]
+[package.metadata.mixviu]
 component = "inputd"
 kind = "service"
 layer = "core"
```

```diff
diff --git a/frozen/inputd-schema/Cargo.toml b/libraries/inputd-schema/Cargo.toml
index fd6923c7..ce61a10b 100644
--- a/frozen/inputd-schema/Cargo.toml
+++ b/libraries/inputd-schema/Cargo.toml
@@ -9,7 +9,7 @@ repository.workspace = true
 publish = false
 description = "inputd's wire contract: keymap.v1 rows, the input.* commands, and the fixed user-key rule"
 
-[package.metadata.mixos]
+[package.metadata.mixviu]
 component = "inputd"
 kind = "service"
 layer = "core"
```
