# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

PA eDocket Desktop is a production-grade cross-platform desktop application for Pennsylvania court docket management. Built with **Tauri v2** (Rust backend) and **React/TypeScript** (frontend), it provides live docket search, document drafting, Bluebook citations, and e-filing integration for Pennsylvania courts.

## Development Commands

### Frontend Development
```bash
npm run dev              # Start Vite dev server (port 1420)
npm run build            # Build frontend for production
npm run lint             # Run ESLint
npm run lint:fix         # Run ESLint with auto-fix
npm run format           # Format code with Prettier
```

### Tauri Development
```bash
npm run tauri:dev        # Start Tauri with hot reload (recommended)
npm run tauri:build      # Build production application
```

### Rust Development
```bash
cd src-tauri

# Testing
cargo test               # Run all tests
cargo test --lib         # Unit tests only
cargo test --test integration_tests  # Integration tests

# Code Quality
cargo fmt --check        # Check formatting
cargo clippy             # Run linter
cargo bench              # Run benchmarks

# Logging
RUST_LOG=debug cargo run # Run with debug logging
```

### Build and Package
```bash
./scripts/package.sh                    # Basic release build
./scripts/package.sh --debug            # Debug build
./scripts/package.sh --sign             # Signed build
./scripts/package.sh --clean            # Clean build
./scripts/build-production.sh           # Full production build
```

### Database Management
```bash
npm run db:migrate       # Run database migrations
npm run db:seed          # Seed database with test data
```

### Testing
```bash
./scripts/run_tests.sh               # Run all tests
./scripts/run_tests.sh --unit        # Unit tests only
./scripts/run_tests.sh --integration # Integration tests only
./scripts/run_tests.sh --coverage    # With coverage
./scripts/run_tests.sh --bench       # Performance benchmarks
```

## Architecture

### High-Level Structure

This is a **hybrid architecture** with a Rust backend and TypeScript frontend:

```
Frontend (React/TypeScript)
    ↓ Tauri IPC Commands
Rust Backend (Command Handlers)
    ↓
Service Layer (Business Logic)
    ↓
Provider Layer (External APIs)
```

### Rust Backend Architecture (`src-tauri/`)

The Rust backend follows a **layered architecture**:

1. **Command Handlers** (`src/services/commands.rs`) - Tauri IPC command handlers that receive calls from frontend
2. **Service Layer** (`src/services/`) - Business logic modules:
   - `citations.rs` - Bluebook citation processing
   - `drafting.rs` - Document generation from templates
   - `export.rs` - Export to JSON, CSV, PDF, ZIP
   - `database.rs` - SQLite database operations
   - `court_rules.rs` - Court-specific formatting rules
   - `automation.rs` - Background job scheduling
   - `task_runner.rs` - Async task execution
   - `watchlist.rs` - Case monitoring and notifications
   - `security.rs` - Credential management via OS keychain

3. **Provider Layer** (`src/providers/`) - External API integrations with traits:
   - `SearchProvider` trait - Defines search and docket retrieval interface
   - `EFilingProvider` trait - Defines e-filing operations interface
   - Implementations:
     - `ujs_portal.rs` - PA UJS Web Portal scraping
     - `pacfile.rs` - PACFile e-filing system
     - `county_efiling.rs` - Philadelphia/Allegheny county systems
     - `ctrack.rs` - C-Track civil case management
   - `rate_limiter.rs` - Rate limiting for all provider requests
   - `client.rs` - Shared HTTP client with retry logic

4. **Domain Models** (`src/domain/mod.rs`) - Shared Rust structs that mirror TypeScript types

5. **Configuration** (`src/config/`) - YAML-based configuration loaders

### Frontend Architecture (`src/`)

React application with clear separation:

1. **Pages** (`src/pages/`) - Top-level route components
2. **Components** (`src/components/`) - Reusable UI components
3. **Domain Types** (`src/types/domain.ts`) - TypeScript interfaces matching Rust structs (200+ lines)
4. **API Types** (`src/types/api.ts`) - Request/response shapes
5. **Hooks** (`src/hooks/`) - React hooks for state management
6. **Citation Engine** (`src/lib/citations/`) - Client-side citation processing:
   - `parser.ts` - Extract citations from text
   - `formatter.ts` - Format to Bluebook style
   - `validator.ts` - Validate citation correctness
   - `engine.ts` - Main interface and TOA generation
   - `rules.ts` - Bluebook rule definitions

### Key Design Patterns

1. **Trait-Based Providers**: All court data sources implement `SearchProvider` or `EFilingProvider` traits for consistent interfaces

2. **Rate Limiting**: All external API calls go through `rate_limiter.rs` to respect court system limits

3. **Type Safety**: Domain models defined once in Rust (`src-tauri/src/domain/`) and mirrored in TypeScript (`src/types/domain.ts`)

