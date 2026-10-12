<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children?: Snippet;
    class?: string;
  }

  let { children, class: className = "" }: Props = $props();
</script>

<p class="text-xs text-muted-foreground {className}">
  {#if children}
    {@render children()}
  {/if}
</p>
