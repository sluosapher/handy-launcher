import type { DownloadedModel } from './api';

export type ModelAvailability = {
  state: 'idle' | 'missing' | 'ready';
  selectedModelDownloaded: boolean;
  alternativeModels: string[];
};

export function evaluateModelAvailability(
  selectedModel: string | null,
  downloadedModels: DownloadedModel[]
): ModelAvailability {
  if (!selectedModel) {
    return {
      state: 'idle',
      selectedModelDownloaded: false,
      alternativeModels: []
    };
  }

  const selectedModelDownloaded = downloadedModels.some((model) => model.name === selectedModel);
  const alternativeModels = downloadedModels
    .map((model) => model.name)
    .filter((name) => name !== selectedModel);

  return {
    state: selectedModelDownloaded ? 'ready' : 'missing',
    selectedModelDownloaded,
    alternativeModels
  };
}
