<script lang="ts">
  import type { Snippet } from "svelte";
  import X from "lucide-svelte/icons/x";

  interface Props {
    open?: boolean;
    side?: "left" | "right" | "top" | "bottom";
    onClose?: () => void;
    children?: Snippet;
    class?: string;
  }

  let {
    open = true,
    side = "left",
    onClose,
    children,
    class: className = ""
  }: Props = $props();

  const sideClasses = {
    left: "inset-y-0 left-0 w-72 sm:w-80 border-r",
    right: "inset-y-0 right-0 w-72 sm:w-80 border-l",
    top: "inset-x-0 top-0 border-b",
    bottom: "inset-x-0 bottom-0 border-t"
  };
</script>

{#if open}
  <div
    class="fixed z-50 bg-card border-border shadow-2xl flex flex-col justify-between transition-transform duration-200 select-none font-sans {sideClasses[side]} {className}"
    role="dialog"
    aria-modal="true"
  >
    {#if children}
      {@render children()}
    {/if}
  </div>
{/if}
