import { describe, expect, test } from 'bun:test';

import { resolveSetupStep } from './setup-flow';

describe('setup flow', () => {
  test('skips directly to setup when Ollama is already installed', () => {
    expect(
      resolveSetupStep({
        currentStep: 1,
        systemHealthState: 'warning',
        hasOllamaBinary: true,
        isOllamaRunning: false,
        holdOnSetupStep: false,
        isSetupComplete: false
      })
    ).toBe(3);
  });

  test('keeps the system check step active until the timed auto-advance runs', () => {
    expect(
      resolveSetupStep({
        currentStep: 2,
        systemHealthState: 'pass',
        hasOllamaBinary: false,
        isOllamaRunning: false,
        holdOnSetupStep: false,
        isSetupComplete: false
      })
    ).toBe(2);
  });

  test('keeps the user on setup while reconfiguring even if setup is already complete', () => {
    expect(
      resolveSetupStep({
        currentStep: 3,
        systemHealthState: 'pass',
        hasOllamaBinary: true,
        isOllamaRunning: true,
        holdOnSetupStep: true,
        isSetupComplete: true
      })
    ).toBe(3);
  });
});
