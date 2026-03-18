<script>
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { goto } from '$app/navigation';
  import {
    checkOllamaStatus,
    configureHandyWithOllama,
    downloadOllamaModel,
    getHandyConfigStatus,
    getSystemInfo,
    installOllama,
    listOllamaModels,
    openHandyApp,
    openLauncherDataDir,
    openHandyDownloadPage,
    openOllamaDownloadPage,
    startOllamaServer,
    stopOllamaServer
  } from '$lib/api';
  import { resolveCompletionAction } from '$lib/completion-action';
  import {
    installProgress,
    modelDownloadProgress,
    ollamaStatus,
    selectedModel,
    selectedProfile,
    setupStep,
    systemInfo
  } from '$lib/stores';
  import { evaluateModelAvailability } from '$lib/model-availability';
  import { findProfileByModel, getSelectableProfiles } from '$lib/model-profiles';
  import { resolveSelectionSummary } from '$lib/selection-summary';
  import { resolveSystemCheckActions } from '$lib/system-check-actions';
  import { isSetupComplete } from '$lib/setup-completion';
  import { resolveSetupStep } from '$lib/setup-flow';
  import { resolveSetupSupportAction } from '$lib/setup-support-action';
  import { evaluateSystemHealth } from '$lib/system-health';
  import ProgressBar from '$lib/components/ProgressBar.svelte';

  let loading = true;
  let errorMessage = '';
  let actionError = '';
  let installLoading = false;
  let startLoading = false;
  let stopLoading = false;
  let configureLoading = false;
  let statusLabel = 'Checking Ollama...';
  let statusDetails = '';
  let hasBinary = false;
  let isRunning = false;
  let currentPort = 11434;
  let configureMessage = '';
  let handyConfigStatus = null;
  let availableProfiles = getSelectableProfiles(0);
  let recommendedProfileId = 'light';
  let systemHealth = null;
  let systemAdvanceTimer = null;
  let downloadedModels = [];
  let downloadLoading = false;
  let downloadMessage = '';
  let modelAvailability = evaluateModelAvailability(null, []);
  let completionLoading = false;
  let holdOnSetupStep = false;
  let showTechnicalDetails = false;
  let completionAction = resolveCompletionAction({
    handyInstalled: false,
    handyRunning: false
  });
  let supportAction = resolveSetupSupportAction({
    setupStep: 1,
    hasBinary: false,
    hasActionError: false,
    systemHealthState: null
  });
  let systemCheckActions = resolveSystemCheckActions(null);
  let selectionSummary = resolveSelectionSummary(null);
  let stopListeningToDownloadProgress = null;

  const stepLabels = [
    { step: 1, label: 'Welcome' },
    { step: 2, label: 'System check' },
    { step: 3, label: 'Setup' },
    { step: 4, label: 'Done' }
  ];

  function syncProfileSelection(modelName) {
    const profile = findProfileByModel(modelName);
    if (profile) {
      selectedProfile.set(profile.id);
    }
  }

  function chooseProfile(profile) {
    if (!profile.supported) {
      return;
    }

    selectedProfile.set(profile.id);
    selectedModel.set(profile.model);
    showTechnicalDetails = false;
    configureMessage = '';
  }

  async function refreshState() {
    loading = true;
    errorMessage = '';
    let nextHasOllamaBinary = false;
    let nextIsOllamaRunning = false;
    let nextSetupComplete = false;

    if (!installLoading) {
      actionError = '';
    }

    try {
      const system = await getSystemInfo();
      systemInfo.set(system);
      systemHealth = evaluateSystemHealth(system);
      availableProfiles = getSelectableProfiles(system.available_ram_gb);
      recommendedProfileId = availableProfiles.find((profile) => profile.recommended)?.id ?? 'light';
      selectedProfile.update((current) => current ?? recommendedProfileId);
      selectedModel.update((model) => model ?? availableProfiles.find((profile) => profile.id === recommendedProfileId)?.model ?? 'llama3.2:1b');
    } catch (err) {
      errorMessage = typeof err === 'string' ? err : 'Unable to read system info';
    }

    try {
      const status = await checkOllamaStatus();
      ollamaStatus.set(status);
      if (!('NotInstalled' in status) && !('Installing' in status)) {
        installProgress.set(null);
      }

      nextHasOllamaBinary = 'Ready' in status || 'Running' in status;
      nextIsOllamaRunning = 'Running' in status;

      if ('Running' in status) {
        currentPort = status.Running?.port ?? currentPort;
        downloadedModels = await listOllamaModels(currentPort);
      } else {
        downloadedModels = [];
      }
    } catch (err) {
      errorMessage = typeof err === 'string' ? err : 'Ollama status check failed';
    }

    try {
      handyConfigStatus = await getHandyConfigStatus();
      nextSetupComplete = isSetupComplete(handyConfigStatus, $selectedModel);
    } catch (err) {
      errorMessage = typeof err === 'string' ? err : 'Unable to inspect Handy configuration';
    } finally {
      const nextStep = resolveSetupStep({
        currentStep: $setupStep,
        systemHealthState: systemHealth?.state ?? null,
        hasOllamaBinary: nextHasOllamaBinary,
        isOllamaRunning: nextIsOllamaRunning,
        holdOnSetupStep,
        isSetupComplete: nextSetupComplete
      });

      if (nextStep !== $setupStep) {
        setupStep.set(nextStep);
      }

      loading = false;
    }
  }

  async function installBinary() {
    if (installLoading || startLoading || stopLoading) {
      return;
    }

    installLoading = true;
    actionError = '';

    const initialProgress = {
      percent: 5,
      status: 'preparing installer'
    };
    installProgress.set(initialProgress);
    ollamaStatus.set({
      Installing: {
        progress: initialProgress
      }
    });

    try {
      const progress = await installOllama();
      installProgress.set(progress);
      ollamaStatus.set({
        Installing: {
          progress
        }
      });
      await refreshState();
    } catch (err) {
      installProgress.set(null);
      actionError = typeof err === 'string' ? err : 'Unable to install Ollama';
      ollamaStatus.set({
        Error: {
          message: actionError
        }
      });
    } finally {
      installLoading = false;
    }
  }

  async function startServer() {
    if (installLoading || startLoading || stopLoading) {
      return;
    }

    startLoading = true;
    actionError = '';

    try {
      await startOllamaServer();
      await refreshState();
    } catch (err) {
      actionError = typeof err === 'string' ? err : 'Unable to start Ollama';
    } finally {
      startLoading = false;
    }
  }

  async function stopServer() {
    if (installLoading || startLoading || stopLoading) {
      return;
    }

    stopLoading = true;
    actionError = '';

    try {
      await stopOllamaServer();
      await refreshState();
    } catch (err) {
      actionError = typeof err === 'string' ? err : 'Unable to stop Ollama';
    } finally {
      stopLoading = false;
    }
  }

  async function configureHandy() {
    if (
      installLoading ||
      startLoading ||
      stopLoading ||
      configureLoading ||
      !isRunning ||
      !$selectedModel
    ) {
      return;
    }

    configureLoading = true;
    actionError = '';
    configureMessage = '';

    try {
      holdOnSetupStep = false;
      await configureHandyWithOllama($selectedModel, currentPort);
      configureMessage = `Handy configured for ${$selectedModel} on port ${currentPort}.`;
      await refreshState();
      setupStep.set(4);
    } catch (err) {
      actionError = typeof err === 'string' ? err : 'Unable to configure Handy';
    } finally {
      configureLoading = false;
    }
  }

  async function downloadSelectedModel() {
    if (!isRunning || !$selectedModel || downloadLoading) {
      return;
    }

    downloadLoading = true;
    actionError = '';
    downloadMessage = '';

    try {
      modelDownloadProgress.set({
        modelName: $selectedModel,
        percent: 0,
        status: 'requesting download'
      });
      const progress = await downloadOllamaModel($selectedModel, currentPort);
      modelDownloadProgress.set({
        modelName: $selectedModel,
        percent: progress.percent,
        status: progress.status
      });
      downloadMessage = `${$selectedModel} downloaded. ${progress.status}`;
      downloadedModels = await listOllamaModels(currentPort);
    } catch (err) {
      modelDownloadProgress.set(null);
      actionError = typeof err === 'string' ? err : 'Unable to download the selected model';
    } finally {
      if (!actionError) {
        setTimeout(() => modelDownloadProgress.set(null), 1200);
      }
      downloadLoading = false;
    }
  }

  async function runCompletionAction() {
    if (completionLoading || completionAction.disabled) {
      return;
    }

    completionLoading = true;
    actionError = '';

    try {
      if (completionAction.action === 'open') {
        await openHandyApp();
      } else {
        await openHandyDownloadPage();
      }
    } catch (err) {
      actionError = typeof err === 'string' ? err : 'Unable to open Handy';
    } finally {
      completionLoading = false;
    }
  }

  async function runSupportAction() {
    actionError = '';

    try {
      if (supportAction.action === 'logs') {
        await openLauncherDataDir();
      } else {
        await openOllamaDownloadPage();
      }
    } catch (err) {
      actionError =
        typeof err === 'string'
          ? err
          : supportAction.action === 'logs'
          ? 'Unable to open logs'
          : 'Unable to open the Ollama download page';
    }
  }

  function beginSetup() {
    setupStep.set(2);
  }

  function continueFromSystemCheck() {
    if (systemHealth?.state === 'fail') {
      return;
    }

    setupStep.set(3);
  }

  function reconfigureSetup() {
    holdOnSetupStep = true;
    setupStep.set(3);
  }

  onMount(() => {
    selectedProfile.update((profile) => profile ?? 'light');
    selectedModel.update((model) => model ?? 'llama3.2:1b');
    refreshState();
    listen('ollama-model-download-progress', (event) => {
      modelDownloadProgress.set({
        modelName: event.payload.model_name,
        percent: event.payload.progress.percent,
        status: event.payload.progress.status
      });
    }).then((unlisten) => {
      stopListeningToDownloadProgress = unlisten;
    });

    return () => {
      if (systemAdvanceTimer) {
        clearTimeout(systemAdvanceTimer);
      }
      if (stopListeningToDownloadProgress) {
        stopListeningToDownloadProgress();
      }
    };
  });

  $: syncProfileSelection($selectedModel);
  $: modelAvailability = evaluateModelAvailability($selectedModel, downloadedModels);
  $: completionAction = resolveCompletionAction({
    handyInstalled: handyConfigStatus?.config_exists ?? false,
    handyRunning: handyConfigStatus?.handy_running ?? false
  });
  $: selectionSummary = resolveSelectionSummary($selectedModel);
  $: systemCheckActions = resolveSystemCheckActions(systemHealth?.state ?? null);
  $: supportAction = resolveSetupSupportAction({
    setupStep: $setupStep,
    hasBinary,
    hasActionError: Boolean(actionError),
    systemHealthState: systemHealth?.state ?? null
  });

  $: {
    if (systemAdvanceTimer) {
      clearTimeout(systemAdvanceTimer);
      systemAdvanceTimer = null;
    }

    if ($setupStep === 2 && systemHealth?.state === 'pass') {
      systemAdvanceTimer = setTimeout(() => {
        setupStep.set(3);
      }, 500);
    }
  }

  $: {
    const payload = $ollamaStatus;
    const progressState = $installProgress;
    hasBinary = false;
    isRunning = false;

    if (installLoading && progressState) {
      statusLabel = 'Installing Ollama';
      statusDetails = `${progressState.percent ?? 0}% | ${progressState.status ?? 'preparing'}`;
    } else if (!payload) {
      statusLabel = 'Checking Ollama...';
      statusDetails = '';
    } else if ('NotInstalled' in payload) {
      statusLabel = 'Ollama not installed';
      statusDetails = 'Download and install Ollama to continue';
    } else if ('Ready' in payload) {
      hasBinary = true;
      const ready = payload.Ready;
      statusLabel = 'Ollama ready';
      statusDetails = ready?.version ? `Version ${ready.version}` : 'Version unknown';
    } else if ('Running' in payload) {
      hasBinary = true;
      isRunning = true;
      const running = payload.Running;
      currentPort = running?.port ?? 11434;
      statusLabel = 'Ollama running';
      statusDetails = `Port ${currentPort} | ${running?.version ?? 'unknown version'}`;
    } else if ('Installing' in payload) {
      const installing = payload.Installing;
      statusLabel = 'Installing Ollama';
      statusDetails = `${installing.progress.percent ?? 0}% | ${installing.progress.status ?? 'preparing'}`;
    } else if ('Error' in payload) {
      const details = payload.Error;
      statusLabel = 'Ollama error';
      statusDetails = details?.message ?? 'Check logs for details';
    } else {
      const [pending] = Object.entries(payload);
      statusLabel = pending?.[0] ?? 'Ollama';
      statusDetails = JSON.stringify(pending?.[1] ?? {});
    }
  }
