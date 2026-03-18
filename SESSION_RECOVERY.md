# Session Recovery Guide

**Use this file when starting a new session to quickly resume implementation.**

---

## 30-Second Recovery

```powershell
# 1. Check current status
Get-Content task_plan.md | Select-String "Current Phase:"
Get-Content progress.md | Select-String -Pattern "^## Session:" | Select-Object -Last 1

# 2. Check for uncommitted work
git status

# 3. Verify environment is ready
rustc --version; bun --version

# 4. Read implementation guide for current phase
# (see Current Phase below)
```

---

## Current Phase Quick Reference

**Current Phase:** Phase 0 - Project Scaffold & Environment Setup  
**Status:** In Progress  

### Immediate Next Steps
1. Initialize Tauri project: `bun create tauri-app@latest`
2. Install dependencies
3. Configure Tailwind
4. Create folder structure
5. Test dev mode

### Key Files to Read
- `IMPLEMENTATION.md` - Phase 0 section
- `docs/development/setup.md` - Dev environment setup
- `docs/architecture/system-architecture.md` - Overall architecture

---

## Full Recovery Procedure

### Step 1: Read Planning Files (2 minutes)
1. Open `IMPLEMENTATION.md` - understand current phase
2. Open `task_plan.md` - check phase status and checklist
3. Open `progress.md` - review recent work
4. Open `DEBUGGING.md` if you expect issues

### Step 2: Check Environment (1 minute)
```powershell
# Required versions
rustc --version    # Should be 1.75+
bun --version      # Should be 1.0+
node --version     # Should be 20+

# On Windows - verify build tools
cl                 # Should show Microsoft C++ compiler

# On macOS - verify Xcode tools
xcode-select -p    # Should show path
```

### Step 3: Check Project State (1 minute)
```powershell
# Check git status
git status

# See recent commits
git log --oneline -5

# Check what files exist
Get-ChildItem -Recurse src-tauri | Select-Object -First 20
```

### Step 4: Resume Work
Based on **Current Phase**:

| Phase | Resume Action |
|-------|---------------|
| 0 | Run `bun create tauri-app@latest handy-launcher --template svelte-ts` |
| 1 | Continue implementing `ollama_manager.rs` functions |
| 2 | Continue implementing `config_manager.rs` functions |
| 3 | Build UI components in `src/lib/components/` |
| 4 | Build status dashboard in `src/routes/status/` |
| 5 | Add tray configuration to `main.rs` |
| 6 | Write tests, run `cargo test` |
| 7 | Run `bun run tauri build` |

---

## Common Recovery Issues

### "I don't remember where I left off"
1. Check `task_plan.md` for "Current Phase"
2. Check `progress.md` for last session
3. Run `git diff` to see uncommitted changes
4. Check for TODO comments in code: `rg "TODO" src-tauri/src`

### "The dev server won't start"
1. Check `bun install` has been run
2. Delete `node_modules` and reinstall
3. Check `src-tauri/target` isn't corrupted
4. See `DEBUGGING.md` Phase 0 errors

### "There are compiler errors"
1. Check against `IMPLEMENTATION.md` for correct syntax
2. Check imports are correct
3. Run `cargo check` for Rust errors
4. Check `bun run check` for TypeScript errors

### "Tests are failing"
1. Read `DEBUGGING.md` Testing Failures section
2. Check if environment changed
3. Run with verbose: `cargo test -- --nocapture`
4. Check for test data fixtures

---

## Quick Commands Reference

### Development
```powershell
# Start dev mode
bun run tauri dev

# Check Rust
Cargo check

# Check TypeScript
bun run check

# Run tests
cargo test
```

### Git
```powershell
# See what changed
git status
git diff

# Commit progress
git add .
git commit -m "phase X: description of work"

# View history
git log --oneline -10
```

### Debugging
```powershell
# Check Ollama status
Invoke-RestMethod http://127.0.0.1:63452/api/version

# Check Handy config
Get-Content "$env:APPDATA/com.pais.handy/settings_store.json"

# View logs
Get-Content "$env:APPDATA/HandyLauncher/logs/launcher.log" -Tail 50
```

---

## Phase-Specific Recovery Notes

### Phase 0 Recovery
If scaffold was partially created:
- Delete `node_modules` and `src-tauri/target` if corrupted
- Re-run `bun install`
- Check `Cargo.toml` exists in `src-tauri/`

### Phase 1-2 Recovery (Backend)
If Rust code has issues:
- Run `cargo clean` then `cargo check`
- Check `src-tauri/src/lib.rs` exports all modules
- Verify `tauri.conf.json` has correct commands listed

### Phase 3-4 Recovery (Frontend)
If UI doesn't work:
- Check `src/lib/stores.ts` exports
- Verify Tailwind classes are applied
- Check browser console for JS errors

### Phase 5+ Recovery
If app behavior is wrong:
- Check `main.rs` for tray setup
- Verify `tauri.conf.json` permissions
- Test with `bun run tauri dev -- --release`

---

## Session Start Template

When starting a new session, update `progress.md` with:

```markdown
## Session: YYYY-MM-DD HH:MM

### Resuming From
- Phase: X
- Status: In Progress
- Last action: [description]

### Plan for This Session
1. [ ] Task 1
2. [ ] Task 2
3. [ ] Task 3

### Environment Check
- [ ] Rust: OK
- [ ] Bun: OK
- [ ] Project: OK

### Work Done
- [description]

### Issues Encountered
- [description and resolution]

### Next Session
- Continue with: [specific task]
- Blockers: [none or description]
```
