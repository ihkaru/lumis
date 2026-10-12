<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "./lib/api";
  import type { Message, Session, User } from "./lib/types";

  // Svelte 5 Runes
  let user = $state<User | null>(null);
  let sessions = $state<Session[]>([]);
  let activeSessionId = $state<string | null>(null);
  let messages = $state<Message[]>([]);
  let promptText = $state("");
  let isSending = $state(false);
  let errorMessage = $state<string | null>(null);

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

<div class="flex h-screen w-screen bg-slate-950 text-slate-100 antialiased select-none">
  <!-- Sidebar -->
  <aside class="w-72 bg-slate-900 border-r border-slate-800 flex flex-col justify-between">
    <div class="p-4 border-b border-slate-800 flex items-center justify-between">
      <div class="flex items-center gap-2">
        <span class="text-xl">⚡</span>
        <h1 class="font-bold text-lg tracking-tight bg-gradient-to-r from-cyan-400 to-blue-500 bg-clip-text text-transparent">
          Lumis
        </h1>
      </div>
      <button
        onclick={handleCreateSession}
        class="p-1.5 text-xs bg-cyan-600 hover:bg-cyan-500 rounded-md font-medium transition"
        title="New Chat"
      >
        + New
      </button>
    </div>

    <!-- Session List -->
    <div class="flex-1 overflow-y-auto p-2 space-y-1">
      {#each sessions as s (s.id)}
        <button
          onclick={() => selectSession(s.id)}
          class="w-full text-left px-3 py-2.5 rounded-lg text-sm truncate transition {activeSessionId === s.id ? 'bg-slate-800 text-cyan-400 font-medium' : 'text-slate-400 hover:bg-slate-800/50 hover:text-slate-200'}"
        >
          {s.title}
        </button>
      {/each}
    </div>

    <!-- User Profile Footer -->
    <div class="p-3 border-t border-slate-800 text-xs text-slate-400 flex items-center justify-between">
      <span class="truncate">{user ? user.username : "Connecting..."}</span>
      <span class="px-2 py-0.5 rounded bg-slate-800 text-cyan-400 border border-slate-700">
        {user ? user.role : "Guest"}
      </span>
    </div>
  </aside>

  <!-- Main Chat Area -->
  <main class="flex-1 flex flex-col justify-between bg-slate-950">
    <!-- Chat Header -->
    <header class="h-14 border-b border-slate-800 px-6 flex items-center justify-between">
      <div class="font-medium text-slate-200">
        {activeSession ? activeSession.title : "Select a Chat"}
      </div>
      <div class="text-xs text-slate-400 bg-slate-900 px-2.5 py-1 rounded-full border border-slate-800">
        Model: {activeSession ? activeSession.model_id : "Standard"}
      </div>
    </header>

    <!-- Error Banner -->
    {#if errorMessage}
      <div class="bg-red-500/10 border-b border-red-500/20 text-red-400 text-xs px-6 py-2">
        {errorMessage}
      </div>
    {/if}

    <!-- Message Bubble Stream -->
    <div class="flex-1 overflow-y-auto p-6 space-y-4">
      {#each messages as m (m.id)}
        <div class="flex {m.role === 'user' ? 'justify-end' : 'justify-start'}">
          <div
            class="max-w-2xl px-4 py-3 rounded-2xl text-sm leading-relaxed shadow-sm {m.role === 'user' ? 'bg-cyan-600 text-white rounded-br-none' : 'bg-slate-900 border border-slate-800 text-slate-200 rounded-bl-none'}"
          >
            {m.content}
          </div>
        </div>
      {/each}
    </div>

    <!-- Input Bar -->
    <footer class="p-4 border-t border-slate-800 bg-slate-950">
      <div class="max-w-4xl mx-auto flex gap-2">
        <textarea
          bind:value={promptText}
          onkeydown={handleKeyDown}
          placeholder="Tanyakan sesuatu... (Enter untuk kirim, Shift+Enter untuk baris baru)"
          class="flex-1 bg-slate-900 border border-slate-800 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-cyan-500 resize-none h-12 text-slate-100 placeholder-slate-500"
          rows="1"
        ></textarea>
        <button
          onclick={handleSend}
          disabled={isSending || !promptText.trim()}
          class="px-5 py-2.5 bg-cyan-600 hover:bg-cyan-500 disabled:opacity-50 text-white text-sm font-medium rounded-xl transition shadow"
        >
          {isSending ? "..." : "Kirim"}
        </button>
      </div>
    </footer>
  </main>
</div>