</script>

<svelte:head>
  <title>Handy Launcher Setup</title>
  <meta
    name="description"
    content="Prepare your machine for local transcription with the Handy Launcher setup wizard."
  />
</svelte:head>

<main class="min-h-screen bg-surface text-primary">
  <section class="mx-auto max-w-4xl space-y-10 px-6 py-16">
    <div class="flex flex-wrap gap-3 text-xs uppercase tracking-[0.24em] text-secondary">
      {#each stepLabels as item}
        <div
          class:step-active={$setupStep === item.step}
          class="rounded-full border border-white/10 px-4 py-2"
        >
          {item.step}. {item.label}
        </div>
      {/each}
    </div>

    <div class="space-y-6 rounded-2xl border border-white/10 bg-card p-8 shadow-lg">
      <div class="flex flex-col gap-2 md:flex-row md:items-center md:justify-between">
        <div>
          <h1 class="text-3xl font-semibold">Handy Launcher</h1>
          <p class="text-sm uppercase tracking-wide text-secondary">Local transcription setup</p>
        </div>
      <div class="flex gap-3">
        <button
          class="rounded-lg bg-accent px-6 py-3 font-medium text-white disabled:opacity-50"
          on:click={refreshState}
          disabled={loading || installLoading}
        >
          {loading ? 'Refreshing...' : 'Refresh status'}
        </button>
        <button
          class="rounded-lg border border-white/20 px-6 py-3 text-white"
          on:click={runSupportAction}
        >
          {supportAction.label}
        </button>
        <button
          class="rounded-lg border border-white/20 px-6 py-3 text-white"
          on:click={() => goto('/status')}
        >
          Status dashboard
        </button>
      </div>
      </div>

      <p class="text-base text-secondary">
        Install and manage Ollama locally so Handy can run without sending voice data to the cloud. This guide
        checks your hardware, installs Ollama, and configures Handy for local inference.
      </p>

      {#if errorMessage}
        <p class="text-sm text-error">{errorMessage}</p>
      {/if}

      {#if $setupStep === 1}
        <div class="rounded-2xl border border-white/10 bg-white/5 p-6">
          <p class="text-sm uppercase tracking-wide text-secondary">Step 1</p>
          <h2 class="mt-2 text-2xl font-semibold text-primary">Enable local transcription</h2>
          <p class="mt-3 max-w-2xl text-sm text-secondary">
            The launcher will check your machine, prepare Ollama locally, and update Handy so voice
            data stays on-device.
          </p>

          <div class="mt-6 flex flex-wrap gap-3">
            <button class="rounded-lg bg-accent px-6 py-3 font-medium text-white" on:click={beginSetup}>
              Get started
            </button>
            <button class="rounded-lg border border-white/20 px-6 py-3 text-white">Learn more</button>
          </div>
        </div>
      {:else if $setupStep === 2}
        <div class="rounded-2xl border border-white/10 bg-white/5 p-6">
          <p class="text-sm uppercase tracking-wide text-secondary">Step 2</p>
          <div class="mt-2 flex flex-wrap items-start justify-between gap-4">
            <div>
              <h2 class="text-2xl font-semibold text-primary">Checking your device</h2>
              <p class="mt-2 text-sm text-secondary">
                {systemHealth?.summary ?? 'Gathering system information...'}
              </p>
            </div>
            {#if systemHealth?.state === 'pass'}
              <p class="rounded-full bg-success/15 px-3 py-2 text-xs font-medium uppercase tracking-wide text-success">
                Auto-advancing
              </p>
            {/if}
          </div>

          <div class="mt-6 grid gap-4 md:grid-cols-3">
            {#each systemHealth?.checks ?? [] as check}
              <div class={`rounded-xl border p-4 ${check.state === 'fail' ? 'border-error/40 bg-error/10' : check.state === 'warning' ? 'border-warning/40 bg-warning/10' : 'border-success/30 bg-success/10'}`}>
                <p class="text-sm uppercase tracking-wide text-secondary">{check.label}</p>
                <p class="mt-2 text-xl font-semibold text-primary">{check.value}</p>
                <p class="mt-2 text-xs text-secondary">{check.message}</p>
              </div>
            {/each}
          </div>

          <div class="mt-6 flex flex-wrap gap-3">
            <button
              class="rounded-lg bg-accent px-6 py-3 font-medium text-white disabled:opacity-50"
              on:click={continueFromSystemCheck}
              disabled={!systemHealth || systemCheckActions.continueDisabled}
            >
              {systemCheckActions.primaryLabel}
            </button>
            <button class="rounded-lg border border-white/20 px-6 py-3 text-white" on:click={() => setupStep.set(1)}>
              Back
            </button>
          </div>

          {#if systemCheckActions.supportVisible}
            <div class="mt-4 rounded-xl border border-warning/40 bg-warning/10 p-4 text-sm text-secondary">
              <p class="font-semibold text-primary">Troubleshooting required</p>
              <p class="mt-2">{systemCheckActions.supportMessage}</p>
              <button
                class="mt-4 rounded-lg border border-white/20 px-4 py-2 text-xs text-white"
                on:click={runSupportAction}
              >
                {supportAction.label}
              </button>
            </div>
          {/if}
        </div>
      {:else if $setupStep === 3}
        <div class="space-y-1 text-sm text-secondary">
          <p class="font-semibold text-primary">Ollama status</p>
          <p>{statusLabel}</p>
          <p class="text-xs">{statusDetails}</p>

          <div class="mt-4 flex flex-wrap gap-3">
            {#if isRunning}
              <button
                class="rounded-lg border border-white/20 bg-error/80 px-6 py-3 font-medium text-white disabled:opacity-50"
                on:click={stopServer}
                disabled={installLoading || stopLoading}
              >
                {stopLoading ? 'Stopping...' : 'Stop Ollama'}
              </button>
            {:else if hasBinary}
              <button
                class="rounded-lg bg-success px-6 py-3 font-medium text-white disabled:opacity-50"
                on:click={startServer}
                disabled={installLoading || startLoading}
              >
                {startLoading ? 'Starting...' : 'Start Ollama'}
              </button>
            {:else}
              <button
                class="rounded-lg bg-accent px-6 py-3 font-medium text-white disabled:opacity-50"
                on:click={installBinary}
                disabled={installLoading}
              >
                {installLoading ? 'Installing...' : 'Install Ollama'}
              </button>

              <p class="max-w-sm text-xs text-secondary">
                The launcher will download the official Ollama installer and run it in the background.
              </p>

              <button
                class="rounded-lg border border-white/20 px-4 py-2 text-xs text-white"
                on:click={runSupportAction}
              >
                {supportAction.label}
              </button>

              {#if $installProgress}
                <div class="min-w-[16rem]">
                  <ProgressBar
                    label="Install progress"
                    percent={$installProgress.percent}
                    status={$installProgress.status}
                  />
                </div>
              {/if}
            {/if}
          </div>

        {#if actionError}
          <p class="text-xs text-error">{actionError}</p>
        {/if}
        </div>

        {#if isRunning}
          <div class="mt-6 space-y-3 rounded-xl border border-white/10 bg-white/5 p-4">
            <div>
              <p class="font-semibold text-primary">Handy configuration</p>
              <p class="text-xs text-secondary">
                Apply the required Handy `settings_store.json` changes for the active Ollama server.
              </p>
              {#if handyConfigStatus?.handy_running}
                <p class="mt-2 text-xs text-warning">
                  Handy appears to be running. Close it before writing configuration changes.
                </p>
              {/if}
            </div>

            <div class="space-y-3">
              <div class="flex items-center justify-between gap-3">
                <span class="text-xs uppercase tracking-wide text-secondary">Model profile</span>
                <p class="text-xs text-secondary">
                  Recommended: {availableProfiles.find((profile) => profile.id === recommendedProfileId)?.label ?? 'Light'}
                </p>
              </div>

              <div class="grid gap-3 md:grid-cols-3">
                {#each availableProfiles as profile}
                  <button
                    class:selected={$selectedProfile === profile.id}
                    class:unsupported={!profile.supported}
                    class="rounded-xl border border-white/10 bg-surface p-4 text-left transition"
                    on:click={() => chooseProfile(profile)}
                    disabled={!profile.supported}
                  >
                    <span class="flex items-start justify-between gap-2">
                      <span class="block">
                        <span class="block font-semibold text-primary">{profile.label}</span>
                        <span class="mt-1 block text-xs text-secondary">{profile.description}</span>
                      </span>
                      {#if profile.recommended}
                        <span class="rounded-full bg-success/20 px-2 py-1 text-[10px] uppercase tracking-wide text-success">
                          Recommended
                        </span>
                      {/if}
                    </span>

                    <span class="mt-4 block space-y-1 text-xs text-secondary">
                      <span class="block">Model: {profile.model}</span>
                      <span class="block">Download: {profile.sizeLabel}</span>
                      <span class="block">RAM required: {profile.ramRequiredGb} GB+</span>
                      <span class="block">Setup time: {profile.downloadEstimate}</span>
                    </span>

                    {#if !profile.supported}
                      <span class="mt-4 block text-xs text-warning">
                        Unavailable with {$systemInfo?.available_ram_gb.toFixed(1) ?? 'current'} GB available RAM.
                      </span>
                    {/if}
                  </button>
                {/each}
              </div>

              <p class="text-xs text-secondary">
                {selectionSummary.title}
              </p>
              <p class="text-xs text-secondary">{selectionSummary.setupEstimate}</p>
              <button
                class="w-fit rounded-lg border border-white/20 px-3 py-2 text-xs text-white"
                on:click={() => (showTechnicalDetails = !showTechnicalDetails)}
              >
                {showTechnicalDetails ? 'Hide details' : 'Show details'}
              </button>

              {#if showTechnicalDetails && selectionSummary.details.length > 0}
                <div class="grid gap-2 rounded-xl border border-white/10 bg-surface/60 p-4 text-xs text-secondary md:grid-cols-2">
                  {#each selectionSummary.details as [label, value]}
                    <p><span class="text-primary">{label}:</span> {value}</p>
                  {/each}
                </div>
              {/if}
            </div>

            <div class="flex flex-wrap items-center gap-3">
              {#if modelAvailability.state !== 'ready'}
                <button
                  class="rounded-lg border border-white/20 px-6 py-3 font-medium text-white disabled:opacity-50"
                  on:click={downloadSelectedModel}
                  disabled={downloadLoading || !$selectedModel}
                >
                  {downloadLoading ? 'Downloading model...' : 'Confirm and download'}
                </button>
              {/if}
              <button
                class="rounded-lg bg-accent px-6 py-3 font-medium text-white disabled:opacity-50"
                on:click={configureHandy}
                disabled={configureLoading || !$selectedModel || handyConfigStatus?.handy_running || modelAvailability.state !== 'ready'}
              >
                {configureLoading ? 'Configuring...' : 'Configure Handy'}
              </button>
              <p class="text-xs text-secondary">Writes Handy's local config with the `custom` provider.</p>
            </div>

            {#if $modelDownloadProgress}
              <ProgressBar
                label={`Model download: ${$modelDownloadProgress.modelName}`}
                percent={$modelDownloadProgress.percent}
                status={$modelDownloadProgress.status}
                minimumVisiblePercent={8}
              />
            {/if}

            <div class="space-y-1 text-xs text-secondary">
              <p>
                Model status:
                {#if modelAvailability.state === 'ready'}
                  Ready to configure
                {:else if modelAvailability.state === 'missing'}
                  Download required before configuring Handy
                {:else}
                  Select a model profile
                {/if}
              </p>

              {#if modelAvailability.alternativeModels.length > 0 && modelAvailability.state === 'missing'}
                <p>Already downloaded: {modelAvailability.alternativeModels.join(', ')}</p>
              {/if}

              {#if downloadMessage}
                <p class="text-success">{downloadMessage}</p>
              {/if}
            </div>

            {#if handyConfigStatus}
              <div class="grid gap-2 text-xs text-secondary md:grid-cols-2">
                <p>Config file: {handyConfigStatus.config_path ?? 'Unavailable'}</p>
                <p>Current provider: {handyConfigStatus.current_provider_id ?? 'Not set'}</p>
                <p>Current model: {handyConfigStatus.configured_model ?? 'Not set'}</p>
                <p>Latest backup: {handyConfigStatus.latest_backup_path ?? 'No backup yet'}</p>
                <p>Downloaded models: {downloadedModels.length > 0 ? downloadedModels.map((model) => model.name).join(', ') : 'None detected'}</p>
              </div>
            {/if}

            {#if configureMessage}
              <p class="text-xs text-success">{configureMessage}</p>
            {/if}
          </div>
        {/if}
      {:else}
        <div class="rounded-2xl border border-success/20 bg-success/10 p-6">
          <p class="text-sm uppercase tracking-wide text-success">Step 4</p>
          <h2 class="mt-2 text-2xl font-semibold text-primary">Local transcription is ready</h2>
          <p class="mt-3 max-w-2xl text-sm text-secondary">
            Handy is configured to use the local Ollama server with {$selectedModel ?? 'your selected model'}.
            You can return to setup at any time to switch models or re-run configuration.
          </p>

          <div class="mt-6 grid gap-3 text-sm text-secondary md:grid-cols-2">
            <p>Configured model: {handyConfigStatus?.configured_model ?? $selectedModel ?? 'Unknown'}</p>
            <p>Provider: {handyConfigStatus?.current_provider_id ?? 'custom'}</p>
            <p>Server port: {currentPort}</p>
            <p>Backup: {handyConfigStatus?.latest_backup_path ?? 'Created when available'}</p>
          </div>

          <div class="mt-6 flex flex-wrap gap-3">
            <button
              class="rounded-lg bg-accent px-6 py-3 font-medium text-white disabled:opacity-50"
              on:click={runCompletionAction}
              disabled={completionLoading || completionAction.disabled}
            >
              {#if completionLoading}
                Opening Handy...
              {:else}
                {completionAction.label}
              {/if}
            </button>
            <button class="rounded-lg border border-white/20 px-6 py-3 text-white" on:click={refreshState}>
              Refresh status
            </button>
            <button class="rounded-lg border border-white/20 px-6 py-3 text-white" on:click={runSupportAction}>
              View logs
            </button>
            <button class="rounded-lg border border-white/20 px-6 py-3 text-white" on:click={reconfigureSetup}>
              Reconfigure
            </button>
          </div>

          {#if actionError}
            <p class="mt-3 text-xs text-error">{actionError}</p>
          {/if}

          <p class="mt-3 text-xs text-secondary">{supportAction.description}</p>
        </div>
      {/if}
    </div>

    <div class="rounded-2xl border border-white/10 bg-card p-8">
      <div class="mb-6 flex items-center justify-between">
        <div>
          <h2 class="text-xl font-semibold">System health</h2>
          <p class="text-sm text-secondary">CPU / RAM / storage snapshot</p>
        </div>
      </div>

      <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
        <div class="space-y-1 rounded-xl bg-white/5 p-4">
          <p class="text-sm uppercase tracking-wide text-secondary">RAM</p>
          <p class="text-2xl font-semibold">
            {#if $systemInfo}
              {$systemInfo.total_ram_gb.toFixed(1)} GB
            {:else}
              Loading...
            {/if}
          </p>
          <p class="text-xs text-secondary">
            {#if $systemInfo}
              {$systemInfo.available_ram_gb.toFixed(1)} GB available
            {/if}
          </p>
        </div>

        <div class="space-y-1 rounded-xl bg-white/5 p-4">
          <p class="text-sm uppercase tracking-wide text-secondary">Disk</p>
          <p class="text-2xl font-semibold">
            {#if $systemInfo}
              {$systemInfo.total_disk_gb.toFixed(1)} GB
            {:else}
              Loading...
            {/if}
          </p>
          <p class="text-xs text-secondary">
            {#if $systemInfo}
              {$systemInfo.available_disk_gb.toFixed(1)} GB free
            {/if}
          </p>
        </div>

        <div class="space-y-1 rounded-xl bg-white/5 p-4">
          <p class="text-sm uppercase tracking-wide text-secondary">Operating system</p>
          <p class="text-2xl font-semibold">
            {#if $systemInfo}
              {$systemInfo.os_name}
            {:else}
              Loading...
            {/if}
          </p>
          <p class="text-xs text-secondary">
            {#if $systemInfo}
              Version {$systemInfo.os_version}
            {/if}
          </p>
        </div>
      </div>
    </div>
  </section>
</main>

<style>
  .selected {
    border-color: rgba(255, 255, 255, 0.5);
    background: rgba(255, 255, 255, 0.08);
  }

  .unsupported {
    opacity: 0.55;
  }

  .step-active {
    border-color: rgba(255, 255, 255, 0.45);
    background: rgba(255, 255, 255, 0.08);
    color: white;
  }
</style>
