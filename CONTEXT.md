# LiteStation Project Context

## Product Vision

LiteStation is a modern, cross-platform desktop application for working with SQLite databases.

The product takes inspiration from TablePlus—particularly its simplicity, speed, keyboard-friendly workflows, and focused interface—without attempting to reproduce TablePlus or support its full database ecosystem.

LiteStation supports SQLite only.

## Goals

LiteStation should let users:

- Open and manage local SQLite database files.
- Connect to SQLite databases on remote machines through SSH.
- Browse tables, views, indexes, triggers, and schema definitions.
- Write and execute SQL using a capable query editor.
- Inspect query results in a fast, readable data grid.
- Insert, update, and delete rows.
- Import databases from SQL dumps and other SQLite database files.
- Export data and databases as JSON, CSV, and SQL.
- Perform common database operations without writing SQL.
- Work efficiently through keyboard shortcuts and a command palette.
- Safely manage multiple saved database connections.
- Use the application on Linux, macOS, and Windows.

The interface should feel focused and lightweight, exposing advanced functionality without overwhelming the user.

## Current Project State

The v1 local SQLite MVP is implemented:

- Tauri 2 desktop shell with a Rust backend and Vue 3/TypeScript frontend.
- Native file dialogs, local read-only and read-write connections, grouped schema browsing, and explicit close.
- SQL editing and execution for selected SQL, the statement at the cursor, and confirmed sequential run-all (non-atomic), with bounded results.
- Bounded table browsing plus guarded insert, update, and delete operations.
- Atomic SQL-dump import and validated SQLite-file copy import; the latter does not merge into or modify the open database.
- JSON/CSV exports of displayed results and SQL-dump export; output files do not overwrite existing destinations.
- Saved and recent local connection metadata, light/dark themes, structured errors, and automated frontend/Rust coverage.

Post-v1 exclusions include SSH/remote databases, release signing, and broader database engines or hosted services. Cross-platform packaging is configured, but platform-specific manual testing and release validation remain future work.

## Core Product Principles

### SQLite-focused

SQLite is the only supported database engine. Product and architecture decisions should take advantage of SQLite-specific functionality rather than introducing abstractions intended for other database systems.

### Local-first

The application runs locally and does not require a LiteStation account or hosted service.

Database contents and connection details must not be sent to an external service.

### Safe by default

Database tools can cause irreversible data loss. LiteStation should:

- Clearly distinguish read-only and writable connections.
- Warn before destructive operations.
- Make transaction state visible.
- Avoid silently changing database pragmas or journal settings.
- Never persist credentials as plain text.
- Require SSH host identity verification.
- Avoid remote-file techniques that risk corruption or stale writes.

### Responsive with large datasets

The interface must remain responsive when databases or query results are large.

Operations should support cancellation where practical. Results should be retrieved and displayed in bounded pages or batches rather than loading an unlimited result set into frontend memory.

### Keyboard-friendly

Common actions should have discoverable keyboard shortcuts. A command palette should provide access to actions without requiring users to navigate menus.

### Native desktop behavior

The application should respect platform conventions while maintaining a consistent identity across Linux, macOS, and Windows.

## Primary Workflows

### Local databases

A user can:

1. Select an existing SQLite database file.
2. Choose whether to open it as read-only or writable.
3. Browse its schema and data.
4. Execute SQL.
5. Edit data when the connection is writable.
6. Import an SQL dump or another SQLite database file.
7. Export selected data or a database in JSON, CSV, or SQL format.

Database operations should run in the Rust backend. The frontend must not access database files directly.

### Remote databases over SSH

A user can save an SSH connection containing:

- Display name.
- Host and port.
- Username.
- Authentication configuration.
- Remote SQLite database path.
- Read-only or writable mode.

SQLite operations must execute on the machine where the database file resides. LiteStation must not treat an SSH-mounted filesystem as if it were a local disk and must not use a copy-edit-upload workflow that could overwrite concurrent changes.

The exact remote execution protocol is an implementation decision that must be settled before remote support is built.

### SQL execution

The query workspace should support:

- Multiple query tabs.
- Running the selected SQL.
- Running the statement under the cursor.
- Running all statements in the editor.
- Parameterized execution where applicable.
- Execution duration and affected-row reporting.
- Clear SQLite error messages.
- Query cancellation where supported.
- Query history.
- Saved or favorite queries.

### Data browsing and editing

Users should be able to:

- Browse table rows.
- Sort and filter results.
- Select and copy cells or rows.
- Inspect long text and binary values.
- Insert rows.
- Edit existing rows.
- Delete rows.
- Review generated SQL before risky changes when practical.

Edits must correctly handle primary keys, composite keys, nullable values, SQLite row identifiers, and tables that cannot be updated unambiguously.

