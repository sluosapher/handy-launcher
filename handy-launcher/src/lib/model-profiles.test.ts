import { describe, expect, test } from 'bun:test';

import {
  getRecommendedProfileId,
  getSelectableProfiles,
  modelProfiles
} from './model-profiles';

describe('model profile selection', () => {
  test('recommends the highest supported profile for available RAM', () => {
    expect(getRecommendedProfileId(12)).toBe('balanced');
    expect(getRecommendedProfileId(6.5)).toBe('fast');
    expect(getRecommendedProfileId(4.5)).toBe('light');
  });

  test('falls back to light when no profile fully fits the machine', () => {
    expect(getRecommendedProfileId(2)).toBe('light');
  });

  test('marks only supported profiles as selectable', () => {
    const selectable = getSelectableProfiles(6).map((profile) => ({
      id: profile.id,
      supported: profile.supported
    }));

    expect(selectable).toEqual([
      { id: 'light', supported: true },
      { id: 'fast', supported: true },
      { id: 'balanced', supported: false }
    ]);
  });

  test('keeps the documented profile metadata stable', () => {
    expect(modelProfiles.map((profile) => profile.model)).toEqual([
      'llama3.2:1b',
      'phi4-mini',
      'qwen2.5:7b'
    ]);
  });
});
