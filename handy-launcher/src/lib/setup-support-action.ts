import type { SetupStep } from './stores';

export type SetupSupportAction = {
  label: string;
  description: string;
  action: 'logs' | 'ollama-download';
};

export type SetupSupportSnapshot = {
  setupStep: SetupStep;
  hasBinary: boolean;
  hasActionError: boolean;
  systemHealthState: 'pass' | 'warning' | 'fail' | null;
};

const LOGS_DESCRIPTION =
  'Open the launcher data directory to inspect logs, retained backups, and installer artifacts.';

export function resolveSetupSupportAction(
  snapshot: SetupSupportSnapshot
): SetupSupportAction {
  if (
    snapshot.setupStep === 4 ||
    snapshot.setupStep === 2 && snapshot.systemHealthState === 'fail' ||
    snapshot.hasActionError ||
    snapshot.hasBinary
  ) {
    return {
      label: snapshot.setupStep === 4 ? 'View logs' : 'Troubleshooting',
      description: LOGS_DESCRIPTION,
      action: 'logs'
    };
  }

  return {
    label: 'Manual Ollama setup',
    description:
      'Open the official Ollama download page if the built-in installer cannot finish.',
    action: 'ollama-download'
  };
}
