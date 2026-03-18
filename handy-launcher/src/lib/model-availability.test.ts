import { describe, expect, test } from 'bun:test';

import { evaluateModelAvailability } from './model-availability';

describe('model availability', () => {
  test('marks the selected model as ready when it is already downloaded', () => {
    const result = evaluateModelAvailability('phi4-mini', [
      { name: 'llama3.2:1b', size: 1_000, modified_at: null },
      { name: 'phi4-mini', size: 2_000, modified_at: null }
    ]);

    expect(result.state).toBe('ready');
    expect(result.selectedModelDownloaded).toBe(true);
    expect(result.alternativeModels).toEqual(['llama3.2:1b']);
  });

  test('marks the selected model as missing and suggests downloaded alternatives', () => {
    const result = evaluateModelAvailability('qwen2.5:7b', [
      { name: 'llama3.2:1b', size: 1_000, modified_at: null },
      { name: 'phi4-mini', size: 2_000, modified_at: null }
    ]);

    expect(result.state).toBe('missing');
    expect(result.selectedModelDownloaded).toBe(false);
    expect(result.alternativeModels).toEqual(['llama3.2:1b', 'phi4-mini']);
  });

  test('returns idle when no model is selected yet', () => {
    const result = evaluateModelAvailability(null, []);

    expect(result.state).toBe('idle');
    expect(result.selectedModelDownloaded).toBe(false);
    expect(result.alternativeModels).toEqual([]);
  });
});
