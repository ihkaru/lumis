<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { api } from "./lib/api";
  import { router } from "./lib/router.svelte";
  import type { ArtifactSpecification, Message, Session, User } from "./lib/types";
  import ErrorBoundary from "./lib/components/common/ErrorBoundary.svelte";
  import LoginPage from "./lib/components/auth/LoginPage.svelte";
  import MarkdownScratchpad from "./lib/components/chat/MarkdownScratchpad.svelte";
  import NewChatWelcome from "./lib/components/chat/NewChatWelcome.svelte";
  import Sidebar from "./lib/components/sidebar/Sidebar.svelte";
  import UserBubble from "./lib/components/message/UserBubble.svelte";
  import AssistantBubble from "./lib/components/message/AssistantBubble.svelte";
  import MessageInput from "./lib/components/input/MessageInput.svelte";
  import type { ChatMode } from "./lib/components/input/ModeSelector.svelte";
  import type { AttachedFile } from "./lib/components/input/FileDropzone.svelte";
  import SettingsModal from "./lib/components/settings/SettingsModal.svelte";
  import type { ThemePalette } from "./lib/components/settings/ThemesTab.svelte";
  import { Canvas } from "$lib/components/ui/canvas";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import Menu from "lucide-svelte/icons/menu";
  import Cpu from "lucide-svelte/icons/cpu";
  import FileEdit from "lucide-svelte/icons/file-edit";

  let brandTitle = $state("Lumis");
  let user = $state<User | null>(null);
  let sessions = $state<Session[]>([]);
  let activeSessionId = $state<string | null>(null);
  let messages = $state<Message[]>([]);
  let promptText = $state("");
  let isGenerating = $state(false);
  let selectedModelId = $state("my9model-free");
  let chatMode = $state<ChatMode>("normal");
  let errorMessage = $state<string | null>(null);
  let isMobileDrawerOpen = $state(typeof window !== "undefined" ? window.innerWidth >= 768 : true);
  let isSettingsOpen = $state(false);
  let isDark = $state(true);
  let themePalette = $state<ThemePalette>("llumen");
  let activeArtifact = $state<ArtifactSpecification | null>(null);
  let activeAbortController: AbortController | null = null;
  let eventSource: EventSource | null = null;
  let messageContainerEl = $state<HTMLDivElement | null>(null);

  let activeSession = $derived(
    sessions.find((s) => s.id === activeSessionId) || null
  );

  onMount(async () => {
    try {
      const brand = await api.getBranding().catch(() => null);
      if (brand?.app_title) {
        brandTitle = brand.app_title;
        document.title = `${brand.app_title} — Enterprise AI`;
      }
      const auth = await api.getMe().catch(() => ({ authenticated: false }));
      if (auth.authenticated && auth.user) {
        user = auth.user;
      }
      await loadSessions();

      // Route synchronization on initial load
      if (router.params.id) {
        selectSession(router.params.id);
      } else if (router.path === "/" && sessions.length > 0) {
        selectSession(sessions[0].id);
      }

      // 9router Real-Time Zero-Copy SSE Stream listener
      eventSource = api.connectStream((raw) => {
        try {
          const data = JSON.parse(raw);
          if (data.event === "chunk" && data.session_id === activeSessionId) {
            const lastIdx = messages.length - 1;
            if (lastIdx >= 0 && messages[lastIdx].role === "assistant") {
              messages[lastIdx].content += data.delta;
              messages = [...messages];
            } else {
              messages = [...messages, { id: "stream_" + Date.now(), session_id: activeSessionId, role: "assistant", content: data.delta, created_at: new Date().toISOString() }];
            }
            scrollToBottom();
            if (data.is_done) isGenerating = false;
          } else if (data.event === "error" && data.session_id === activeSessionId) {
            errorMessage = data.error;
            isGenerating = false;
          }
        } catch {}
      });
    } catch (e) {
      console.error("Init error:", e);
    }
  });

  onDestroy(() => { eventSource?.close(); });

  // Watch URL params for deep-link navigation
  $effect(() => {
    if (router.params.id && router.params.id !== activeSessionId) {
      selectSession(router.params.id);
    }
  });

  async function loadSessions() {
    try {
      const page = await api.listSessions();
      sessions = page.items;
    } catch {
      errorMessage = "Gagal memuat sesi percakapan.";
    }
  }

  async function selectSession(id: string) {
    activeSessionId = id;
    if (typeof window !== "undefined" && window.innerWidth < 768) isMobileDrawerOpen = false;
    errorMessage = null;
    router.navigate(`/chat/${id}`);

    try {
      const page = await api.listMessages(id);
      messages = page.items.reverse();
      scrollToBottom();
    } catch {
      errorMessage = "Gagal memuat pesan.";
    }
  }

  async function handleCreateSession(customTitle?: string) {
    try {
      const title = customTitle || `Chat ${sessions.length + 1}`;
      const newSess = await api.createSession(title, selectedModelId);
      sessions = [newSess, ...sessions];
      selectSession(newSess.id);
    } catch {
      errorMessage = "Gagal membuat sesi baru.";
    }
  }

  async function handleSend(
    text: string,
    files: AttachedFile[],
    mode: ChatMode,
    modelId: string
  ) {
    if (!activeSessionId) {
      await handleCreateSession(text.slice(0, 30));
    }
    if (!activeSessionId || isGenerating) return;

    isGenerating = true;
    errorMessage = null;
    activeAbortController = new AbortController();

    // Optimistic UI append for User
    const localUserMsg: Message = { id: "temp_" + Date.now(), session_id: activeSessionId, role: "user", content: text, created_at: new Date().toISOString() };
    messages = [...messages, localUserMsg];
    scrollToBottom();

    try {
      await api.sendMessage(activeSessionId, text, modelId);
      messages = [...messages, { id: "stream_" + Date.now(), session_id: activeSessionId, role: "assistant", content: "", created_at: new Date().toISOString() }];
      scrollToBottom();
    } catch (err: any) {
      if (err.name !== "AbortError") errorMessage = err.message || "Gagal mengirimkan pesan.";
      isGenerating = false;
    } finally {
      activeAbortController = null;
    }
  }

  function handleStop() { activeAbortController?.abort(); activeAbortController = null; isGenerating = false; }
  function scrollToBottom() { setTimeout(() => { if (messageContainerEl) messageContainerEl.scrollTop = messageContainerEl.scrollHeight; }, 50); }
  function renameSession(id: string, newTitle: string) { sessions = sessions.map((s) => (s.id === id ? { ...s, title: newTitle } : s)); }
  function deleteSession(id: string) { sessions = sessions.filter((s) => s.id !== id); if (activeSessionId === id) { activeSessionId = null; messages = []; router.navigate("/chat/new"); } }
  function togglePin(target: Session) { sessions = sessions.map((s) => (s.id === target.id ? { ...s, is_pinned: !s.is_pinned } : s)); }
