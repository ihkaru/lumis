<script lang="ts">
  import Zap from "lucide-svelte/icons/zap";
  import Plus from "lucide-svelte/icons/plus";
  import X from "lucide-svelte/icons/x";
  import Settings from "lucide-svelte/icons/settings";
  import UserIcon from "lucide-svelte/icons/user";
  import Pin from "lucide-svelte/icons/pin";
  import SessionSearch from "./SessionSearch.svelte";
  import SessionItem from "./SessionItem.svelte";
  import SessionRenameDialog from "./SessionRenameDialog.svelte";
  import SessionDeleteDialog from "./SessionDeleteDialog.svelte";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import type { Session, User } from "$lib/types";

  interface Props {
    brandTitle?: string;
    sessions: Session[];
    activeSessionId: string | null;
    user: User | null;
    isOpenMobile: boolean;
    onSelectSession: (id: string) => void;
    onCreateSession: () => void;
    onCloseMobile: () => void;
    onRenameSession: (id: string, newTitle: string) => void;
    onDeleteSession: (id: string) => void;
    onTogglePinSession: (session: Session) => void;
    onOpenSettings: () => void;
  }

  let {
    brandTitle = "Lumis",
    sessions = [],
    activeSessionId = null,
    user,
    isOpenMobile = false,
    onSelectSession,
    onCreateSession,
    onCloseMobile,
    onRenameSession,
    onDeleteSession,
    onTogglePinSession,
    onOpenSettings,
  }: Props = $props();

  let searchQuery = $state("");
  let sessionToRename = $state<Session | null>(null);
  let sessionToDelete = $state<Session | null>(null);

  // Mobile Touch Swipe Gesture Detection
  let touchStartX = 0;
  let touchEndX = 0;

  function handleTouchStart(e: TouchEvent) {
    touchStartX = e.changedTouches[0].screenX;
  }

  function handleTouchEnd(e: TouchEvent) {
    touchEndX = e.changedTouches[0].screenX;
    // Left swipe closes sidebar
    if (touchStartX - touchEndX > 50 && isOpenMobile) {
      onCloseMobile();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && isOpenMobile) {
      onCloseMobile();
    }
  }

  // Filtered sessions
  let filteredSessions = $derived(
    searchQuery.trim()
      ? sessions.filter((s) => s.title.toLowerCase().includes(searchQuery.toLowerCase().trim()))
      : sessions
  );

  let pinnedSessions = $derived(filteredSessions.filter((s) => s.is_pinned));
  let regularSessions = $derived(filteredSessions.filter((s) => !s.is_pinned));
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- Mobile Backdrop Overlay with Tap-to-Dismiss -->
{#if isOpenMobile}
  <button
    type="button"
    class="fixed inset-0 z-40 bg-black/75 backdrop-blur-xs md:hidden border-0 p-0 w-full h-full cursor-default select-none transition-opacity duration-200"
    onclick={onCloseMobile}
    aria-label="Tutup Menu Navigasi"
  ></button>
{/if}

<aside
  class="fixed md:static inset-y-0 left-0 z-50 w-72 sm:w-80 bg-card border-r border-border flex flex-col justify-between gpu-layer transform-gpu will-change-transform ease-spring-120fps transition-transform duration-200 shadow-2xl md:shadow-none {isOpenMobile ? 'translate-x-0' : '-translate-x-full md:translate-x-0'} select-none font-sans"
  ontouchstart={handleTouchStart}
  ontouchend={handleTouchEnd}
  aria-label="Sidebar Navigasi"
>
  <!-- Sidebar Header -->
  <div class="h-16 px-4 border-b border-border flex items-center justify-between shrink-0">
    <div class="flex items-center gap-2 min-w-0 pr-2">
      <div class="p-1.5 rounded-lg bg-cyan-950/60 border border-cyan-800/40 text-cyan-400 shrink-0">
        <Zap class="w-5 h-5" />
      </div>
      <span class="font-bold text-base tracking-tight bg-gradient-to-r from-cyan-400 to-blue-500 bg-clip-text text-transparent truncate">
        {brandTitle}
      </span>
    </div>

    <!-- Header Actions (New Chat & Mobile Close Button) -->
    <div class="flex items-center gap-1.5 shrink-0">
      <Button
        variant="cyan"
        size="sm"
        onclick={() => {
          onCreateSession();
          if (isOpenMobile) onCloseMobile();
        }}
        class="h-9 px-2.5 gap-1 text-xs font-medium cursor-pointer"
        title="Buat Percakapan Baru"
        aria-label="Buat Percakapan Baru"
      >
        <Plus class="w-4 h-4" />
        <span class="hidden sm:inline">Baru</span>
      </Button>

      <!-- Prominent Mobile Close Button (>= 44px touch target) -->
      <button
        type="button"
        onclick={onCloseMobile}
        class="md:hidden min-h-[44px] min-w-[44px] h-11 w-11 flex items-center justify-center rounded-xl bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground transition-colors cursor-pointer border border-border/40 touch-manipulation"
        aria-label="Tutup Menu Navigasi"
        title="Tutup Menu Navigasi"
      >
        <X class="w-5 h-5" />
      </button>
    </div>
  </div>

  <!-- Search Input Bar -->
  <SessionSearch bind:searchQuery onSearchChange={(q) => (searchQuery = q)} />

  <!-- Session Lists Viewport -->
  <div class="flex-1 overflow-y-auto px-2 space-y-4 smooth-momentum-scroll">
    <!-- Pinned Sessions Group -->
    {#if pinnedSessions.length > 0}
      <div class="space-y-1">
        <div class="px-3 pt-1 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground flex items-center gap-1">
          <Pin class="h-3 w-3 text-cyan-400 rotate-45" />
          <span>Disematkan ({pinnedSessions.length})</span>
        </div>
        {#each pinnedSessions as s (s.id)}
          <SessionItem
            session={s}
            isActive={activeSessionId === s.id}
            onSelect={() => {
              onSelectSession(s.id);
              if (isOpenMobile) onCloseMobile();
            }}
            onTogglePin={onTogglePinSession}
            onOpenRename={(sess) => (sessionToRename = sess)}
            onOpenDelete={(sess) => (sessionToDelete = sess)}
          />
        {/each}
      </div>
    {/if}

    <!-- Regular Sessions Group -->
    <div class="space-y-1">
      {#if pinnedSessions.length > 0 && regularSessions.length > 0}
        <div class="px-3 pt-1 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
          Obrolan Terkini ({regularSessions.length})
        </div>
      {/if}

      {#if filteredSessions.length === 0}
        <div class="p-6 text-center text-xs text-muted-foreground">
          {searchQuery ? "Tidak ada obrolan yang cocok." : "Belum ada riwayat percakapan."}
        </div>
      {:else}
        {#each regularSessions as s (s.id)}
          <SessionItem
            session={s}
            isActive={activeSessionId === s.id}
            onSelect={() => {
              onSelectSession(s.id);
              if (isOpenMobile) onCloseMobile();
            }}
            onTogglePin={onTogglePinSession}
            onOpenRename={(sess) => (sessionToRename = sess)}
            onOpenDelete={(sess) => (sessionToDelete = sess)}
          />
        {/each}
      {/if}
    </div>
  </div>

  <!-- User Profile Footer with Settings Trigger -->
  <div class="p-3 border-t border-border text-xs text-muted-foreground flex items-center justify-between bg-card/70 shrink-0">
    <div class="flex items-center gap-2 truncate min-w-0 pr-2">
      <UserIcon class="w-4 h-4 text-cyan-400 shrink-0" />
      <span class="truncate font-medium text-foreground">{user ? user.username : "Pengguna"}</span>
      <Badge variant="cyan" class="font-mono text-[10px] shrink-0 py-0 px-1.5 uppercase">
        {user ? user.role : "Guest"}
      </Badge>
    </div>
    <Button
      variant="ghost"
      size="icon"
      onclick={() => {
        onOpenSettings();
        if (isOpenMobile) onCloseMobile();
      }}
      class="min-h-[44px] min-w-[44px] h-10 w-10 text-muted-foreground hover:text-foreground shrink-0 cursor-pointer touch-manipulation"
      title="Buka Pengaturan"
      aria-label="Buka Pengaturan"
    >
      <Settings class="w-4 h-4" />
    </Button>
  </div>
</aside>

<!-- Rename Modal -->
<SessionRenameDialog
  isOpen={sessionToRename !== null}
  initialTitle={sessionToRename?.title || ""}
  onClose={() => (sessionToRename = null)}
  onRename={(newTitle) => {
    if (sessionToRename) onRenameSession(sessionToRename.id, newTitle);
  }}
/>

<!-- Delete Modal -->
<SessionDeleteDialog
  isOpen={sessionToDelete !== null}
  sessionTitle={sessionToDelete?.title || ""}
  onClose={() => (sessionToDelete = null)}
  onConfirmDelete={() => {
    if (sessionToDelete) onDeleteSession(sessionToDelete.id);
  }}
/>
