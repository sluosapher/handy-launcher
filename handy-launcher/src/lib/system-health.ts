import type { SystemSnapshot } from './stores';

export type HealthCheckState = 'pass' | 'warning' | 'fail';

export type HealthCheck = {
  id: 'ram' | 'disk' | 'os';
  label: string;
  value: string;
  state: HealthCheckState;
  message: string;
};

export type SystemHealthResult = {
  state: HealthCheckState;
  summary: string;
  checks: HealthCheck[];
};

const RECOMMENDED_RAM_GB = 8;
const RECOMMENDED_DISK_GB = 10;

export function evaluateSystemHealth(system: SystemSnapshot): SystemHealthResult {
  const normalizedOs = system.os_name.toLowerCase();
  const osSupported = normalizedOs.includes('windows') || normalizedOs.includes('mac');

  const checks: HealthCheck[] = [
    {
      id: 'ram',
      label: 'RAM',
      value: `${system.available_ram_gb.toFixed(1)} GB available`,
      state: system.available_ram_gb >= RECOMMENDED_RAM_GB ? 'pass' : 'warning',
      message:
        system.available_ram_gb >= RECOMMENDED_RAM_GB
          ? 'Enough memory for the recommended local models.'
          : 'Lower memory detected. Use Light or Fast for smoother setup.'
    },
    {
      id: 'disk',
      label: 'Disk',
      value: `${system.available_disk_gb.toFixed(1)} GB free`,
      state: system.available_disk_gb >= RECOMMENDED_DISK_GB ? 'pass' : 'warning',
      message:
        system.available_disk_gb >= RECOMMENDED_DISK_GB
          ? 'Enough free space for Ollama and model downloads.'
          : 'Free space is tight. Prefer a smaller profile or clear disk space first.'
    },
    {
      id: 'os',
      label: 'Operating system',
      value: `${system.os_name} ${system.os_version}`,
      state: osSupported ? 'pass' : 'fail',
      message: osSupported
        ? 'Supported platform for the current launcher target.'
        : 'This launcher currently supports Windows and macOS.'
    }
  ];

  const state = checks.some((check) => check.state === 'fail')
    ? 'fail'
    : checks.some((check) => check.state === 'warning')
      ? 'warning'
      : 'pass';

  return {
    state,
    summary:
      state === 'pass'
        ? 'This machine is ready for local transcription.'
        : state === 'warning'
          ? 'Setup can continue, but the lighter profiles are a safer fit.'
          : 'This machine is not on a currently supported operating system.',
    checks
  };
}
