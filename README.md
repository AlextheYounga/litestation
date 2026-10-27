# LiteStation

LiteStation is a focused cross-platform desktop application for working with
local SQLite databases.

## Current capabilities (v1 MVP)

- Open existing local SQLite files with full read and write access; missing
  paths are never created.
- Browse tables, views, indexes, and triggers, and browse table rows in bounded
  pages. Writable connections support guarded insert, update, and delete
  operations use optimistic conflict checks to avoid overwriting stale rows.
- Edit and execute SQL with CodeMirror. Run the selected SQL, the statement at
  the cursor, or all statements sequentially. Run-all is explicitly confirmed
  and non-atomic; earlier successful statements remain applied if a later one
  fails. The UI requests at most 100 query rows; the backend enforces an
  absolute 1,000-row limit plus cell, payload, SQL-size, and execution limits.
- Import SQL dumps atomically into a writable connection (with size and safety
  checks), or import another SQLite database as a validated, consistent copy.
  SQLite import does not merge into the open database or alter the source.
- Export displayed, bounded results as JSON or CSV, and export a database SQL
  dump. Exports are staged atomically and never overwrite an existing file.
- Keep saved and recent local connection metadata, with explicit close and
  forget actions. Database contents and credentials remain local.
- Light and dark themes, native dialogs, and cross-platform Tauri packaging.

SSH connections are planned, but are not part of v1.

## Stack

- Tauri 2
- Rust with `rusqlite`
- Vue 3 and TypeScript
- Vite and pnpm

## Prerequisites

Install a current Rust toolchain, Node.js with pnpm, and the system packages
described by the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

## Development

```sh
pnpm install
pnpm tauri dev
```

Build the frontend:

```sh
pnpm build
```

Run the Rust test suite:

```sh
cargo test --workspace
```

Run the frontend tests, format and lint Rust, and build the Tauri application:

```sh
pnpm test
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
pnpm tauri build --debug --no-bundle
```

`pnpm build` builds the frontend without packaging. Tauri builds use the local
frontend output and produce debug artifacts when `--debug` is supplied.

Product direction and settled goals are documented in
[`CONTEXT.md`](CONTEXT.md).
