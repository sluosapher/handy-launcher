import type { HealthCheckState } from './system-health';

export type SystemCheckActions = {
  primaryLabel: string;
  continueDisabled: boolean;
  supportVisible: boolean;
  supportMessage: string;
};

export function resolveSystemCheckActions(
  state: HealthCheckState | null
): SystemCheckActions {
  if (state === 'fail') {
    return {
      primaryLabel: 'Continue',
      continueDisabled: true,
      supportVisible: true,
      supportMessage: 'This machine needs manual troubleshooting before setup can continue.'
    };
  }

  if (state === 'warning') {
    return {
      primaryLabel: 'Continue anyway',
      continueDisabled: false,
      supportVisible: false,
      supportMessage: ''
    };
  }

  return {
    primaryLabel: 'Continue',
    continueDisabled: false,
    supportVisible: false,
    supportMessage: ''
  };
}
