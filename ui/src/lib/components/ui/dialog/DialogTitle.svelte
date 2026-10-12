<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children?: Snippet;
    class?: string;
  }

  let { children, class: className = "" }: Props = $props();
</script>

<h2 class="text-base font-semibold text-foreground tracking-tight break-words [overflow-wrap:anywhere] {className}">
  {#if children}
    {@render children()}
  {/if}
</h2>