### Import and export

LiteStation should support:

- Importing a SQL dump into a SQLite database.
- Importing another SQLite database file into the current workflow without modifying the source file.
- Exporting query results or selected table data as JSON or CSV.
- Exporting a database or selected schema and data as a SQL dump.

Import and export must make the source, destination, overwrite behavior, and scope clear before writing data. The application must report errors without leaving an import's transaction partially committed.

## Interface Direction

The primary workspace should use a compact database-tool layout:

- **Connection sidebar:** Saved and active connections.
- **Object browser:** Tables, views, indexes, and triggers.
- **Workspace tabs:** Table data, structure views, and SQL queries.
- **Main content:** Query editor or result grid.
- **Status area:** Connection state, transaction state, execution time, and row count.
- **Command palette:** Searchable application and database actions.

The design should prioritize content density, clear hierarchy, and restrained visual styling. Both light and dark themes should be supported.

TablePlus is a source of product inspiration, not a design specification. LiteStation should develop its own visual identity and interaction details.

## Architecture Boundaries

### Vue frontend

The frontend is responsible for:

- Rendering application state.
- User interaction.
- Query editing.
- Result-grid presentation.
- Connection and schema navigation.
- Dialogs, menus, shortcuts, and command discovery.

The frontend should communicate with Rust through typed Tauri commands and events. It should not contain SQLite or SSH implementation logic.

### Rust backend

The Rust backend is responsible for:

- Opening and closing database connections.
- Executing SQL and managing transactions.
- Schema introspection.
- Importing and exporting data and SQL dumps.
- Mapping SQLite values into transport-safe representations.
- SSH connections and remote execution.
- Filesystem access.
- Credential integration.
- Query cancellation and resource limits.
- Returning structured errors to the frontend.

Core database behavior should remain independent of Tauri command handlers so it can be tested without launching the desktop application.

### Security boundary

All file, database, SSH, and credential operations cross a privileged boundary. Tauri permissions and capabilities should be kept as narrow as practical.

Secrets should use operating-system credential storage when persisted. Non-secret connection metadata may be stored in the application data directory.

## Initial MVP

The first usable release should include:

1. Opening local SQLite databases.
2. Read-only and writable connection modes.
3. Schema and object browsing.
4. SQL editor with current-statement and selected-query execution.
5. Paginated or bounded query results.
6. Table data browsing.
7. Basic row insertion, editing, and deletion.
8. Importing SQL dumps and other SQLite database files.
9. Exporting query results or table data as JSON and CSV.
10. Exporting databases or selected schema and data as SQL dumps.
11. Structured errors and destructive-action confirmation.
12. Saved local connections and recently opened databases.
13. Linux, macOS, and Windows packaging.

Remote SSH access is a core product goal, but should follow a stable local database foundation unless its protocol is deliberately selected as part of the first milestone.

## Later Capabilities

Potential post-MVP capabilities include:

- Remote SQLite access over SSH.
- SSH agent and private-key authentication.
- Query history and favorites.
- Database backup and restore.
- Table and index creation tools.
- Schema modification workflows.
- Explain-query-plan visualization.
- Attached databases.
- Data comparison.
- Customizable shortcuts and themes.

These are directions rather than committed release requirements.

## Explicit Non-goals

LiteStation will not initially provide:

- Support for PostgreSQL, MySQL, SQL Server, or other database engines.
- Cloud-hosted account synchronization.
- Multi-user collaboration.
- A browser-hosted database administration service.
- Mobile applications.
- A general-purpose programming IDE.
- Complete feature parity with TablePlus.
- Unsafe editing of SQLite files through SSHFS or similar network mounts.

## Quality Expectations

Changes should favor simple, explicit implementations over speculative abstractions.

Important behavior should be covered by automated tests, especially:

- SQLite value conversion.
- Query execution and error handling.
- Transaction behavior.
- Schema introspection.
- Row identity and update safety.
- Pagination and result limits.
- Import and export correctness, rollback behavior, and format handling.
- SSH host verification and remote protocol handling.

User-facing failures should explain what happened and provide enough information to correct the problem without exposing credentials.

## Decisions Still Required

Before remote SSH support is implemented, decide:

- Whether the remote host must already have the `sqlite3` CLI installed.
- Whether LiteStation deploys a small remote helper process.
- Which SSH authentication methods are supported initially.
- How SSH host keys are approved and stored.
- How cancellation and interrupted connections affect remote transactions.

Before building the main interface, decide:

- Which Vue component and styling approach to use.
- Which SQL editor component to use.
- Which data-grid implementation can handle large result sets accessibly and efficiently.
- The initial shortcut conventions for each operating system.
