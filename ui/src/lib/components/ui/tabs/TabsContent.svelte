<script lang="ts">
  import type { Snippet } from "svelte";
  import { getContext } from "svelte";

  interface Props {
    value: string;
    children?: Snippet;
    class?: string;
  }

  let { value, children, class: className = "" }: Props = $props();

  const ctx = getContext<{ current: string }>("tabs");
  let isActive = $derived(ctx?.current === value);
</script>

{#if isActive}
  <div
    role="tabpanel"
    class="mt-2 focus-visible:outline-none {className}"
  >
    {#if children}
      {@render children()}
    {/if}
  </div>
{/if}
