<script lang="ts">
  import { cn } from "$lib/utils";

  interface Props {
    open?: boolean;
    onclick?: () => void;
    class?: string;
  }

  let { open = true, onclick, class: className = "" }: Props = $props();
</script>

{#if open}
  <button
    type="button"
    class={cn(
      "fixed inset-0 z-50 bg-black/75 backdrop-blur-xs transition-opacity duration-200 border-0 p-0 w-full h-full cursor-default select-none",
      className
    )}
    {onclick}
    aria-label="Tutup Dialog"
  ></button>
{/if}
