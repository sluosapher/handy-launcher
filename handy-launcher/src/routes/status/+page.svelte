<script>
  import { onDestroy, onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import {
    checkOllamaStatus,
    getLauncherDebugSnapshot,
    getHandyConfigStatus,
    listOllamaModels,
    openLauncherDataDir,
    startOllamaServer,
    stopOllamaServer,
    verifyOllamaServer
  } from '$lib/api';
  import { advanceDebugPanelUnlock } from '$lib/debug-panel-access';
  import DebugPanel from '$lib/components/DebugPanel.svelte';
  import { installProgress, modelDownloadProgress, ollamaStatus, setupStep, systemInfo } from '$lib/stores';
  import ActionButton from '$lib/components/ActionButton.svelte';
  import ProgressBar from '$lib/components/ProgressBar.svelte';
  import StatusCard from '$lib/components/StatusCard.svelte';

  const POLL_INTERVAL_MS = 5000;

  let loading = true;
  let refreshing = false;
  let actionError = '';
  let testResult = '';
  let startLoading = false;
  let stopLoading = false;
  let testLoading = false;
  let currentPort = 11434;
  let isRunning = false;
  let hasBinary = false;
  let downloadedModels = [];
  let configStatus = null;
  let lastRefresh = null;
  let runningSince = null;
  let statusLabel = 'Checking Ollama...';
  let statusDetails = '';
  let statusTone = 'neutral';
  let pollHandle = null;
  let memoryTone = 'neutral';
  let debugLoading = false;
  let debugSnapshot = null;
  let debugUnlock = { tapCount: 0, unlocked: false, windowStartedAt: null };

  function formatDuration(ms) {
    if (ms < 0) {
      return '00:00';
    }
    const totalSeconds = Math.floor(ms / 1000);
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = totalSeconds % 60;

    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    }
    if (minutes > 0) {
      return `${minutes}m ${seconds}s`;
    }

    return `${seconds}s`;
  }

  function deriveStatusData(payload) {
    const fallback = {
      label: 'Checking Ollama...',
      details: '',
      hasBinary: false,
      running: false,
      port: currentPort,
      tone: 'neutral'
    };

    if (!payload) {
      return fallback;
    }

    if ('NotInstalled' in payload) {
      return {
        ...fallback,
        label: 'Ollama not installed',
        details: 'Install Ollama from the setup tab to continue',
        tone: 'error'
      };
    }

    if ('Running' in payload) {
      const running = payload.Running ?? {};
      const port = running.port ?? currentPort;
      const version = running.version ?? 'unknown version';
      return {
        label: 'Ollama running',
        details: `Port ${port} · ${version}`,
        hasBinary: true,
        running: true,
        port,
        tone: 'success'
      };
    }

    if ('Ready' in payload) {
      const ready = payload.Ready ?? {};
      return {
        label: 'Ollama ready',
        details: ready.version ? `Version ${ready.version}` : 'Ready to start',
        hasBinary: true,
        running: false,
        port: currentPort,
        tone: 'warning'
      };
    }

    if ('Installing' in payload) {
      const installing = payload.Installing ?? {};
      const progress = installing.progress ?? {};
      const percentText =
        typeof progress.percent === 'number' ? `${progress.percent}% · ` : '';
      return {
        label: 'Installing Ollama',
        details: `${percentText}${progress.status ?? 'preparing'}`,
        hasBinary: true,
        running: false,
        port: currentPort,
        tone: 'warning'
      };
    }

    if ('Error' in payload) {
      const error = payload.Error ?? {};
      return {
        ...fallback,
        label: 'Ollama error',
        details: error.message ?? 'Check logs for details',
        tone: 'error'
      };
    }

    const [pendingKey, pendingValue] = Object.entries(payload)[0] ?? ['', {}];
    return {
      label: pendingKey || 'Ollama',
      details: pendingValue ? JSON.stringify(pendingValue) : '',
      hasBinary: false,
      running: false,
      port: currentPort,
      tone: 'neutral'
    };
  }

  function updateStatus(payload) {
    const state = deriveStatusData(payload);
    statusLabel = state.label;
    statusDetails = state.details;
    statusTone = state.tone;
    hasBinary = state.hasBinary;
    currentPort = state.port;
    ollamaStatus.set(payload);

    if (state.running) {
      isRunning = true;
      if (!runningSince) {
        runningSince = Date.now();
      }
    } else {
      isRunning = false;
      runningSince = null;
    }
  }

  async function refreshStatus() {
    if (refreshing) {
      return;
    }

    refreshing = true;
    actionError = '';
    try {
      const payload = await checkOllamaStatus();
      updateStatus(payload);

      if (payload && 'Running' in payload) {
        downloadedModels = await listOllamaModels(currentPort);
      } else {
        downloadedModels = [];
      }

      configStatus = await getHandyConfigStatus();
    } catch (err) {
      actionError = typeof err === 'string' ? err : 'Unable to refresh Ollama status';
    } finally {
      lastRefresh = new Date();
      loading = false;
      refreshing = false;
    }
  }

  async function startServer() {
    if (startLoading || isRunning) {
      return;
    }

    startLoading = true;
    actionError = '';
    try {
      await startOllamaServer();
      await refreshStatus();
    } catch (err) {
      actionError = typeof err === 'string' ? err : 'Unable to start Ollama';
    } finally {
      startLoading = false;
    }
  }

  async function stopServer() {
    if (stopLoading || !isRunning) {
      return;
    }

    stopLoading = true;
    actionError = '';
    try {
      await stopOllamaServer();
      await refreshStatus();
    } catch (err) {
      actionError = typeof err === 'string' ? err : 'Unable to stop Ollama';
    } finally {
      stopLoading = false;
    }
  }

  async function testConnection() {
    if (testLoading || !isRunning) {
      return;
    }

    testLoading = true;
    actionError = '';
    testResult = '';
    try {
      const version = await verifyOllamaServer(currentPort);
      testResult = `Connection OK · ${version}`;
    } catch (err) {
      const message = typeof err === 'string' ? err : 'Verify failed';
      testResult = `Connection failed`;
      actionError = message;
    } finally {
      testLoading = false;
    }
  }

  async function openLogs() {
    try {
      await openLauncherDataDir();
    } catch (err) {
      actionError = typeof err === 'string' ? err : 'Unable to open logs';
    }
  }

  async function refreshDebugPanel() {
    if (debugLoading || !debugUnlock.unlocked) {
      return;
    }

    debugLoading = true;
    try {
      debugSnapshot = await getLauncherDebugSnapshot();
    } catch (err) {
      actionError = typeof err === 'string' ? err : 'Unable to load debug snapshot';
    } finally {
      debugLoading = false;
    }
  }

  function unlockDebugPanel() {
    debugUnlock = advanceDebugPanelUnlock(debugUnlock, Date.now());
    if (debugUnlock.unlocked) {
      refreshDebugPanel();
    }
  }

  function goToSetup() {
    setupStep.set(3);
    goto('/');
  }

  onMount(() => {
    refreshStatus();
    pollHandle = setInterval(refreshStatus, POLL_INTERVAL_MS);

    return () => {
      if (pollHandle) {
        clearInterval(pollHandle);
      }
    };
  });

  onDestroy(() => {
    if (pollHandle) {
      clearInterval(pollHandle);
    }
  });

  $: uptimeValue = isRunning
    ? formatDuration(Date.now() - (runningSince ?? Date.now()))
    : 'Stopped';

  $: memoryUsage = $systemInfo
    ? `${($systemInfo.total_ram_gb - $systemInfo.available_ram_gb).toFixed(1)} / ${$systemInfo.total_ram_gb.toFixed(1)} GB used`
    : 'Gathering system data';

  $: lastActivity = lastRefresh ? lastRefresh.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) : 'Pending';

  $: memoryTone = $systemInfo && $systemInfo.available_ram_gb < 2 ? 'warning' : 'neutral';
