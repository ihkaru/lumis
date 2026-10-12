<script lang="ts">
  import X from "lucide-svelte/icons/x";

  interface Props {
    onclick?: () => void;
    class?: string;
  }

  let { onclick, class: className = "" }: Props = $props();
</script>

<button
  type="button"
  class="min-h-[44px] min-w-[44px] h-11 w-11 flex items-center justify-center rounded-xl bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground transition-colors cursor-pointer select-none border border-border/40 {className}"
  {onclick}
  aria-label="Tutup Panel Navigasi"
>
  <X class="w-5 h-5" />
</button>
