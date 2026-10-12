<script lang="ts">
  import type { Snippet } from "svelte";
  import { getContext } from "svelte";

  interface Props {
    value: string;
    children?: Snippet;
    class?: string;
  }

  let { value, children, class: className = "" }: Props = $props();

  const ctx = getContext<{ current: string; setTab: (v: string) => void }>("tabs");
  let isActive = $derived(ctx?.current === value);
</script>

<button
  type="button"
  role="tab"
  aria-selected={isActive}
  onclick={() => ctx?.setTab(value)}
  class="inline-flex items-center justify-center whitespace-nowrap rounded-lg px-3 py-1.5 text-xs font-medium transition-all cursor-pointer min-h-[44px] sm:min-h-0 touch-manipulation {isActive ? 'bg-background text-foreground shadow-xs' : 'hover:bg-background/50 hover:text-foreground text-muted-foreground'} {className}"
>
  {#if children}
    {@render children()}
  {/if}
</button>
