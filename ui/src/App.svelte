<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "./lib/api";
  import type { Message, Session, User } from "./lib/types";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Textarea } from "$lib/components/ui/textarea";
  import Zap from "lucide-svelte/icons/zap";
  import Plus from "lucide-svelte/icons/plus";
  import Menu from "lucide-svelte/icons/menu";
  import X from "lucide-svelte/icons/x";
  import Send from "lucide-svelte/icons/send";
  import UserIcon from "lucide-svelte/icons/user";
  import Cpu from "lucide-svelte/icons/cpu";
  import Shield from "lucide-svelte/icons/shield";
  import Loader2 from "lucide-svelte/icons/loader-2";

  // Svelte 5 Runes
  let user = $state<User | null>(null);
  let sessions = $state<Session[]>([]);
  let activeSessionId = $state<string | null>(null);
  let messages = $state<Message[]>([]);
  let promptText = $state("");
  let isSending = $state(false);
  let errorMessage = $state<string | null>(null);
  let isMobileDrawerOpen = $state(false);

  let activeSession = $derived(
    sessions.find((s) => s.id === activeSessionId) || null
  );

  onMount(async () => {
    try {
      const auth = await api.getMe();
      if (auth.user) user = auth.user;
      await loadSessions();
    } catch (e) {
      console.error("Initialization error:", e);
    }
  });

  async function loadSessions() {
    try {
      const page = await api.listSessions();
      sessions = page.items;
      if (sessions.length > 0 && !activeSessionId) {
        selectSession(sessions[0].id);
      }
    } catch (e) {
      errorMessage = "Failed to load chat sessions";
    }
  }

  async function selectSession(id: string) {
    activeSessionId = id;
    isMobileDrawerOpen = false;
    try {
      const page = await api.listMessages(id);
      messages = page.items.reverse();
    } catch (e) {
      errorMessage = "Failed to load messages";
    }
  }

  async function handleCreateSession() {
    try {
      const newSession = await api.createSession(`Chat ${sessions.length + 1}`);
      sessions = [newSession, ...sessions];
      selectSession(newSession.id);
    } catch (e) {
      errorMessage = "Failed to create new session";
    }
  }

  async function handleSend() {
    if (!promptText.trim() || !activeSessionId || isSending) return;
    const content = promptText.trim();
    promptText = "";
    isSending = true;
    errorMessage = null;

    // Optimistic UI append (O(1))
    const localUserMsg: Message = {
      id: "temp_" + Date.now(),
      session_id: activeSessionId,
      role: "user",
      content,
      created_at: new Date().toISOString(),
    };
    messages = [...messages, localUserMsg];

    try {
      await api.sendMessage(activeSessionId, content);
      // Refresh messages
      const page = await api.listMessages(activeSessionId);
      messages = page.items.reverse();
    } catch (e: any) {
      errorMessage = e.message || "Failed to send message";
    } finally {
      isSending = false;
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }
</script>

<div class="flex h-screen w-screen bg-background text-foreground antialiased select-none font-sans overflow-hidden">
  <!-- Mobile Backdrop Overlay (120Hz/144Hz GPU Composited) -->
  {#if isMobileDrawerOpen}
    <button
      type="button"
      class="fixed inset-0 bg-slate-950/80 backdrop-blur-sm z-40 md:hidden gpu-layer will-change-opacity transition-opacity duration-200 cursor-default w-full h-full border-0 p-0"
      onclick={() => (isMobileDrawerOpen = false)}
      aria-label="Close navigation sidebar"
    ></button>
  {/if}

  <!-- Sidebar (shadcn-svelte + 120Hz/144Hz Native GPU Compositor) -->
  <aside
    class="fixed md:static inset-y-0 left-0 z-50 w-72 bg-card border-r border-border flex flex-col justify-between gpu-layer transform-gpu will-change-transform ease-spring-120fps transition-transform duration-200 shadow-xl md:shadow-none {isMobileDrawerOpen ? 'translate-x-0' : '-translate-x-full md:translate-x-0'}"
  >
    <!-- Sidebar Header -->
    <div class="h-16 px-4 border-b border-border flex items-center justify-between">
      <div class="flex items-center gap-2.5">
        <div class="p-1.5 rounded-lg bg-cyan-950/60 border border-cyan-800/40 text-cyan-400">
          <Zap class="w-5 h-5" />
        </div>
        <span class="font-bold text-lg tracking-tight bg-gradient-to-r from-cyan-400 to-blue-500 bg-clip-text text-transparent">
          Lumis
        </span>
      </div>
      <div class="flex items-center gap-1.5">
        <Button
          variant="cyan"
          size="touch"
          onclick={handleCreateSession}
          class="gap-1.5 text-xs font-medium"
          title="Create New Chat"
          aria-label="Create New Chat"
        >
          <Plus class="w-3.5 h-3.5" />
          <span>Baru</span>
        </Button>
        <Button
          variant="ghost"
          size="icon"
          onclick={() => (isMobileDrawerOpen = false)}
          class="md:hidden text-muted-foreground hover:text-foreground"
          aria-label="Close Drawer"
        >
          <X class="w-5 h-5" />
        </Button>
      </div>
    </div>

    <!-- Session List (Smooth Momentum Scrolling) -->
    <div class="flex-1 overflow-y-auto p-2 space-y-1 smooth-momentum-scroll">
      {#each sessions as s (s.id)}
        <Button
          variant={activeSessionId === s.id ? "secondary" : "ghost"}
          onclick={() => selectSession(s.id)}
          class="w-full min-h-[44px] justify-start px-3 py-2.5 text-sm truncate break-words {activeSessionId === s.id ? 'text-cyan-400 font-medium' : 'text-muted-foreground hover:text-foreground'}"
        >
          <span class="truncate">{s.title}</span>
        </Button>
      {/each}
    </div>

    <!-- User Profile Footer -->
    <div class="p-3 border-t border-border text-xs text-muted-foreground flex items-center justify-between bg-card/60">
      <div class="flex items-center gap-2 truncate">
        <UserIcon class="w-4 h-4 text-muted-foreground shrink-0" />
        <span class="truncate font-medium text-slate-300">{user ? user.username : "Connecting..."}</span>
      </div>
      <Badge variant="cyan" class="font-mono text-[11px] shrink-0 gap-1">
        <Shield class="w-3 h-3 text-cyan-400" />
        {user ? user.role : "Guest"}
      </Badge>
    </div>
  </aside>

  <!-- Main Chat Area -->
  <main class="flex-1 flex flex-col justify-between bg-background overflow-hidden">
    <!-- Chat Header with Mobile Hamburger -->
    <header class="h-16 border-b border-border px-4 md:px-6 flex items-center justify-between shrink-0 bg-background/80 backdrop-blur-md">
      <div class="flex items-center gap-3">
        <Button
          variant="outline"
          size="icon"
          onclick={() => (isMobileDrawerOpen = true)}
          class="md:hidden"
          aria-label="Open Sidebar Menu"
        >
          <Menu class="w-5 h-5" />
        </Button>
        <div class="font-medium text-foreground truncate max-w-xs md:max-w-md">
          {activeSession ? activeSession.title : "Pilih Percakapan"}
        </div>
      </div>
      <Badge variant="outline" class="font-mono text-xs gap-1.5 py-1.5 px-3">
        <Cpu class="w-3.5 h-3.5 text-cyan-400" />
        <span class="hidden sm:inline text-muted-foreground">Model:</span>
        <span class="text-foreground font-medium">{activeSession ? activeSession.model_id : "Standard"}</span>
      </Badge>
    </header>

    <!-- Error Banner -->
    {#if errorMessage}
      <div class="bg-destructive/20 border-b border-destructive/40 text-red-400 text-xs px-6 py-2.5 break-words [overflow-wrap:anywhere]">
        {errorMessage}
      </div>
    {/if}

    <!-- Message Bubble Stream (Word-Wrap Aware + 120Hz Smooth Scroll) -->
    <div class="flex-1 overflow-y-auto p-4 md:p-6 space-y-4 smooth-momentum-scroll gpu-layer">
      {#each messages as m (m.id)}
        <div class="flex {m.role === 'user' ? 'justify-end' : 'justify-start'}">
          <div
            class="max-w-2xl px-4 py-3 rounded-2xl text-sm leading-relaxed shadow-sm break-words whitespace-pre-wrap [overflow-wrap:anywhere] {m.role === 'user' ? 'bg-cyan-600 text-white rounded-br-none' : 'bg-card border border-border text-card-foreground rounded-bl-none'}"
          >
            {m.content}
          </div>
        </div>
      {/each}
    </div>

    <!-- Input Bar -->
    <footer class="p-3 md:p-4 border-t border-border bg-background shrink-0">
      <div class="max-w-4xl mx-auto flex items-end gap-2">
        <Textarea
          bind:value={promptText}
          onkeydown={handleKeyDown}
          placeholder="Tanyakan sesuatu... (Enter untuk kirim, Shift+Enter untuk baris baru)"
          class="flex-1 min-h-[48px] max-h-36 resize-none"
          rows={1}
        />
        <Button
          variant="cyan"
          size="touch"
          onclick={handleSend}
          disabled={isSending || !promptText.trim()}
          class="min-h-[48px] px-5 gap-2"
          aria-label="Send Message"
        >
          {#if isSending}
            <Loader2 class="w-4 h-4 animate-spin" />
          {:else}
            <Send class="w-4 h-4" />
            <span class="hidden sm:inline">Kirim</span>
          {/if}
        </Button>
      </div>
    </footer>
  </main>
</div>
