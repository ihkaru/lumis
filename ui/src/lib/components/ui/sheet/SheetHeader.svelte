<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children?: Snippet;
    class?: string;
  }

  let { children, class: className = "" }: Props = $props();
</script>

<div class="flex flex-col space-y-1.5 p-4 border-b border-border text-left {className}">
  {#if children}
    {@render children()}
  {/if}
</div>
