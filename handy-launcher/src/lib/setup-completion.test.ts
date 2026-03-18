import { describe, expect, test } from 'bun:test';

import { isSetupComplete } from './setup-completion';

describe('setup completion', () => {
  test('returns true when Handy is configured for the selected local model', () => {
    expect(
      isSetupComplete(
        {
          current_provider_id: 'custom',
          configured_model: 'phi4-mini'
        },
        'phi4-mini'
      )
    ).toBe(true);
  });

  test('returns false when the provider or model does not match', () => {
    expect(
      isSetupComplete(
        {
          current_provider_id: 'openai',
          configured_model: 'phi4-mini'
        },
        'phi4-mini'
      )
    ).toBe(false);

    expect(
      isSetupComplete(
        {
          current_provider_id: 'custom',
          configured_model: 'llama3.2:1b'
        },
        'phi4-mini'
      )
    ).toBe(false);
  });

  test('returns false when config status or selected model is missing', () => {
    expect(isSetupComplete(null, 'phi4-mini')).toBe(false);
    expect(isSetupComplete({ current_provider_id: 'custom', configured_model: null }, null)).toBe(false);
  });
});
