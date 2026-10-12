<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    open?: boolean;
    children?: Snippet;
  }

  let { open = false, children }: Props = $props();
</script>

<div class="relative inline-flex group/tooltip">
  {#if children}
    {@render children()}
  {/if}
</div>
