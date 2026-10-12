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
  class="absolute right-4 top-4 rounded-lg p-2 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors cursor-pointer min-h-[44px] min-w-[44px] flex items-center justify-center touch-manipulation {className}"
  {onclick}
  aria-label="Tutup Dialog"
>
  <X class="w-4 h-4" />
</button>
