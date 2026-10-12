<script lang="ts">
  import ChevronLeft from "lucide-svelte/icons/chevron-left";
  import ChevronRight from "lucide-svelte/icons/chevron-right";

  interface Props {
    current: number;
    total: number;
    onPrev?: () => void;
    onNext?: () => void;
    class?: string;
  }

  let { current = 1, total = 1, onPrev, onNext, class: className = "" }: Props = $props();

  let canPrev = $derived(current > 1);
  let canNext = $derived(current < total);
</script>

{#if total > 1}
  <div
    class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-md bg-muted/60 border border-border/60 text-[11px] font-mono text-muted-foreground select-none {className}"
    role="navigation"
    aria-label="Message branch pagination"
  >
    <button
      type="button"
      class="p-0.5 rounded hover:bg-background/80 hover:text-foreground disabled:opacity-30 disabled:pointer-events-none transition-colors duration-150 cursor-pointer"
      disabled={!canPrev}
      onclick={() => onPrev?.()}
      aria-label="Previous version"
    >
      <ChevronLeft class="w-3.5 h-3.5" />
    </button>
    <span class="px-1 font-medium text-foreground tracking-tight">
      {current} / {total}
    </span>
    <button
      type="button"
      class="p-0.5 rounded hover:bg-background/80 hover:text-foreground disabled:opacity-30 disabled:pointer-events-none transition-colors duration-150 cursor-pointer"
      disabled={!canNext}
      onclick={() => onNext?.()}
      aria-label="Next version"
    >
      <ChevronRight class="w-3.5 h-3.5" />
    </button>
  </div>
{/if}
