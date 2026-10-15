<script lang="ts">
  import MessageSquare from "lucide-svelte/icons/message-square";
  import Globe from "lucide-svelte/icons/globe";
  import Atom from "lucide-svelte/icons/atom";

  export type ChatMode = "normal" | "web_search" | "deep_research";

  interface Props {
    activeMode: ChatMode;
    onSelectMode: (mode: ChatMode) => void;
  }

  let { activeMode = "normal", onSelectMode }: Props = $props();

  const modes = [
    { id: "normal" as ChatMode, label: "Normal", icon: MessageSquare },
    { id: "web_search" as ChatMode, label: "Web Search", icon: Globe },
    { id: "deep_research" as ChatMode, label: "Deep Research", icon: Atom },
  ];
</script>

<div class="inline-flex items-center p-0.5 rounded-lg border border-border bg-card/60 font-sans select-none text-xs">
  {#each modes as m}
    <button
      type="button"
      onclick={() => onSelectMode(m.id)}
      class="flex items-center gap-1.5 h-7 px-2.5 rounded-md font-medium transition-all duration-150 cursor-pointer {activeMode === m.id ? 'bg-primary/10 text-primary border border-primary/25 shadow-xs font-semibold' : 'text-muted-foreground hover:text-foreground'}"
      title={m.label}
    >
      <m.icon class="h-3.5 w-3.5" />
      <span class="hidden sm:inline text-[11px]">{m.label}</span>
    </button>
  {/each}
</div>
