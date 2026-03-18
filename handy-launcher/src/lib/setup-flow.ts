import type { SetupStep } from './stores';

export type SetupFlowSnapshot = {
  currentStep: SetupStep;
  systemHealthState: 'pass' | 'warning' | 'fail' | null;
  hasOllamaBinary: boolean;
  isOllamaRunning: boolean;
  holdOnSetupStep: boolean;
  isSetupComplete: boolean;
};

export function resolveSetupStep(snapshot: SetupFlowSnapshot): SetupStep {
  if (snapshot.holdOnSetupStep) {
    return 3;
  }

  if (snapshot.isSetupComplete) {
    return 4;
  }

  if (snapshot.hasOllamaBinary || snapshot.isOllamaRunning) {
    return 3;
  }

  return snapshot.currentStep;
}
