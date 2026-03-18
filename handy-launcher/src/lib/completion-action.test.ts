import { describe, expect, test } from 'bun:test';

import { resolveCompletionAction } from './completion-action';

describe('completion action', () => {
  test('prefers opening Handy when the app is installed and not already running', () => {
    expect(
      resolveCompletionAction({
        handyInstalled: true,
        handyRunning: false
      })
    ).toEqual({
      label: 'Open Handy',
      disabled: false,
      action: 'open'
    });
  });

  test('disables the button when Handy is already running', () => {
    expect(
      resolveCompletionAction({
        handyInstalled: true,
        handyRunning: true
      })
    ).toEqual({
      label: 'Handy is already open',
      disabled: true,
      action: 'open'
    });
  });
});
