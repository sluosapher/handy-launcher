type PartialConfigStatus = {
  current_provider_id: string | null;
  configured_model: string | null;
} | null;

export function isSetupComplete(
  configStatus: PartialConfigStatus,
  selectedModel: string | null
): boolean {
  if (!configStatus || !selectedModel) {
    return false;
  }

  return (
    configStatus.current_provider_id === 'custom' &&
    configStatus.configured_model === selectedModel
  );
}
