<script lang="ts">
  import ListTree from "lucide-svelte/icons/list-tree";
  import ChevronRight from "lucide-svelte/icons/chevron-right";

  interface Props {
    content: string;
    onScrollToHeading?: (headingText: string) => void;
    class?: string;
  }

  let { content = "", onScrollToHeading, class: className = "" }: Props = $props();

  let isExpanded = $state(false);

  interface HeadingItem {
    level: number;
    text: string;
    id: string;
  }

  let headings = $derived.by(() => {
    if (!content) return [];
    const lines = content.split("\n");
    const items: HeadingItem[] = [];

    for (const line of lines) {
      const match = line.match(/^(#{1,3})\s+(.+)$/);
      if (match) {
        const level = match[1].length;
        const text = match[2].trim().replace(/[*_`]/g, "");
        const id = text.toLowerCase().replace(/[^\w\s-]/g, "").replace(/\s+/g, "-");
        items.push({ level, text, id });
      }
    }
    return items;
  });

  function handleJump(item: HeadingItem) {
    if (onScrollToHeading) {
      onScrollToHeading(item.text);
    } else {
      const el = document.getElementById(item.id);
      if (el) el.scrollIntoView({ behavior: "smooth", block: "start" });
    }
  }
</script>

{#if headings.length >= 2}
  <div class="relative font-sans text-xs select-none {className}">
    <button
      type="button"
      class="inline-flex items-center gap-1.5 px-2 py-1 rounded-lg bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground border border-border/60 transition-colors cursor-pointer text-[11px]"
      onclick={() => (isExpanded = !isExpanded)}
      title="Daftar Isi / Minimap Outline"
    >
      <ListTree class="w-3.5 h-3.5 text-cyan-400" />
      <span>Outline ({headings.length})</span>
      <ChevronRight class="w-3 h-3 transition-transform duration-150 {isExpanded ? 'rotate-90' : ''}" />
    </button>

    {#if isExpanded}
      <div class="absolute right-0 top-7 z-20 w-64 max-h-60 overflow-y-auto p-2 rounded-xl bg-popover/95 backdrop-blur-md border border-border shadow-xl space-y-1 animate-in fade-in zoom-in-95 duration-100">
        <div class="px-2 py-1 text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
          Navigasi Bagian
        </div>
        {#each headings as h}
          <button
            type="button"
            class="w-full text-left px-2 py-1 rounded-md text-[11px] text-muted-foreground hover:text-foreground hover:bg-muted/80 transition-colors cursor-pointer truncate {h.level === 1 ? 'font-semibold text-foreground' : h.level === 2 ? 'pl-4' : 'pl-6'}"
            onclick={() => handleJump(h)}
            title={h.text}
          >
            {h.text}
          </button>
        {/each}
      </div>
    {/if}
  </div>
{/if}
