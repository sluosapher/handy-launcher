# Setup Flow Auto-Advance Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the setup wizard advance deterministically when the machine already satisfies later setup requirements.

**Architecture:** Extract the wizard progression rules into a pure TypeScript helper in `src/lib` so the behavior is testable without mounting the Svelte page. Keep `+page.svelte` responsible for fetching state and scheduling the existing 500 ms transition, while the helper decides which step should be active for a given snapshot.

**Tech Stack:** SvelteKit, TypeScript, Bun test

---

### Task 1: Extract step-resolution rules

**Files:**
- Create: `handy-launcher/src/lib/setup-flow.ts`
- Test: `handy-launcher/src/lib/setup-flow.test.ts`

**Step 1: Write the failing test**

```ts
test('skips directly to setup when Ollama is already installed', () => {
  expect(
    resolveSetupStep({
      currentStep: 1,
      systemHealthState: 'warning',
      hasOllamaBinary: true,
      isSetupComplete: false
    })
  ).toBe(3);
});
```

**Step 2: Run test to verify it fails**

Run: `bun test src/lib/setup-flow.test.ts`
Expected: FAIL because `resolveSetupStep` does not exist yet.

**Step 3: Write minimal implementation**

```ts
export function resolveSetupStep(snapshot: SetupFlowSnapshot): SetupStep {
  if (snapshot.isSetupComplete) return 4;
  if (snapshot.hasOllamaBinary || snapshot.isOllamaRunning) return 3;
  if (snapshot.currentStep === 2 && snapshot.systemHealthState === 'pass') return 3;
  return snapshot.currentStep;
}
```

**Step 4: Run test to verify it passes**

Run: `bun test src/lib/setup-flow.test.ts`
Expected: PASS

### Task 2: Wire the page to the helper

**Files:**
- Modify: `handy-launcher/src/routes/+page.svelte`
- Test: `handy-launcher/src/lib/setup-flow.test.ts`

**Step 1: Write the failing test**

Add a second test that captures the step-priority rule:

```ts
test('prefers the completion screen over the setup screen when Handy is already configured', () => {
  expect(
    resolveSetupStep({
      currentStep: 3,
      systemHealthState: 'pass',
      hasOllamaBinary: true,
      isSetupComplete: true
    })
  ).toBe(4);
});
```

**Step 2: Run test to verify it fails**

Run: `bun test src/lib/setup-flow.test.ts`
Expected: FAIL until the completion priority is implemented.

**Step 3: Write minimal implementation**

Update `+page.svelte` to call `resolveSetupStep(...)` after status/config refreshes and keep the existing timer only for the `2 -> 3` pass state.

**Step 4: Run test to verify it passes**

Run: `bun test src/lib/setup-flow.test.ts`
Expected: PASS

### Task 3: Verify integration safety

**Files:**
- Modify: `progress.md`

**Step 1: Run targeted verification**

Run: `bun test src/lib/setup-flow.test.ts && bun run check`

**Step 2: Record the result**

Add a short progress entry with the helper path, page wiring, and verification outcome.
