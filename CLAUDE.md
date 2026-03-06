# CLAUDE.md — OpenClaw Manager

This file provides context and conventions for AI assistants (like Claude Code) working in this repository.

---

## Project Overview

**OpenClaw Manager** is a cross-platform desktop application built with **Tauri 2.0** (Rust backend + React frontend). It provides a GUI for managing the OpenClaw AI assistant framework — configuring AI providers, message channels (Telegram, Discord, Slack, etc.), MCP servers, skills, agents, and service lifecycle.

- **Current version**: 0.0.17
- **License**: MIT
- **Platforms**: macOS, Windows, Linux

---

## Architecture

```
openclaw-manager/
├── src/                    # React + TypeScript frontend
│   ├── components/         # UI components (one folder per page/feature)
│   ├── hooks/              # Custom React hooks
│   ├── lib/                # Utilities: logger.ts, tauri.ts (IPC types)
│   ├── stores/             # Zustand global state (appStore.ts)
│   ├── styles/             # Tailwind globals
│   ├── App.tsx             # Root component, routing, error boundary
│   └── main.tsx            # React entry point
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── commands/       # Tauri command handlers (exposed to frontend)
│   │   ├── models/         # Data structures (config, status)
│   │   └── utils/          # Platform utils, shell, file ops, log sanitizer
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri config (version, window, updater)
├── public/                 # Static assets
├── index.html              # HTML entry
├── package.json            # NPM scripts and JS dependencies
├── vite.config.ts          # Vite bundler config
├── tailwind.config.js      # Tailwind theme (custom brand colors)
└── tsconfig.json           # TypeScript config (strict mode, @/* alias)
```

### Frontend → Backend Communication

All Tauri IPC calls use `invoke<ReturnType>('command_name', args)`. Types for all commands and return values are defined in `src/lib/tauri.ts`. Never import Node.js or Tauri APIs directly in component files — go through `src/lib/tauri.ts`.

### Backend Commands (src-tauri/src/main.rs)

40+ commands are registered. Key categories:

| Category | Examples |
|---|---|
| Service lifecycle | `get_service_status`, `start_service`, `stop_service`, `restart_service` |
| Process/version checking | `check_openclaw_installed`, `get_openclaw_version`, `check_port_in_use` |
| Configuration | `get_config`, `save_config`, `get_ai_config`, `save_provider`, `get_channels_config` |
| Gateway/auth | `get_or_create_gateway_token`, `get_dashboard_url`, `repair_device_token` |
| AI providers | `get_official_providers`, `add_available_model` |
| Skills | `get_available_skills`, `install_skill` |
| Diagnostics | `run_system_diagnostics`, `check_connectivity`, `test_ai_provider` |
| Installation | `install_openclaw` |

---

## Tech Stack

### Frontend
- **React 18** — UI library
- **TypeScript 5** — strict mode enabled
- **Vite 6** — build tool, dev server on port **1420** (required by Tauri)
- **TailwindCSS 3** — utility-first styling with custom OpenClaw theme
- **Zustand 5** — global state management (`src/stores/appStore.ts`)
- **Framer Motion 11** — animations
- **Lucide React** — icons

### Backend (Rust)
- **Tauri 2.0** — desktop app framework
- **Tokio** — async runtime
- **Serde / Serde JSON / serde_yaml** — serialization
- **Chrono** — date/time
- **regex** — pattern matching
- **thiserror** — ergonomic error types
- **log + env_logger** — structured logging (`RUST_LOG` env var)

---

## Development Workflow

### Prerequisites
- **Node.js** 18+ and npm
- **Rust** 1.70+ with `rustup`
- **Tauri CLI** (installed via npm)
- Platform build tools:
  - macOS: Xcode Command Line Tools
  - Windows: MSVC Build Tools, WebView2
  - Linux: `build-essential`, `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`

### Common Commands

```bash
# Install JS dependencies
npm install

# Full desktop app with hot-reload (recommended for dev)
npm run tauri:dev

# Frontend only in browser (no Tauri APIs available)
npm run dev

# Type-check + production build (JS only)
npm run build

# Build release installers for current platform
npm run tauri:build

# Rust type-check (fast, no compile)
cd src-tauri && cargo check

# Run Rust unit tests
cd src-tauri && cargo test
```

### Build Output Locations
- Frontend bundle: `dist/`
- Release binaries: `src-tauri/target/release/bundle/`
  - Windows: `msi/*.msi`, `nsis/*.exe`
  - macOS: `dmg/*.dmg`
  - Linux: `deb/*.deb`, `appimage/*.AppImage`

---

## Key Conventions

