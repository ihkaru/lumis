import Root from "./Button.svelte";
import { tv, type VariantProps } from "tailwind-variants";

export const buttonVariants = tv({
  base: "inline-flex items-center justify-center whitespace-nowrap rounded-lg text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 select-none",
  variants: {
    variant: {
      default: "bg-primary text-primary-foreground shadow hover:bg-primary/90",
      destructive: "bg-destructive text-destructive-foreground shadow-sm hover:bg-destructive/90",
      outline: "border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground",
      secondary: "bg-secondary text-secondary-foreground shadow-sm hover:bg-secondary/80",
      ghost: "hover:bg-accent hover:text-accent-foreground",
      link: "text-primary underline-offset-4 hover:underline",
      cyan: "bg-primary text-primary-foreground shadow hover:bg-primary/90 active:bg-primary/80",
    },
    size: {
      default: "h-9 px-4 py-2",
      sm: "h-8 rounded-md px-3 text-xs",
      lg: "h-11 rounded-md px-8",
      icon: "h-9 w-9 min-h-[44px] min-w-[44px]",
      touch: "min-h-[44px] min-w-[44px] px-3 py-2",
    },
  },
  defaultVariants: {
    variant: "default",
    size: "default",
  },
});

export type ButtonVariant = VariantProps<typeof buttonVariants>["variant"];
export type ButtonSize = VariantProps<typeof buttonVariants>["size"];

export {
  Root,
  Root as Button,
};
