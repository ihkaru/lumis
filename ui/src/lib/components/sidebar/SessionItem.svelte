<script lang="ts">
  import Pin from "lucide-svelte/icons/pin";
  import PinOff from "lucide-svelte/icons/pin-off";
  import Pencil from "lucide-svelte/icons/pencil";
  import Trash2 from "lucide-svelte/icons/trash-2";
  import type { Session } from "$lib/types";

  interface Props {
    session: Session;
    isActive: boolean;
    onSelect: () => void;
    onTogglePin: (session: Session) => void;
    onOpenRename: (session: Session) => void;
    onOpenDelete: (session: Session) => void;
  }

  let { session, isActive, onSelect, onTogglePin, onOpenRename, onOpenDelete }: Props = $props();
</script>

<div
  class="group relative flex items-center justify-between w-full min-h-[44px] px-3 py-2 rounded-xl text-xs font-sans transition-all select-none cursor-pointer {isActive ? 'bg-cyan-950/50 text-cyan-400 border border-cyan-800/40 shadow-xs' : 'text-muted-foreground hover:bg-muted/40 hover:text-foreground'}"
  onclick={onSelect}
  role="button"
  tabindex={0}
  onkeydown={(e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onSelect();
    }
  }}
>
  <div class="flex items-center gap-2 min-w-0 flex-1 pr-1">
    {#if session.is_pinned}
      <Pin class="h-3.5 w-3.5 text-cyan-400 shrink-0 rotate-45" />
    {/if}
    <span class="truncate font-medium {isActive ? 'text-cyan-400 font-semibold' : 'text-foreground'}">
      {session.title}
    </span>
  </div>

  <!-- Action Hover Buttons -->
  <div class="opacity-0 group-hover:opacity-100 flex items-center gap-0.5 shrink-0 transition-opacity">
    <button
      type="button"
      onclick={(e) => {
        e.stopPropagation();
        onTogglePin(session);
      }}
      class="p-1 rounded hover:bg-muted text-muted-foreground hover:text-cyan-400"
      title={session.is_pinned ? "Lepas sematan" : "Sematkan ke atas"}
      aria-label="Pin atau unpin sesi"
    >
      {#if session.is_pinned}
        <PinOff class="h-3 w-3" />
      {:else}
        <Pin class="h-3 w-3" />
      {/if}
    </button>

    <button
      type="button"
      onclick={(e) => {
        e.stopPropagation();
        onOpenRename(session);
      }}
      class="p-1 rounded hover:bg-muted text-muted-foreground hover:text-foreground"
      title="Ubah judul"
      aria-label="Ubah judul sesi"
    >
      <Pencil class="h-3 w-3" />
    </button>

    <button
      type="button"
      onclick={(e) => {
        e.stopPropagation();
        onOpenDelete(session);
      }}
      class="p-1 rounded hover:bg-muted text-muted-foreground hover:text-destructive"
      title="Hapus obrolan"
      aria-label="Hapus sesi"
    >
      <Trash2 class="h-3 w-3" />
    </button>
  </div>
</div>
