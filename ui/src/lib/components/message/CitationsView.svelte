<script lang="ts">
  import Globe from "lucide-svelte/icons/globe";
  import ExternalLink from "lucide-svelte/icons/external-link";
  import ChevronDown from "lucide-svelte/icons/chevron-down";

  export interface Citation {
    url: string;
    title?: string;
    snippet?: string;
  }

  interface Props {
    citations: Citation[];
  }

  let { citations = [] }: Props = $props();
  let isOpen = $state(false);

  function getDomain(url: string) {
    try {
      return new URL(url).hostname;
    } catch {
      return url;
    }
  }
</script>

{#if citations.length > 0}
  <div class="my-2.5 overflow-hidden rounded-xl border border-border/80 bg-muted/20 font-sans text-xs">
    <button
      type="button"
      onclick={() => (isOpen = !isOpen)}
      class="flex w-full items-center justify-between px-3.5 py-2 text-left hover:bg-muted/40 transition-colors select-none cursor-pointer"
      aria-expanded={isOpen}
    >
      <div class="flex items-center gap-2">
        <Globe class="h-3.5 w-3.5 text-cyan-400 shrink-0" />
        <span class="font-medium text-foreground">Sumber Rujukan Web & RAG</span>
        <span class="text-[10px] text-muted-foreground font-mono">({citations.length} sumber)</span>
      </div>
      <ChevronDown class="h-4 w-4 text-muted-foreground transition-transform duration-200 {isOpen ? 'rotate-180' : ''}" />
    </button>

    {#if isOpen}
      <div class="border-t border-border/60 p-2.5 grid grid-cols-1 sm:grid-cols-2 gap-2 bg-background/40">
        {#each citations as item}
          <a
            href={item.url}
            target="_blank"
            rel="noopener noreferrer"
            class="group flex items-start gap-2 p-2 rounded-lg border border-border/60 bg-card hover:border-cyan-500/40 hover:bg-muted/40 transition-all text-left"
          >
            <div class="p-1 rounded bg-muted text-muted-foreground group-hover:text-cyan-400 shrink-0 mt-0.5">
              <ExternalLink class="h-3 w-3" />
            </div>
            <div class="min-w-0 flex-1">
              <div class="font-semibold text-xs text-foreground truncate group-hover:text-cyan-400 transition-colors">
                {item.title || getDomain(item.url)}
              </div>
              <div class="text-[10px] text-muted-foreground truncate font-mono">
                {getDomain(item.url)}
              </div>
            </div>
          </a>
        {/each}
      </div>
    {/if}
  </div>
{/if}