</script>

<ErrorBoundary>
  <!-- Route: Login Screen -->
  {#if router.path === "/login"}
    <LoginPage
      {brandTitle}
      onLoginSuccess={(uname) => {
        user = { id: uname, username: uname, role: "member" };
        router.navigate("/chat/new");
      }}
    />
  <!-- Route: Standalone Markdown Preview -->
  {:else if router.path === "/markdown"}
    <MarkdownScratchpad />
  <!-- Main Application Layout -->
  {:else}
    <div class="flex h-screen w-screen bg-background text-foreground antialiased font-sans overflow-hidden {isDark ? 'dark' : ''}">
      <!-- Sidebar -->
      <Sidebar
        {brandTitle}
        {sessions}
        {activeSessionId}
        {user}
        isOpenMobile={isMobileDrawerOpen}
        onSelectSession={selectSession}
        onCreateSession={() => {
          if (typeof window !== "undefined" && window.innerWidth < 768) isMobileDrawerOpen = false;
          router.navigate("/chat/new");
        }}
        onCloseMobile={() => (isMobileDrawerOpen = false)}
        onRenameSession={renameSession}
        onDeleteSession={deleteSession}
        onTogglePinSession={togglePin}
        onOpenSettings={() => {
          if (typeof window !== "undefined" && window.innerWidth < 768) isMobileDrawerOpen = false;
          isSettingsOpen = true;
        }}
      />

      <!-- Main Chat Area -->
      <main class="flex-1 flex flex-col justify-between bg-background overflow-hidden">
        <!-- Chat Header -->
        <header class="h-16 border-b border-border px-4 md:px-6 flex items-center justify-between shrink-0 bg-background/80 backdrop-blur-md select-none">
          <div class="flex items-center gap-3 min-w-0">
            <Button
              variant="outline"
              size="icon"
              onclick={() => (isMobileDrawerOpen = !isMobileDrawerOpen)}
              class="{isMobileDrawerOpen ? 'md:hidden' : ''} min-h-[44px] min-w-[44px] sm:min-h-9 sm:min-w-9"
              aria-label="Buka Menu"
            >
              <Menu class="w-5 h-5" />
            </Button>
            <div class="font-medium text-foreground truncate max-w-xs md:max-w-md">
              {activeSession ? activeSession.title : "Percakapan Baru"}
            </div>
          </div>

          <div class="flex items-center gap-2">
            <Button
              variant="ghost"
              size="sm"
              onclick={() => router.navigate("/markdown")}
              class="gap-1.5 text-xs text-muted-foreground hover:text-foreground hidden sm:flex"
              title="Buka Markdown Scratchpad"
            >
              <FileEdit class="w-3.5 h-3.5 text-cyan-400" />
              <span>Scratchpad</span>
            </Button>

            <Badge variant="outline" class="font-mono text-xs gap-1.5 py-1.5 px-3">
              <Cpu class="w-3.5 h-3.5 text-cyan-400" />
              <span class="text-foreground font-medium">{selectedModelId}</span>
            </Badge>
          </div>
        </header>

        <!-- Error Notification Banner -->
        {#if errorMessage}
          <div class="bg-destructive/20 border-b border-destructive/40 text-red-400 text-xs px-6 py-2.5 break-words [overflow-wrap:anywhere]">
            {errorMessage}
          </div>
        {/if}

        <!-- Message Viewport -->
        <div
          bind:this={messageContainerEl}
          class="flex-1 overflow-y-auto p-4 md:p-6 space-y-5 smooth-momentum-scroll gpu-layer"
        >
          {#if router.path === "/chat/new" || (!activeSessionId && messages.length === 0)}
            <NewChatWelcome
              {brandTitle}
              onSelectPrompt={(text) => {
                promptText = text;
              }}
            />
          {:else}
            {#each messages as m (m.id)}
              {#if m.role === "user"}
                <UserBubble message={m} onEditPrompt={(t) => (promptText = t)} />
              {:else}
                <AssistantBubble
                  message={m}
                  isStreaming={isGenerating && m.id === messages[messages.length - 1]?.id}
                  onOpenArtifact={(art) => (activeArtifact = art)}
                  onRegenerate={() => {
                    const lastUserMsg = [...messages].reverse().find((x) => x.role === "user");
                    if (lastUserMsg) handleSend(lastUserMsg.content, [], chatMode, selectedModelId);
                  }}
                />
              {/if}
            {/each}
          {/if}
        </div>

        <!-- Input Bar Composer -->
        <footer class="p-3 md:p-4 border-t border-border bg-background shrink-0">
          <MessageInput
            bind:promptText
            {isGenerating}
            {selectedModelId}
            {chatMode}
            onSend={handleSend}
            onStop={handleStop}
            onModelChange={(m) => (selectedModelId = m)}
            onModeChange={(md) => (chatMode = md)}
          />
        </footer>
      </main>

      <!-- Artifact Split-Pane Canvas -->
      {#if activeArtifact}
        <div class="fixed md:static inset-0 md:inset-auto z-40 md:z-20 w-full md:w-[480px] lg:w-[560px] xl:w-[640px] h-full shrink-0">
          <Canvas
            artifact={activeArtifact}
            sessionId={activeSessionId || undefined}
            onClose={() => (activeArtifact = null)}
          />
        </div>
      {/if}

      <!-- Settings Modal -->
      <SettingsModal
        isOpen={isSettingsOpen}
        {user}
        {isDark}
        palette={themePalette}
        onClose={() => (isSettingsOpen = false)}
        onLogout={() => {
          user = null;
          isSettingsOpen = false;
          router.navigate("/login");
        }}
        onToggleDark={(d) => (isDark = d)}
        onSelectPalette={(p) => (themePalette = p)}
      />
    </div>
  {/if}
</ErrorBoundary>
