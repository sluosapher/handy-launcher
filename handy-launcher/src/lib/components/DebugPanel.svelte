<script>
  export let loading = false;
  export let snapshot = null;
  export let actionError = '';
  export let systemInfo = null;
  export let ollamaStatus = null;
  export let configStatus = null;
</script>

<section class="rounded-2xl border border-warning/30 bg-warning/10 p-6">
  <div class="flex flex-wrap items-start justify-between gap-4">
    <div>
      <p class="text-xs uppercase tracking-[0.3em] text-warning">Troubleshooting mode</p>
      <h2 class="mt-2 text-xl font-semibold text-primary">Debug panel</h2>
      <p class="mt-2 text-sm text-secondary">
        Inspect the latest launcher diagnostics, current raw state, and log-file location.
      </p>
    </div>
    {#if loading}
      <p class="rounded-full border border-white/20 px-3 py-1 text-xs text-white">Refreshing…</p>
    {/if}
  </div>

  {#if actionError}
    <p class="mt-4 rounded-xl border border-error/40 bg-error/10 px-4 py-3 text-xs text-error">
      {actionError}
    </p>
  {/if}

  <div class="mt-6 grid gap-4 lg:grid-cols-2">
    <div class="rounded-xl border border-white/10 bg-surface/70 p-4">
      <p class="text-xs uppercase tracking-[0.3em] text-secondary">Launcher logs</p>
      <p class="mt-2 break-all text-xs text-secondary">
        {snapshot?.log_path ?? 'Log file not created yet'}
      </p>

      <div class="mt-4 max-h-72 overflow-auto rounded-lg bg-black/30 p-3 font-mono text-[11px] text-white/90">
        {#if snapshot?.recent_logs?.length}
          {#each snapshot.recent_logs as line}
            <p class="whitespace-pre-wrap break-words">{line}</p>
          {/each}
        {:else}
          <p class="text-white/60">No recent log lines available.</p>
        {/if}
      </div>
    </div>

    <div class="space-y-4">
      <div class="rounded-xl border border-white/10 bg-surface/70 p-4">
        <p class="text-xs uppercase tracking-[0.3em] text-secondary">System snapshot</p>
        <pre class="mt-3 overflow-auto text-xs text-white/90">{JSON.stringify(systemInfo, null, 2)}</pre>
      </div>

      <div class="rounded-xl border border-white/10 bg-surface/70 p-4">
        <p class="text-xs uppercase tracking-[0.3em] text-secondary">Ollama state</p>
        <pre class="mt-3 overflow-auto text-xs text-white/90">{JSON.stringify(ollamaStatus, null, 2)}</pre>
      </div>

      <div class="rounded-xl border border-white/10 bg-surface/70 p-4">
        <p class="text-xs uppercase tracking-[0.3em] text-secondary">Handy config status</p>
        <pre class="mt-3 overflow-auto text-xs text-white/90">{JSON.stringify(configStatus, null, 2)}</pre>
      </div>
    </div>
  </div>
</section>
