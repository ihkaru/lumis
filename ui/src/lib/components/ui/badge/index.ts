import Root from "./Badge.svelte";
import { tv, type VariantProps } from "tailwind-variants";

export const badgeVariants = tv({
  base: "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 select-none",
  variants: {
    variant: {
      default: "border-transparent bg-primary text-primary-foreground shadow hover:bg-primary/80",
      secondary: "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80",
      destructive: "border-transparent bg-destructive text-destructive-foreground shadow hover:bg-destructive/80",
      outline: "text-foreground",
      cyan: "border-primary/40 bg-primary/10 text-primary",
    },
  },
  defaultVariants: {
    variant: "default",
  },
});

export type BadgeVariant = VariantProps<typeof badgeVariants>["variant"];

export {
  Root,
  Root as Badge,
};
