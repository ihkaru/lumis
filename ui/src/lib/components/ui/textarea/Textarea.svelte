<script lang="ts">
  import type { HTMLTextareaAttributes } from "svelte/elements";
  import { cn } from "$lib/utils";

  interface Props extends HTMLTextareaAttributes {
    value?: string;
  }

  let {
    class: className,
    value = $bindable(""),
    ...restProps
  }: Props = $props();
</script>

<textarea
  bind:value
  class={cn(
    "flex min-h-[48px] w-full rounded-xl border border-input bg-background px-4 py-3 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-primary focus-visible:border-primary disabled:cursor-not-allowed disabled:opacity-50 break-words [overflow-wrap:anywhere]",
    className
  )}
  {...restProps}
></textarea>
