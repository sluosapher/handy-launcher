import { findProfileByModel } from './model-profiles';

export type SelectionSummary = {
  title: string;
  setupEstimate: string;
  details: Array<[string, string]>;
};

export function resolveSelectionSummary(selectedModel: string | null): SelectionSummary {
  if (!selectedModel) {
    return {
      title: 'Using: no model selected',
      setupEstimate: 'Estimated setup time: choose a profile first',
      details: []
    };
  }

  const profile = findProfileByModel(selectedModel);
  if (!profile) {
    return {
      title: `Using: ${selectedModel}`,
      setupEstimate: 'Estimated setup time: not available',
      details: [['Model', selectedModel]]
    };
  }

  return {
    title: `Using: ${profile.model}`,
    setupEstimate: `Estimated setup time: ${profile.downloadEstimate}`,
    details: [
      ['Profile', profile.label],
      ['Model', profile.model],
      ['Download', profile.sizeLabel],
      ['Memory', `${profile.ramRequiredGb} GB RAM+`]
    ]
  };
}
