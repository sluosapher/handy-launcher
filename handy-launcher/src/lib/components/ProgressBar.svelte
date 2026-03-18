<script lang="ts">
  export let label = 'Progress';
  export let percent = 0;
  export let status = '';
  export let minimumVisiblePercent = 6;
</script>

<div class="rounded-lg border border-white/10 bg-white/5 px-4 py-3">
  <p class="text-xs uppercase tracking-wide text-secondary">{label}</p>
  <p class="mt-1 text-sm text-primary">{Math.max(0, percent)}% | {status}</p>
  <div class="mt-3 h-2 overflow-hidden rounded-full bg-white/10">
    <div
      class="h-full rounded-full bg-accent transition-all duration-300"
      style={`width: ${Math.max(minimumVisiblePercent, percent)}%`}
    ></div>
  </div>
</div>
