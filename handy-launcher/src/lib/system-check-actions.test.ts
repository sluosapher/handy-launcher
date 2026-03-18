import { describe, expect, test } from 'bun:test';

import { resolveSystemCheckActions } from './system-check-actions';

describe('system check actions', () => {
  test('keeps the continue path active for warning states', () => {
    expect(resolveSystemCheckActions('warning')).toEqual({
      primaryLabel: 'Continue anyway',
      continueDisabled: false,
      supportVisible: false,
      supportMessage: ''
    });
  });

  test('shows troubleshooting guidance when the system check fails', () => {
    expect(resolveSystemCheckActions('fail')).toEqual({
      primaryLabel: 'Continue',
      continueDisabled: true,
      supportVisible: true,
      supportMessage:
        'This machine needs manual troubleshooting before setup can continue.'
    });
  });
});