</script>

<svelte:head>
  <title>Handy Launcher Status</title>
</svelte:head>

<main class="min-h-screen bg-surface text-primary">
  <section class="mx-auto max-w-5xl space-y-8 px-6 py-16">
    <header class="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
      <div>
        <button
          class="text-left text-xs uppercase tracking-[0.3em] text-secondary"
          on:click={unlockDebugPanel}
        >
          Handy launcher
        </button>
        <h1 class="text-3xl font-semibold">Status dashboard</h1>
        <p class="text-sm text-secondary">Monitor Ollama, test the connection, and open logs.</p>
      </div>

      <div class="flex flex-wrap gap-3">
        <ActionButton label="Back to setup" variant="ghost" on:click={goToSetup} />
        <ActionButton label="View logs" variant="ghost" on:click={openLogs} />
      </div>
    </header>

    <div class="rounded-2xl border border-white/10 bg-card p-6 shadow-lg">
      <div class="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
        <div>
          <p class="text-xs uppercase tracking-[0.3em] text-secondary">Current status</p>
          <h2 class="text-2xl font-semibold">{statusLabel}</h2>
          <p class="text-sm text-secondary">{statusDetails}</p>
        </div>
        <div class="flex flex-wrap gap-2">
          <span
            class={`rounded-full px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.3em] ${
              statusTone === 'success'
                ? 'bg-success/20 text-success'
                : statusTone === 'warning'
                ? 'bg-warning/20 text-warning'
                : statusTone === 'error'
                ? 'bg-error/20 text-error'
                : 'bg-white/10 text-white'
            }`}
          >
            {statusTone}
          </span>
        </div>
      </div>

      {#if $installProgress}
        <div class="mt-4 rounded-xl border border-white/10 bg-white/5 p-4">
          <p class="text-xs uppercase tracking-[0.3em] text-secondary">Install progress</p>
          <p class="text-sm text-primary">
            {$installProgress.percent ?? 0}% · {$installProgress.status}
          </p>
          <div class="mt-3 h-2 overflow-hidden rounded-full bg-white/10">
            <div
              class="h-full rounded-full bg-accent transition-all duration-300"
              style={`width: ${Math.max(8, $installProgress.percent ?? 0)}%`}
            ></div>
          </div>
        </div>
      {/if}

      {#if $modelDownloadProgress}
        <div class="mt-4">
          <ProgressBar
            label={`Model download: ${$modelDownloadProgress.modelName}`}
            percent={$modelDownloadProgress.percent}
            status={$modelDownloadProgress.status}
            minimumVisiblePercent={8}
          />
        </div>
      {/if}

      {#if actionError}
        <p class="mt-4 rounded-xl border border-error/40 bg-error/10 px-4 py-3 text-xs text-error">
          {actionError}
        </p>
      {/if}

      <div class="mt-6 grid gap-3 md:grid-cols-3">
        <StatusCard label="Uptime" value={uptimeValue} helper={runningSince ? `Since ${new Date(runningSince).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}` : 'Not running'} tone={isRunning ? 'success' : 'neutral'} />
        <StatusCard label="Memory usage" value={memoryUsage} helper={$systemInfo ? `${$systemInfo.available_ram_gb.toFixed(1)} GB available` : ''} tone={memoryTone} />
        <StatusCard label="Last activity" value={lastActivity} helper={`Refreshed ${lastRefresh ? lastRefresh.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' }) : 'soon'}`} tone={statusTone} />
      </div>

      <div class="mt-6 grid gap-4 md:grid-cols-[1.2fr_0.8fr]">
        <div class="rounded-2xl border border-white/10 bg-white/5 p-5">
          <p class="text-xs uppercase tracking-[0.3em] text-secondary">Model info</p>
          <p class="mt-2 text-lg font-semibold text-primary">
            {configStatus?.configured_model ?? 'No model configured'}
          </p>
          <p class="text-xs text-secondary">
            Provider: {configStatus?.current_provider_id ?? 'custom'}
          </p>
          <p class="text-xs text-secondary">Server port: {currentPort}</p>
          <p class="text-xs text-secondary">
            {downloadedModels.length} downloaded model{downloadedModels.length === 1 ? '' : 's'}
          </p>
        </div>

        <div class="rounded-2xl border border-white/10 bg-white/5 p-5">
          <p class="text-xs uppercase tracking-[0.3em] text-secondary">Downloaded models</p>
          <div class="mt-3 flex flex-wrap gap-2">
            {#if downloadedModels.length === 0}
              <p class="text-xs text-secondary">No models detected</p>
            {:else}
              {#each downloadedModels as model}
                <span class="rounded-full border border-white/10 bg-white/10 px-3 py-1 text-[11px] uppercase tracking-[0.3em]">
                  {model.name}
                </span>
              {/each}
            {/if}
          </div>
        </div>
      </div>

      <div class="mt-6 flex flex-wrap gap-3">
        {#if isRunning}
          <ActionButton
            label={stopLoading ? 'Stopping...' : 'Stop Ollama'}
            variant="danger"
            loading={stopLoading}
            on:click={stopServer}
          />
        {:else if hasBinary}
          <ActionButton
            label={startLoading ? 'Starting...' : 'Start Ollama'}
            variant="success"
            loading={startLoading}
            on:click={startServer}
          />
        {:else}
          <ActionButton label="Install Ollama first" variant="ghost" disabled />
        {/if}
        <ActionButton
          label={testLoading ? 'Testing...' : 'Test connection'}
          variant="ghost"
          loading={testLoading}
          on:click={testConnection}
          disabled={!isRunning}
        />
        <ActionButton label="Switch model" variant="ghost" on:click={goToSetup} />
      </div>

      {#if testResult}
        <p class={`mt-3 text-xs font-semibold ${testResult.startsWith('Connection OK') ? 'text-success' : 'text-warning'}`}>
          {testResult}
        </p>
      {/if}
    </div>

    {#if debugUnlock.unlocked}
      <div class="space-y-3">
        <div class="flex justify-end">
          <ActionButton
            label={debugLoading ? 'Refreshing debug data...' : 'Refresh debug data'}
            variant="ghost"
            loading={debugLoading}
            on:click={refreshDebugPanel}
          />
        </div>
        <DebugPanel
          loading={debugLoading}
          snapshot={debugSnapshot}
          actionError={actionError}
          systemInfo={$systemInfo}
          ollamaStatus={$ollamaStatus}
          configStatus={configStatus}
        />
      </div>
    {/if}
  </section>
</main>
