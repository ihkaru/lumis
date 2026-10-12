<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children?: Snippet;
    class?: string;
  }

  let { children, class: className = "" }: Props = $props();
</script>

<h3 class="font-semibold text-sm text-foreground tracking-tight {className}">
  {#if children}
    {@render children()}
  {/if}
</h3>
