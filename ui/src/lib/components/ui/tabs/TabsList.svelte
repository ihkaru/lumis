<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children?: Snippet;
    class?: string;
  }

  let { children, class: className = "" }: Props = $props();
</script>

<div
  class="inline-flex items-center justify-center rounded-xl bg-muted/60 p-1 text-muted-foreground select-none {className}"
  role="tablist"
>
  {#if children}
    {@render children()}
  {/if}
</div>
