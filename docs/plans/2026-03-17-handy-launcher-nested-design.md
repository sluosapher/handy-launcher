# Handy Launcher Nested App Design

## Overview
- **Goal:** Keep the existing repo root as the planning/coordination layer and host the runnable Tauri + Svelte app inside a nested `handy-launcher/` directory so the implementation matches `IMPLEMENTATION.md` without moving or disturbing the documentation files.
- **Success metric:** All tooling (`bun`, `tauri dev`, Rust/Cargo) operate inside `handy-launcher/`, and the root still only tracks docs, recovery materials, and the new plan artifacts.

## Architecture
- **Workspace separation:** The top-level repo remains the “control plane” for docs, recovery notes, and the implementation plan. The nested directory contains the actual app workspace with `src-tauri`, `src`, `package.json`, etc., mirroring the layout described in Phase 0 of `IMPLEMENTATION.md`.
- **Build invocation:** Commands such as `bun run tauri dev` are scoped to the nested directory so the generated `node_modules` and build artifacts live inside `handy-launcher/`, avoiding pollution of the doc workspace.
- **Configuration:** Update `docs` references to point from the nested app (e.g., `docs/user-guides/ui-specifications.md`) without relocating the docs themselves.

## Directory Layout
```
handy-launcher/            # Repo root (documentation, plans, instructions)
├── docs/                  # Existing docs (requirements, architecture, ui specs, plans)
├── src-tauri/             # (Not yet; part of nested app)
├── handy-launcher/        # Nested Tauri app
│   ├── src-tauri/         # Rust backend per IMPLEMENTATION.md
│   ├── src/               # Svelte frontend routes, components
│   ├── package.json
│   ├── bun.lockb
│   ├── tsconfig.json
│   └── tailwind.config.js
├── task_plan.md           # Phase tracking (control plane)
└── progress.md            # Session log
```
The nested folder will ultimately contain the scaffolding listed under Phase 0 (commands, services, models, utils, Tauri config, etc.).

## Key Decisions
- **Do not rename or move docs:** This preserves the session recovery experience and keeps all references valid.
- **Run `bun create tauri-app@latest handy-launcher --template svelte-ts` inside `handy-launcher/` and then flatten the generated subdirectory so the app files live directly under `handy-launcher/`.**
- **Keep the nested `handy-launcher/` directory under version control** so we can commit the scaffold and follow the already outlined phase-based plan.

## Next Steps
1. Use `bun create tauri-app@latest handy-launcher --template svelte-ts` inside `handy-launcher/` and move the generated files up one level so the app files live directly under `handy-launcher/`.
2. Install frontend dependencies (`bun install`, `@tauri-apps/cli`, `lucide-svelte`, `@tauri-apps/api`, `tailwindcss`, `postcss`, `autoprefixer`) inside the nested folder.
3. Add the Rust module folders and `capabilities/default.json` entries listed in Phase 0, keeping the root repo untouched.
4. Ensure `bun run tauri dev` passes from within the nested folder; this will signal readiness to proceed with Phase 1.
