import { invoke } from '@tauri-apps/api/core';

export const checkOllamaStatus = () => invoke('check_ollama_status');
export const installOllama = () =>
  invoke<{ percent: number; status: string }>('install_ollama');
export const getOllamaBinaryInfo = () => invoke('ollama_binary_info');
export const configureHandyWithOllama = (model: string, port: number) =>
  invoke('configure_handy_with_ollama', { model_name: model, port });
export const getHandyConfig = () => invoke('get_handy_config');
export type HandyConfigStatus = {
  config_path: string | null;
  latest_backup_path: string | null;
  config_exists: boolean;
  handy_running: boolean;
  current_provider_id: string | null;
  configured_model: string | null;
  selected_prompt_id: string | null;
};
export const getHandyConfigStatus = () => invoke<HandyConfigStatus>('get_handy_config_status');
export const getSystemInfo = () => invoke('system_info');
export const openLauncherDataDir = () => invoke('open_launcher_data_dir');
export const openOllamaDownloadPage = () => invoke('open_ollama_download_page');
export const openHandyApp = () => invoke('open_handy_app');
export const openHandyDownloadPage = () => invoke('open_handy_download_page');
export type DebugSnapshot = {
  data_dir: string | null;
  log_path: string | null;
  recent_logs: string[];
};
export const getLauncherDebugSnapshot = (lineLimit?: number) =>
  invoke<DebugSnapshot>('get_launcher_debug_snapshot', { line_limit: lineLimit });

export type StartOllamaResult = { port: number; version: string };

export const startOllamaServer = (portHint?: number) =>
  invoke<StartOllamaResult>('start_ollama_server', { port_hint: portHint });

export const verifyOllamaServer = (port: number) =>
  invoke('verify_ollama_server', { port });

export type DownloadedModel = {
  name: string;
  size: number;
  modified_at?: string | null;
};

export const listOllamaModels = (portHint?: number) =>
  invoke<DownloadedModel[]>('list_ollama_models', { port_hint: portHint });

export const downloadOllamaModel = (modelName: string, portHint?: number) =>
  invoke<{ percent: number; status: string }>('download_ollama_model', {
    model_name: modelName,
    port_hint: portHint
  });

export const stopOllamaServer = () => invoke('stop_ollama_server');

export type ModelDownloadProgress = {
  model_name: string;
  progress: {
    percent: number;
    status: string;
  };
};
