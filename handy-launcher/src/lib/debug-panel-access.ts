export type DebugPanelUnlockState = {
  tapCount: number;
  unlocked: boolean;
  windowStartedAt: number | null;
};

const UNLOCK_TAP_COUNT = 5;
const UNLOCK_WINDOW_MS = 5_000;

export function advanceDebugPanelUnlock(
  state: DebugPanelUnlockState,
  timestamp: number
): DebugPanelUnlockState {
  const windowExpired =
    state.windowStartedAt === null || timestamp - state.windowStartedAt > UNLOCK_WINDOW_MS;

  const tapCount = windowExpired ? 1 : state.tapCount + 1;
  const windowStartedAt = windowExpired ? timestamp : state.windowStartedAt;

  return {
    tapCount,
    windowStartedAt,
    unlocked: tapCount >= UNLOCK_TAP_COUNT
  };
}