### TypeScript
- **Strict mode** is enabled — no implicit `any`, no unused locals/parameters
- Path alias `@/*` maps to `src/*` (use this instead of relative `../../`)
- All Tauri IPC types live in `src/lib/tauri.ts` — add new types here
- State goes into `src/stores/appStore.ts` via Zustand; avoid local state for anything shared

### Rust
- Follow standard Rust idioms: `Result<T, E>`, `Option<T>`, no `unwrap()` in production paths
- Commands return `Result<T, String>` (Tauri requirement for serializable errors)
- Log with `log::info!`, `log::warn!`, `log::error!` — never `println!` in production code
- Security-sensitive values (tokens, keys) must go through `src-tauri/src/utils/log_sanitizer.rs` before logging
- Platform-specific code belongs in `src-tauri/src/utils/platform.rs`

### Styling
- Use **Tailwind utility classes** exclusively — no inline styles, no separate CSS files (except `globals.css`)
- Custom brand colors defined in `tailwind.config.js`:
  - Primary red: `openclaw` (`#f94d3a`)
  - Dark backgrounds: `dark-*` palette
  - Accents: `cyan-*`, `purple-*`, `green-*`
- Custom animations: `animate-pulse-slow`, `animate-glow`, `animate-slide-up`

### Component Structure
Each page has its own folder under `src/components/`. Component folders typically contain:
- A main `*.tsx` component file
- Sub-components as needed within the same folder
- No barrel `index.ts` files — import directly

### Configuration File
The OpenClaw config is stored at `~/.openclaw/openclaw.json`. The Rust `config.rs` command module manages all reads/writes. Never directly manipulate this file from the frontend — always go through Tauri commands.

---

## State Management

Global state in `src/stores/appStore.ts` (Zustand):
- Service status (running, PID, port, uptime, memory, CPU)
- System info (OS, installed versions, config directory)
- Notifications

Fetch state by calling Tauri commands from `App.tsx` or component `useEffect` hooks, then update the store. The `useService` hook (`src/hooks/useService.ts`) wraps service start/stop/restart logic.

---

## Testing

- **Rust**: Unit tests in `src-tauri/src/utils/log_sanitizer_tests.rs`. Run with `cargo test`.
- **Frontend**: No test framework is configured yet. TypeScript strict mode provides compile-time safety. Manual testing via `npm run tauri:dev`.

When adding Rust logic, add corresponding tests in a `_tests.rs` file alongside the module.

---

## Release Process

1. Update version in **both**:
   - `package.json` → `"version"`
   - `src-tauri/tauri.conf.json` → `"version"`
2. Commit the version bump
3. Create and push a git tag: `git tag v0.x.y && git push origin v0.x.y`
4. The GitHub Actions workflow (`.github/workflows/release.yml`) automatically builds all platforms and publishes the GitHub Release
5. Build takes ~10–15 minutes

The updater endpoint is configured in `tauri.conf.json`. Signing uses `TAURI_SIGNING_PRIVATE_KEY` / `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` GitHub secrets.

---

## CI/CD

**`.github/workflows/release.yml`** triggers on:
- Push of `v*` tags
- Manual `workflow_dispatch`

Matrix builds across:
- Windows (x86_64)
- macOS ARM64 + Intel x86_64 (universal DMG)
- Linux Ubuntu 22.04 (x86_64)

---

## Security Notes

- **Never log tokens, API keys, or secrets**. All log output passes through `log_sanitizer.rs` which redacts known sensitive patterns.
- Config files may contain API keys — handle them only in Rust commands, never pass raw keys to the frontend beyond what is strictly necessary.
- Tauri capabilities/permissions are declared in `src-tauri/capabilities/` — follow least-privilege when adding new shell or filesystem access.

---

## File Quick-Reference

| File | Purpose |
|---|---|
| `src/lib/tauri.ts` | All Tauri IPC types and invoke wrappers |
| `src/lib/logger.ts` | Frontend logger utility |
| `src/stores/appStore.ts` | Global Zustand state |
| `src/App.tsx` | Root component, page routing, env checks |
| `src-tauri/src/main.rs` | Backend entry, command registration |
| `src-tauri/src/commands/config.rs` | All config read/write commands (~3800 lines) |
| `src-tauri/src/commands/service.rs` | Service start/stop/status |
| `src-tauri/src/commands/installer.rs` | OpenClaw installation logic |
| `src-tauri/src/commands/diagnostics.rs` | System diagnostics |
| `src-tauri/src/utils/log_sanitizer.rs` | Redacts sensitive values from logs |
| `src-tauri/src/utils/platform.rs` | OS-specific path/command helpers |
| `tailwind.config.js` | Brand colors, custom animations |
| `tauri.conf.json` | App version, window size, updater config |
