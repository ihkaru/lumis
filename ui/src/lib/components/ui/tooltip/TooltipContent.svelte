<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children?: Snippet;
    side?: "top" | "bottom" | "left" | "right";
    class?: string;
  }

  let { children, side = "top", class: className = "" }: Props = $props();

  const posClasses = {
    top: "bottom-full left-1/2 -translate-x-1/2 mb-2",
    bottom: "top-full left-1/2 -translate-x-1/2 mt-2",
    left: "right-full top-1/2 -translate-y-1/2 mr-2",
    right: "left-full top-1/2 -translate-y-1/2 ml-2"
  };
</script>

<div
  role="tooltip"
  class="absolute {posClasses[side]} z-50 overflow-hidden rounded-md bg-popover px-2.5 py-1 text-[11px] text-popover-foreground shadow-md border border-border opacity-0 group-hover/tooltip:opacity-100 transition-opacity duration-150 pointer-events-none select-none font-sans whitespace-nowrap {className}"
>
  {#if children}
    {@render children()}
  {/if}
</div>
