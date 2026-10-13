<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";

  interface Props {
    children?: Snippet;
    class?: string;
  }

  let { children, class: className = "" }: Props = $props();
</script>

<h2 class={cn("text-base font-semibold text-foreground tracking-tight break-words [overflow-wrap:anywhere]", className)}>
  {#if children}
    {@render children()}
  {/if}
</h2>
