export type ModelProfile = {
  id: 'light' | 'fast' | 'balanced';
  label: string;
  description: string;
  model: string;
  sizeLabel: string;
  ramRequiredGb: number;
  downloadEstimate: string;
};

export const modelProfiles: ModelProfile[] = [
  {
    id: 'light',
    label: 'Light',
    description: 'Quick results for lightweight notes and lower-memory machines.',
    model: 'llama3.2:1b',
    sizeLabel: '~1.0 GB',
    ramRequiredGb: 4,
    downloadEstimate: '~2 min'
  },
  {
    id: 'fast',
    label: 'Fast',
    description: 'Balanced speed for everyday dictation on mid-range hardware.',
    model: 'phi4-mini',
    sizeLabel: '~2.5 GB',
    ramRequiredGb: 6,
    downloadEstimate: '~6 min'
  },
  {
    id: 'balanced',
    label: 'Balanced',
    description: 'Best quality for longer transcripts and higher-memory systems.',
    model: 'qwen2.5:7b',
    sizeLabel: '~4.5 GB',
    ramRequiredGb: 8,
    downloadEstimate: '~12 min'
  }
];

export type SelectableModelProfile = ModelProfile & {
  supported: boolean;
  recommended: boolean;
};

export function getRecommendedProfileId(availableRamGb: number): ModelProfile['id'] {
  const supportedProfiles = modelProfiles.filter((profile) => availableRamGb >= profile.ramRequiredGb);
  return supportedProfiles.at(-1)?.id ?? 'light';
}

export function getSelectableProfiles(availableRamGb: number): SelectableModelProfile[] {
  const recommendedId = getRecommendedProfileId(availableRamGb);

  return modelProfiles.map((profile) => ({
    ...profile,
    supported: availableRamGb >= profile.ramRequiredGb,
    recommended: profile.id === recommendedId
  }));
}

export function findProfileByModel(modelName: string | null | undefined): ModelProfile | undefined {
  return modelProfiles.find((profile) => profile.model === modelName);
}
