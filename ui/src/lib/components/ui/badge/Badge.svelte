<script lang="ts">
  import type { HTMLAttributes } from "svelte/elements";
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";
  import { badgeVariants, type BadgeVariant } from "./index";

  interface Props extends HTMLAttributes<HTMLDivElement> {
    variant?: BadgeVariant;
    children?: Snippet;
  }

  let {
    class: className,
    variant = "default",
    children,
    ...restProps
  }: Props = $props();
</script>

<div class={cn(badgeVariants({ variant }), className)} {...restProps}>
  {@render children?.()}
</div>
