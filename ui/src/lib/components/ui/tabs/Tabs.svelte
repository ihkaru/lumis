<script lang="ts">
  import type { Snippet } from "svelte";
  import { setContext } from "svelte";

  interface Props {
    value?: string;
    onValueChange?: (value: string) => void;
    children?: Snippet;
    class?: string;
  }

  let { value = $bindable(""), onValueChange, children, class: className = "" }: Props = $props();

  setContext("tabs", {
    get current() {
      return value;
    },
    setTab(v: string) {
      value = v;
      onValueChange?.(v);
    }
  });
</script>

<div class="flex flex-col {className}">
  {#if children}
    {@render children()}
  {/if}
</div>
