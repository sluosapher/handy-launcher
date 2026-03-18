# Nested App Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Scaffold the nested `handy-launcher` Tauri/Svelte project inside the existing repo so Phase 0 can proceed without disrupting the documentation workspace.

**Architecture:** A two-layer workspace keeps docs in the repo root while the runnable app lives under `handy-launcher/`; the nested folder hosts the full `src-tauri`, `src`, and config files described in `IMPLEMENTATION.md`, and all build/test commands run from that folder.

**Tech Stack:** Bun, Tauri (Rust backend), Svelte+TypeScript frontend, Tailwind CSS.

---

### Task 1: Scaffold Nested Tauri App

**Files:**
- Create: `handy-launcher/.metadata` (auto by `bun create`), `handy-launcher/README.md` (auto), `handy-launcher/package.json`
- Modify: Top-level `task_plan.md` and `progress.md` (reference new workspace in notes)
- Test: N/A (scaffold verification via `bun run tauri dev`)

**Step 1: Run `bun create` inside `handy-launcher/`**

```bash
mkdir -p handy-launcher
cd handy-launcher
bun create tauri-app@latest handy-launcher --template svelte-ts
```

Expected: `handy-launcher/handy-launcher` is created with `package.json`, `src-tauri/`, `src/`, etc.

**Step 2: Flatten the generated structure**

```bash
cd handy-launcher
mv handy-launcher/* .
rm -rf handy-launcher
```

Expected: `handy-launcher/src-tauri` and `handy-launcher/src` exist directly under `handy-launcher/`.

**Step 3: Install frontend dependencies**

```bash
cd handy-launcher
bun install
bun add -d @tauri-apps/cli lucide-svelte @tauri-apps/api
bun add -d tailwindcss postcss autoprefixer
```

Expected: `bun.lockb` created, `node_modules` ready.

**Step 4: Initialize Tailwind and update theme**

```bash
cd handy-launcher
bunx tailwindcss init -p
```

Update `tailwind.config.cjs` to enable dark mode and add `docs/user-guides/ui-specifications.md` colors.

**Step 5: Commit the scaffold**

```bash
git add handy-launcher
git commit -m "feat: scaffold nested handy launcher app"
```

Expected: Git tree includes `handy-launcher/` with Tauri project.

---

### Task 2: Add Rust Module Structure

**Files:**
- Create: `handy-launcher/src-tauri/src/commands/mod.rs`, `services/mod.rs`, `models/mod.rs`, `utils/mod.rs`, plus listed submodules (e.g., `commands/ollama.rs`, `services/ollama_manager.rs`, etc.).
- Modify: `handy-launcher/src-tauri/src/lib.rs` to re-export modules, `handy-launcher/src-tauri/Cargo.toml` to list new modules/crates.
- Test: N/A (initial module scaffolding only).

**Step 1: Create the directory tree**

```bash
cd handy-launcher
mkdir -p src-tauri/src/{commands,services,models,utils}
```

**Step 2: Add placeholder `mod.rs` files**

Create each `mod.rs` with `pub mod` declarations for the submodules (e.g., `pub mod ollama;`) and a `pub mod` in `lib.rs`.

**Step 3: Create stub files**

Add empty files for `commands/ollama.rs`, `services/ollama_manager.rs`, `models/app_state.rs`, `utils/paths.rs`, etc., with basic `pub struct` placeholders.

**Step 4: Update `Cargo.toml`**

Ensure `[lib]` section and dependencies (e.g., `tauri`, `serde`, `tokio`) are listed; run `cargo check` from `handy-launcher/src-tauri`.

**Step 5: Commit the module scaffolding**

```bash
cd handy-launcher
git add src-tauri/src
git commit -m "chore: add phase 0 module scaffolding"
```

Expected: Module files present for later Phase 1/2 work.

---

### Task 3: Configure Tauri Capabilities & Verify

**Files:**
- Modify: `handy-launcher/src-tauri/tauri.conf.json`, `handy-launcher/src-tauri/capabilities/default.json`.
- Test: `handy-launcher` dev server.

**Step 1: Enable required capabilities**

Update `capabilities/default.json` to include `["shell:allow-execute","fs:allow-read","fs:allow-write","os:allow-platform"]`.

**Step 2: Align `tauri.conf.json`**

Ensure `tauri.conf.json` references the capabilities file and sets `allowlist` entries for shell/fs/os as needed.

**Step 3: Run `bun run tauri dev`**

```bash
cd handy-launcher
bun run tauri dev
```

Expected: App compiles and Svelte starter page loads without errors.

**Step 4: Stop dev server and document result**

Use Ctrl+C after verifying; log any warnings in `progress.md`.

**Step 5: Commit configuration changes**

```bash
git add src-tauri/tauri.conf.json src-tauri/capabilities/default.json
git commit -m "chore: configure tauri capabilities"
```

Expected: Capabilities configured and verified workspace ready for Phase 1.
