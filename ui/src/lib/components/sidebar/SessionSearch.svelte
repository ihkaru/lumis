<script lang="ts">
  import Search from "lucide-svelte/icons/search";
  import X from "lucide-svelte/icons/x";

  interface Props {
    searchQuery: string;
    onSearchChange: (query: string) => void;
  }

  let { searchQuery = $bindable(""), onSearchChange }: Props = $props();

  function handleClear() {
    searchQuery = "";
    onSearchChange("");
  }
</script>

<div class="relative px-3 py-2 font-sans select-none">
  <div class="relative flex items-center">
    <Search class="absolute left-2.5 h-3.5 w-3.5 text-muted-foreground pointer-events-none" />
    <input
      type="text"
      bind:value={searchQuery}
      oninput={(e) => onSearchChange((e.target as HTMLInputElement).value)}
      placeholder="Cari obrolan..."
      class="w-full h-8 pl-8 pr-7 rounded-lg border border-border/80 bg-background/60 text-xs text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary/50"
    />
    {#if searchQuery}
      <button
        type="button"
        onclick={handleClear}
        class="absolute right-2 p-0.5 text-muted-foreground hover:text-foreground cursor-pointer"
        aria-label="Bersihkan pencarian"
      >
        <X class="h-3.5 w-3.5" />
      </button>
    {/if}
  </div>
</div>
