# Sourcey documentation for itertools

This directory contains a reproducible Sourcey-generated Rust API site for
`itertools` pinned to upstream commit
`cef9a8203ec485b06ec1889550cde0aa989d103a`.

## Why this belongs here

The generated site keeps API pages, source links, navigation, and doctest views
inside the project repository. It is an optional documentation artifact and does
not change the crate, its MSRV, or the existing docs.rs publication path.

## Rebuild

Requirements: Node.js 20+, npm, rustup, and Rust toolchain
`nightly-2025-11-22` (the rustdoc v57 toolchain pinned by Sourcey 3.6.5).

```bash
cd docs/sourcey-itertools
npm ci
npm run snapshot
npm run build
```

The snapshot command converts nightly rustdoc JSON into Sourcey's stable snapshot
format. The build command reads only the committed snapshot and writes the static
site to `site/`, so a docs host does not need Rust installed.

`adapter/Cargo.toml` is a docs-only manifest that points at the upstream
`src/lib.rs` and mirrors the crate's public feature flags. It intentionally omits
dev dependencies so Cargo cannot confuse `itertools@0.15.0` with an older
transitive `itertools` package while generating rustdoc JSON.

## Pinning and provenance

- Target repository: https://github.com/rust-itertools/itertools
- Target commit: `cef9a8203ec485b06ec1889550cde0aa989d103a`
- Target license: `MIT OR Apache-2.0`
- Sourcey version: `3.6.5`
- Rust toolchain: `nightly-2025-11-22`
- Adapter: native `rustdoc()` snapshot adapter
- Snapshot: `snapshots/rustdoc.json`
- Generated output: `site/`

To refresh, update the target commit first, regenerate the snapshot with the same
pinned Sourcey version, review the generated page inventory, and commit both the
snapshot and rebuilt site together.
