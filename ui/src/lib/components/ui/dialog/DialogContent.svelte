<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";

  interface Props {
    open?: boolean;
    onClose?: () => void;
    children?: Snippet;
    class?: string;
  }

  let { open = true, onClose, children, class: className = "" }: Props = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && open) {
      onClose?.();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div
    class={cn(
      "fixed left-1/2 top-1/2 z-50 w-full max-w-lg -translate-x-1/2 -translate-y-1/2 p-6 rounded-2xl bg-card border border-border shadow-2xl text-foreground duration-200 animate-in fade-in zoom-in-95 font-sans [overflow-wrap:anywhere]",
      className
    )}
    role="dialog"
    aria-modal="true"
  >
    {#if children}
      {@render children()}
    {/if}
  </div>
{/if}
