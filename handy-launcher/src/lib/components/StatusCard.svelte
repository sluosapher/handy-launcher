<script>
  export let label = '';
  export let value = '';
  export let helper = '';
  export let tone = 'neutral';

  const toneClasses = {
    neutral: 'border border-white/10 bg-surface',
    success: 'border border-success/30 bg-success/10 text-success',
    warning: 'border border-warning/30 bg-warning/10 text-warning',
    error: 'border border-error/30 bg-error/10 text-error'
  };

  $: cardClass = toneClasses[tone] ?? toneClasses.neutral;
</script>

<div class={`rounded-2xl px-5 py-4 text-sm ${cardClass}`}>
  <p class="text-[10px] uppercase tracking-[0.3em] text-secondary">{label}</p>
  <p class="mt-2 text-xl font-semibold leading-tight text-primary">{value}</p>
  {#if helper}
    <p class="mt-1 text-xs text-secondary">{helper}</p>
  {/if}
</div>
