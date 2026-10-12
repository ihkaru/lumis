<script lang="ts">
  import type { HTMLButtonAttributes } from "svelte/elements";
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";
  import { buttonVariants, type ButtonVariant, type ButtonSize } from "./index";

  interface Props extends HTMLButtonAttributes {
    variant?: ButtonVariant;
    size?: ButtonSize;
    children?: Snippet;
  }

  let {
    class: className,
    variant = "default",
    size = "default",
    type = "button",
    children,
    ...restProps
  }: Props = $props();
</script>

<button
  {type}
  class={cn(buttonVariants({ variant, size }), className)}
  {...restProps}
>
  {@render children?.()}
</button>
