# LiteStation

LiteStation is a focused cross-platform desktop application for working with
local SQLite databases.

## Current capabilities

- Open an existing SQLite file through the native file picker.
- Open it in read-only or read-write mode without creating a missing file.
- Introspect tables, views, indexes, and triggers in the Rust backend.
- Display the grouped schema in the Vue interface.
- Retain an open database connection and close it explicitly.
- Return structured backend errors for invalid paths and databases.
- Exercise the database backend with automated Rust tests.

SQL query execution, data editing, import/export, saved connections, and SSH
connections are not implemented yet.

## Stack

- Tauri 2
- Rust with `rusqlite`
- Vue 3 and TypeScript
- Vite and pnpm

## Prerequisites

Install a current Rust toolchain, Node.js with pnpm, and the system
dependencies described by the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

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

Product direction and settled goals are documented in
[`CONTEXT.md`](CONTEXT.md).