4. **Command Pattern**: Frontend calls Rust via Tauri commands (e.g., `cmd_search`, `cmd_get_docket`, `cmd_export`)

5. **Configuration-Driven**: Court rules and provider settings in YAML files (`config/courts.yaml`, `config/providers.yaml`, `config/jobs.yaml`)

## Important Development Guidelines

### When Working with Rust Backend

1. **All Tauri commands** are registered in `src-tauri/src/lib.rs` using `invoke_handler!` macro
2. **Command handlers** must be in `src/services/commands.rs` and follow naming convention `cmd_*`
3. **Provider implementations** must handle rate limiting - never call external APIs directly
4. **Error handling**: Use `anyhow::Result` for internal errors, `ProviderError` for provider-specific errors
5. **Database**: Uses SQLx with compile-time checked queries - run migrations before testing

### When Working with Frontend

1. **Type synchronization**: Keep `src/types/domain.ts` in sync with Rust domain models
2. **Tauri invocations**: Use `import { invoke } from '@tauri-apps/api/core'` for calling Rust commands
3. **State management**: Zustand for global state, React Query for server state
4. **Routing**: React Router v6 - main routes defined in `src/App.tsx`
5. **Citations**: Use `CitationEngine` class for all citation operations, never parse manually

### When Working with Providers

1. **Never bypass rate limiting** - all HTTP requests must go through the rate limiter
2. **Configuration**: Provider settings in `config/providers.yaml` include base URLs, rate limits, retry policies
3. **Scraping**: UJS Portal uses `scraper` crate with CSS selectors - be defensive about HTML structure changes
4. **Authentication**: Use `security.rs` service to store credentials in OS keychain (never in code/config)
5. **Testing providers**: Use mock responses in tests - do NOT hit real court systems during tests

### Configuration Files

- `config/courts.yaml` - Court-specific rules (margins, fonts, captions, page limits)
- `config/providers.yaml` - Provider endpoints, rate limits, retry policies
- `config/jobs.yaml` - Background job schedules and retry policies
- `src-tauri/tauri.conf.json` - Tauri app configuration, CSP, bundling

### Testing Strategy

1. **Rust unit tests**: Colocated with modules using `#[cfg(test)]`
2. **Rust integration tests**: In `src-tauri/tests/` directory
3. **Property testing**: Uses `proptest` for domain model validation
4. **Benchmarks**: Uses `criterion` in `src-tauri/benches/`
5. **Mock providers**: Create test doubles for provider traits

## Common Gotchas

1. **Domain type drift**: TypeScript and Rust types can get out of sync - always check both when adding fields
2. **Rate limiting**: Court systems aggressively rate limit - respect the configured limits
3. **Citation parsing**: Bluebook rules are complex - use existing `CitationEngine` rather than custom parsing
4. **SQLite migrations**: Must be in `src-tauri/migrations/` and run before first use
5. **Tauri CSP**: Content Security Policy in `tauri.conf.json` restricts allowed domains - update when adding new APIs
6. **Template paths**: Document templates bundled as resources - must be in `src-tauri/templates/` and listed in `tauri.conf.json`

## Data Flow Examples

### Search Flow
```
User enters search → SearchForm component → invoke('cmd_search') →
commands.rs → providers::ujs_portal → rate_limiter → HTTP request →
scrape HTML → parse to SearchResult → return to frontend → display
```

### E-Filing Flow
```
User submits filing → EFilingPage → invoke('cmd_efiling_submit') →
commands.rs → security::get_credentials → providers::pacfile::submit →
rate_limiter → authenticate → upload files → poll status →
save receipt → notify user
```

### Document Drafting Flow
```
User selects template → DraftingPage → invoke('cmd_draft') →
services::drafting → load template → merge docket data →
apply court rules → generate PDF → export::save_file →
return file path → open in system viewer
```

## External Dependencies to Know

- **Tauri v2**: Desktop framework - breaking changes from v1
- **SQLx**: Async SQL with compile-time query checking
- **Tokio**: Async runtime for all I/O
- **Scraper**: HTML parsing for court websites
- **reqwest**: HTTP client (uses rustls for TLS)
- **React 19**: Note some breaking changes from React 18
- **Zustand**: Simpler than Redux, used for global UI state
- **React Hook Form + Zod**: Form validation pattern used throughout

## Deployment Notes

- GitHub Actions workflows in `.github/workflows/` handle CI/CD
- Multi-platform builds: macOS (DMG), Windows (MSI/EXE), Linux (DEB/AppImage)
- Auto-updater configured in `tauri.conf.json` - publishes to GitHub releases
- Code signing setup via `./scripts/setup-signing.sh`
- Production deployment via `./scripts/deploy.sh`
