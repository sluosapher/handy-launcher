<script>
  export let label = '';
  export let variant = 'primary';
  export let loading = false;
  export let loadingLabel = 'Working...';
  export let disabled = false;
  export let type = 'button';

  const variantClasses = {
    primary: 'bg-accent text-white border border-transparent hover:opacity-90',
    success: 'bg-success text-primary border border-transparent hover:opacity-90',
    danger: 'bg-error text-white border border-transparent hover:opacity-90',
    ghost: 'bg-transparent border border-white/20 text-white hover:border-white/40'
  };

  $: buttonClass = variantClasses[variant] ?? variantClasses.primary;
</script>

<button
  type={type}
  class={`rounded-lg px-5 py-3 font-medium transition ${buttonClass} disabled:cursor-not-allowed disabled:opacity-50`}
  disabled={disabled || loading}
>
  {#if loading}
    <span class="flex items-center justify-center gap-2 text-white">
      <span class="inline-flex h-2 w-2 animate-pulse rounded-full bg-current"></span>
      {loadingLabel}
    </span>
  {:else}
    {label}
  {/if}
</button>
