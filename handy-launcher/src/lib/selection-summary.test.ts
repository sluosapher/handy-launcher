import { describe, expect, test } from 'bun:test';

import { resolveSelectionSummary } from './selection-summary';

describe('selection summary', () => {
  test('returns the selected profile metadata and a setup estimate', () => {
    expect(resolveSelectionSummary('phi4-mini')).toEqual({
      title: 'Using: phi4-mini',
      setupEstimate: 'Estimated setup time: ~6 min',
      details: [
        ['Profile', 'Fast'],
        ['Model', 'phi4-mini'],
        ['Download', '~2.5 GB'],
        ['Memory', '6 GB RAM+']
      ]
    });
  });

  test('returns a neutral placeholder when no model is selected', () => {
    expect(resolveSelectionSummary(null)).toEqual({
      title: 'Using: no model selected',
      setupEstimate: 'Estimated setup time: choose a profile first',
      details: []
    });
  });
});
