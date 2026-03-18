import { describe, expect, test } from 'bun:test';

import { advanceDebugPanelUnlock } from './debug-panel-access';

describe('debug panel access', () => {
  test('unlocks after five taps inside the time window', () => {
    let state = { tapCount: 0, unlocked: false, windowStartedAt: null as number | null };

    state = advanceDebugPanelUnlock(state, 1_000);
    state = advanceDebugPanelUnlock(state, 2_000);
    state = advanceDebugPanelUnlock(state, 3_000);
    state = advanceDebugPanelUnlock(state, 4_000);
    state = advanceDebugPanelUnlock(state, 5_000);

    expect(state.unlocked).toBe(true);
    expect(state.tapCount).toBe(5);
  });

  test('resets the sequence when the tap window expires', () => {
    let state = { tapCount: 0, unlocked: false, windowStartedAt: null as number | null };

    state = advanceDebugPanelUnlock(state, 1_000);
    state = advanceDebugPanelUnlock(state, 2_000);
    state = advanceDebugPanelUnlock(state, 10_500);

    expect(state.unlocked).toBe(false);
    expect(state.tapCount).toBe(1);
    expect(state.windowStartedAt).toBe(10_500);
  });
});
