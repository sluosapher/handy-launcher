import { describe, expect, test } from 'bun:test';

import { evaluateSystemHealth } from './system-health';

describe('system health evaluation', () => {
  test('reports pass when supported hardware clears recommended thresholds', () => {
    const result = evaluateSystemHealth({
      os_name: 'Windows 11',
      os_version: '24H2',
      total_ram_gb: 16,
      available_ram_gb: 12,
      total_disk_gb: 512,
      available_disk_gb: 128
    });

    expect(result.state).toBe('pass');
    expect(result.summary).toContain('ready');
    expect(result.checks.map((check) => check.state)).toEqual(['pass', 'pass', 'pass']);
  });

  test('reports warning when RAM or disk is below the recommended threshold', () => {
    const result = evaluateSystemHealth({
      os_name: 'macOS',
      os_version: '15.0',
      total_ram_gb: 8,
      available_ram_gb: 5.5,
      total_disk_gb: 256,
      available_disk_gb: 9
    });

    expect(result.state).toBe('warning');
    expect(result.checks.find((check) => check.id === 'ram')?.state).toBe('warning');
    expect(result.checks.find((check) => check.id === 'disk')?.state).toBe('warning');
  });

  test('reports fail for unsupported operating systems', () => {
    const result = evaluateSystemHealth({
      os_name: 'Linux',
      os_version: '6.8',
      total_ram_gb: 32,
      available_ram_gb: 24,
      total_disk_gb: 1024,
      available_disk_gb: 800
    });

    expect(result.state).toBe('fail');
    expect(result.checks.find((check) => check.id === 'os')?.state).toBe('fail');
    expect(result.summary).toContain('supported');
  });
});
