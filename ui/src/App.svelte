<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "./lib/api";
  import Icon from "./lib/Icon.svelte";
  import type { Message, Session, User } from "./lib/types";

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

<div class="flex h-screen w-screen bg-slate-950 text-slate-100 antialiased select-none font-sans overflow-hidden">
  <!-- Mobile Backdrop Overlay -->
  {#if isMobileDrawerOpen}
    <button
      type="button"
      class="fixed inset-0 bg-slate-950/80 backdrop-blur-sm z-40 md:hidden transition-opacity cursor-default w-full h-full border-0 p-0"
      onclick={() => (isMobileDrawerOpen = false)}
      aria-label="Close navigation sidebar"
    ></button>
  {/if}

  <!-- Sidebar (Responsive Off-Canvas Drawer on Mobile) -->
  <aside
    class="fixed md:static inset-y-0 left-0 z-50 w-72 bg-slate-900 border-r border-slate-800 flex flex-col justify-between transition-transform duration-200 ease-in-out shadow-xl md:shadow-none {isMobileDrawerOpen ? 'translate-x-0' : '-translate-x-full md:translate-x-0'}"
  >
    <!-- Sidebar Header -->
    <div class="h-16 px-4 border-b border-slate-800 flex items-center justify-between">
      <div class="flex items-center gap-2.5">
        <div class="p-1.5 rounded-lg bg-cyan-950/60 border border-cyan-800/40 text-cyan-400">
          <Icon name="spark" class="w-5 h-5" />
        </div>
        <span class="font-bold text-lg tracking-tight bg-gradient-to-r from-cyan-400 to-blue-500 bg-clip-text text-transparent">
          Lumis
        </span>
      </div>
      <div class="flex items-center gap-1.5">
        <button
          onclick={handleCreateSession}
          class="min-h-[44px] min-w-[44px] px-3 flex items-center justify-center gap-1.5 text-xs bg-cyan-600 hover:bg-cyan-500 active:bg-cyan-700 text-white rounded-lg font-medium transition shadow-sm"
          title="Create New Chat"
          aria-label="Create New Chat"
        >
          <Icon name="plus" class="w-3.5 h-3.5" />
          <span>Baru</span>
        </button>
        <button
          onclick={() => (isMobileDrawerOpen = false)}
          class="md:hidden min-h-[44px] min-w-[44px] flex items-center justify-center text-slate-400 hover:text-slate-200 rounded-lg"
          aria-label="Close Drawer"
        >
          <Icon name="close" class="w-5 h-5" />
        </button>
      </div>
    </div>

    <!-- Session List -->
    <div class="flex-1 overflow-y-auto p-2 space-y-1">
      {#each sessions as s (s.id)}
        <button
          onclick={() => selectSession(s.id)}
          class="w-full min-h-[44px] text-left px-3 py-2.5 rounded-lg text-sm truncate break-words transition {activeSessionId === s.id ? 'bg-slate-800 text-cyan-400 font-medium' : 'text-slate-400 hover:bg-slate-800/50 hover:text-slate-200'}"
        >
          {s.title}
        </button>
      {/each}
    </div>

    <!-- User Profile Footer -->
    <div class="p-3 border-t border-slate-800 text-xs text-slate-400 flex items-center justify-between bg-slate-900/60">
      <div class="flex items-center gap-2 truncate">
        <Icon name="user" class="w-4 h-4 text-slate-500 shrink-0" />
        <span class="truncate font-medium text-slate-300">{user ? user.username : "Connecting..."}</span>
      </div>
      <span class="px-2 py-0.5 rounded bg-slate-800 text-cyan-400 border border-slate-700 font-mono text-[11px] shrink-0 flex items-center gap-1">
        <Icon name="shield" class="w-3 h-3 text-cyan-400" />
        {user ? user.role : "Guest"}
      </span>
    </div>
  </aside>

  <!-- Main Chat Area -->
  <main class="flex-1 flex flex-col justify-between bg-slate-950 overflow-hidden">
    <!-- Chat Header with Mobile Hamburger -->
    <header class="h-16 border-b border-slate-800 px-4 md:px-6 flex items-center justify-between shrink-0 bg-slate-950/80 backdrop-blur-md">
      <div class="flex items-center gap-3">
        <button
          onclick={() => (isMobileDrawerOpen = true)}
          class="md:hidden min-h-[44px] min-w-[44px] flex items-center justify-center text-slate-400 hover:text-slate-200 rounded-lg border border-slate-800 bg-slate-900"
          aria-label="Open Sidebar Menu"
        >
          <Icon name="menu" class="w-5 h-5" />
        </button>
        <div class="font-medium text-slate-200 truncate max-w-xs md:max-w-md">
          {activeSession ? activeSession.title : "Pilih Percakapan"}
        </div>
      </div>
      <div class="flex items-center gap-1.5 text-xs text-slate-400 bg-slate-900 px-3 py-1.5 rounded-full border border-slate-800 font-mono">
        <Icon name="cpu" class="w-3.5 h-3.5 text-cyan-400" />
        <span class="hidden sm:inline">Model:</span>
        <span class="text-slate-300 font-medium">{activeSession ? activeSession.model_id : "Standard"}</span>
      </div>
    </header>

    <!-- Error Banner -->
    {#if errorMessage}
      <div class="bg-red-500/10 border-b border-red-500/20 text-red-400 text-xs px-6 py-2.5 break-words [overflow-wrap:anywhere]">
        {errorMessage}
      </div>
    {/if}

    <!-- Message Bubble Stream (Word-Wrap Aware) -->
    <div class="flex-1 overflow-y-auto p-4 md:p-6 space-y-4">
      {#each messages as m (m.id)}
        <div class="flex {m.role === 'user' ? 'justify-end' : 'justify-start'}">
          <div
            class="max-w-2xl px-4 py-3 rounded-2xl text-sm leading-relaxed shadow-sm break-words whitespace-pre-wrap [overflow-wrap:anywhere] {m.role === 'user' ? 'bg-cyan-600 text-white rounded-br-none' : 'bg-slate-900 border border-slate-800 text-slate-200 rounded-bl-none'}"
          >
            {m.content}
          </div>
        </div>
      {/each}
    </div>

    <!-- Input Bar -->
    <footer class="p-3 md:p-4 border-t border-slate-800 bg-slate-950 shrink-0">
      <div class="max-w-4xl mx-auto flex items-end gap-2">
        <textarea
          bind:value={promptText}
          onkeydown={handleKeyDown}
          placeholder="Tanyakan sesuatu... (Enter untuk kirim, Shift+Enter untuk baris baru)"
          class="flex-1 bg-slate-900 border border-slate-800 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-cyan-500 resize-none min-h-[48px] max-h-36 text-slate-100 placeholder-slate-500 font-sans break-words [overflow-wrap:anywhere]"
          rows="1"
        ></textarea>
        <button
          onclick={handleSend}
          disabled={isSending || !promptText.trim()}
          class="min-h-[48px] px-5 flex items-center justify-center gap-2 bg-cyan-600 hover:bg-cyan-500 active:bg-cyan-700 disabled:opacity-50 text-white text-sm font-medium rounded-xl transition shadow"
          aria-label="Send Message"
        >
          {#if isSending}
            <Icon name="spinner" class="w-4 h-4" />
          {:else}
            <Icon name="send" class="w-4 h-4" />
            <span class="hidden sm:inline">Kirim</span>
          {/if}
        </button>
      </div>
    </footer>
  </main>
</div>
