import type { Writable } from 'svelte/store';
import { writable } from 'svelte/store';

export type SetupStep = 1 | 2 | 3 | 4;
export const setupStep = writable<SetupStep>(1);
export const ollamaStatus: Writable<Record<string, unknown> | null> = writable(null);
export const installProgress = writable<{ percent: number; status: string } | null>(null);
export const modelDownloadProgress = writable<{
  modelName: string;
  percent: number;
  status: string;
} | null>(null);
export const selectedModel = writable<string | null>(null);
export const selectedProfile = writable<'light' | 'fast' | 'balanced' | null>(null);

export type SystemSnapshot = {
  os_name: string;
  os_version: string;
  total_ram_gb: number;
  available_ram_gb: number;
  total_disk_gb: number;
  available_disk_gb: number;
};

export const systemInfo = writable<SystemSnapshot | null>(null);
