import { describe, expect, test } from 'bun:test';

import { resolveSetupSupportAction } from './setup-support-action';

describe('setup support action', () => {
  test('offers manual Ollama setup while the binary is still missing', () => {
    expect(
      resolveSetupSupportAction({
        setupStep: 3,
        hasBinary: false,
        hasActionError: false,
        systemHealthState: 'pass'
      })
    ).toEqual({
      label: 'Manual Ollama setup',
      description: 'Open the official Ollama download page if the built-in installer cannot finish.',
      action: 'ollama-download'
    });
  });

  test('prefers troubleshooting when the system check fails', () => {
    expect(
      resolveSetupSupportAction({
        setupStep: 2,
        hasBinary: false,
        hasActionError: false,
        systemHealthState: 'fail'
      })
    ).toEqual({
      label: 'Troubleshooting',
      description: 'Open the launcher data directory to inspect logs, retained backups, and installer artifacts.',
      action: 'logs'
    });
  });

  test('shows view logs on the completion screen', () => {
    expect(
      resolveSetupSupportAction({
        setupStep: 4,
        hasBinary: true,
        hasActionError: false,
        systemHealthState: 'pass'
      })
    ).toEqual({
      label: 'View logs',
      description: 'Open the launcher data directory to inspect logs, retained backups, and installer artifacts.',
      action: 'logs'
    });
  });
});
