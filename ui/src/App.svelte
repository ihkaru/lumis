<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { api } from "./lib/api";
  import { router } from "./lib/router.svelte";
  import { processStreamEvent } from "./lib/streamHandler";
  import type { ArtifactSpecification, Message, Session, SessionTask, User } from "./lib/types";
  import ErrorBoundary from "./lib/components/common/ErrorBoundary.svelte";
  import LoginPage from "./lib/components/auth/LoginPage.svelte";
  import MarkdownScratchpad from "./lib/components/chat/MarkdownScratchpad.svelte";
  import Sidebar from "./lib/components/sidebar/Sidebar.svelte";
  import ChatHeader from "./lib/components/chat/ChatHeader.svelte";
  import MessageViewport from "./lib/components/chat/MessageViewport.svelte";
  import MessageInput from "./lib/components/input/MessageInput.svelte";
  import type { ChatMode } from "./lib/components/input/ModeSelector.svelte";
  import type { AttachedFile } from "./lib/components/input/FileDropzone.svelte";
  import SettingsModal from "./lib/components/settings/SettingsModal.svelte";
  import type { ThemePalette } from "./lib/components/settings/ThemesTab.svelte";
  import { Canvas } from "$lib/components/ui/canvas";

  let brandTitle = $state("Lumis"), user = $state<User | null>(null), sessions = $state<Session[]>([]), activeSessionId = $state<string | null>(null);
  let messages = $state<Message[]>([]), sessionTasks = $state<SessionTask[]>([]), promptText = $state(""), isGenerating = $state(false);
  let selectedModelId = $state(typeof localStorage !== "undefined" ? localStorage.getItem("lumis_selected_model") || "my9model-free" : "my9model-free");
  let chatMode = $state<ChatMode>("normal"), errorMessage = $state<string | null>(null);
  let isMobileDrawerOpen = $state(typeof window !== "undefined" ? window.innerWidth >= 768 : true);
  let isSettingsOpen = $state(false);
  let isDark = $state(
    typeof localStorage !== "undefined"
      ? localStorage.getItem("lumis_theme_mode") !== "light"
      : true
  );
  let themePalette = $state<ThemePalette>(
    typeof localStorage !== "undefined"
      ? (localStorage.getItem("lumis_theme_palette") as ThemePalette) || "llumen"
      : "llumen"
  );

  $effect(() => {
    if (typeof document !== "undefined") {
      document.documentElement.classList.toggle("dark", isDark);
      document.documentElement.classList.toggle("light", !isDark);
      document.documentElement.setAttribute("data-palette", themePalette);
    }
  });

  function applyThemeMode(dark: boolean) {
    isDark = dark;
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("lumis_theme_mode", dark ? "dark" : "light");
    }
  }

  function applyThemePalette(p: ThemePalette) {
    themePalette = p;
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("lumis_theme_palette", p);
    }
  }

  let activeArtifact = $state<ArtifactSpecification | null>(null);
  let messageViewport = $state<ReturnType<typeof MessageViewport> | null>(null);
  let activeAbortController: AbortController | null = null, eventSource: EventSource | null = null, isAuthChecking = $state(true);

  let activeSession = $derived(sessions.find((s) => s.id === activeSessionId) || null);

  onMount(async () => {
    // 1. Enterprise Route Guard (Svelte 5 Runes Navigation Pipeline)
    router.beforeEach(async (to) => {
      if (isAuthChecking) return true;
      const clean = to.split("?")[0];
      const isPublic = clean === "/login" || clean === "/markdown";
      if (!user && !isPublic) return `/login?redirect=${encodeURIComponent(clean)}`;
      if (user && clean === "/login") return "/chat/new";
      return true;
    });

    try {
      const brand = await api.getBranding().catch(() => null);
      if (brand?.app_title) {
        brandTitle = brand.app_title;
        document.title = `${brand.app_title} — Enterprise AI`;
      }
      const auth = await api.getMe().catch(() => ({ authenticated: false, user: undefined }));
      user = auth.authenticated && auth.user ? auth.user : null;
    } catch {
      user = null;
    } finally {
      isAuthChecking = false;
    }

    if (!user) {
      if (router.path !== "/login" && router.path !== "/markdown") {
        router.navigate(`/login?redirect=${encodeURIComponent(router.path)}`, true);
      }
    } else {
      await loadSessions();
      if (router.params.id) {
        selectSession(router.params.id);
      } else if (router.path === "/" && sessions.length > 0) {
        selectSession(sessions[0].id);
      }
    }

    // 9router Real-Time Zero-Copy SSE Stream listener
    eventSource = api.connectStream((raw) => {
      const res = processStreamEvent(raw, { activeSessionId, messages, isGenerating, sessions, errorMessage, tasks: sessionTasks });
      messages = res.messages; isGenerating = res.isGenerating; sessions = res.sessions;
      errorMessage = res.errorMessage; sessionTasks = res.tasks;
      if (res.shouldScroll) messageViewport?.autoScrollDuringStream();
    });

    if (typeof window !== "undefined" && window.visualViewport) {
      window.visualViewport.addEventListener("resize", updateAppHeight);
      window.visualViewport.addEventListener("scroll", updateAppHeight);
      updateAppHeight();
    }
  });

  onDestroy(() => {
    eventSource?.close();
    if (typeof window !== "undefined" && window.visualViewport) {
      window.visualViewport.removeEventListener("resize", updateAppHeight);
      window.visualViewport.removeEventListener("scroll", updateAppHeight);
    }
  });

  function updateAppHeight() {
    if (typeof window !== "undefined" && window.visualViewport) {
      document.documentElement.style.setProperty("--app-height", `${window.visualViewport.height}px`);
    }
  }

  async function startNewChat() {
    if (typeof window !== "undefined" && window.innerWidth < 768) isMobileDrawerOpen = false;
    if (isGenerating) handleStop();
    errorMessage = null;
    await router.navigate("/chat/new");
    activeSessionId = null;
    messages = [];
    sessionTasks = [];
  }

  // Watch URL params for deep-link navigation
  $effect(() => {
    const currentPath = router.path;
    const targetId = router.params.id;
    if (currentPath === "/chat/new" || (!targetId && currentPath === "/")) {
      if (activeSessionId !== null && !isGenerating) {
        activeSessionId = null;
        messages = [];
        sessionTasks = [];
      }
    } else if (targetId && targetId !== "new" && targetId !== activeSessionId) {
      selectSession(targetId);
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
    if (id === "new") {
      await startNewChat();
      return;
    }
    if (activeSessionId === id && router.path === `/chat/${id}`) return;
    if (isGenerating) handleStop();
    activeSessionId = id;
    if (typeof window !== "undefined" && window.innerWidth < 768) isMobileDrawerOpen = false;
    errorMessage = null;
    await router.navigate(`/chat/${id}`);

    const sess = sessions.find((s) => s.id === id);
    if (sess?.model_id) {
      selectedModelId = sess.model_id;
    }

    try {
      const [page, tasks] = await Promise.all([api.listMessages(id), api.listSessionTasks(id).catch(() => [])]);
      messages = page.items.reverse(); sessionTasks = tasks; messageViewport?.scrollToBottom();
    } catch {
      messages = []; sessionTasks = [];
    }
  }

  async function handleCreateSession(customTitle?: string): Promise<Session | null> {
    try {
      const title = customTitle || `Chat ${sessions.length + 1}`;
      const newSess = await api.createSession(title, selectedModelId);
      sessions = [newSess, ...sessions]; activeSessionId = newSess.id;
      if (typeof window !== "undefined" && window.innerWidth < 768) isMobileDrawerOpen = false;
      errorMessage = null; router.navigate(`/chat/${newSess.id}`, true); return newSess;
    } catch (err: unknown) {
      errorMessage = err instanceof Error ? err.message : "Gagal membuat sesi baru."; return null;
    }
  }

  async function handleSend(text: string, files: AttachedFile[], mode: ChatMode, modelId: string) {
    if (!user) { router.navigate(`/login?redirect=${encodeURIComponent(router.path)}`); return; }
    if (isGenerating) return;

    let targetSessionId = activeSessionId;
    if (!targetSessionId || targetSessionId === "new") {
      const newSess = await handleCreateSession(text.slice(0, 30));
      if (!newSess) return;
      targetSessionId = newSess.id;
    }

    isGenerating = true; errorMessage = null;
    activeAbortController = new AbortController();

    // Optimistic UI append for User
    const localUserMsg: Message = { id: "temp_" + Date.now(), session_id: targetSessionId, role: "user", content: text, created_at: new Date().toISOString() };
    messages = [...messages, localUserMsg];
    messageViewport?.scrollUserMessageToTop(localUserMsg.id);

    try {
      await api.sendMessage(targetSessionId, text, modelId);
      messages = [...messages, { id: "stream_" + Date.now(), session_id: targetSessionId, role: "assistant", content: "", model_id: modelId, created_at: new Date().toISOString() }];
    } catch (err: unknown) {
      const errObj = err as { name?: string; message?: string };
      if (errObj?.name !== "AbortError") errorMessage = errObj?.message || "Gagal mengirimkan pesan.";
      isGenerating = false;
    } finally {
      activeAbortController = null;
    }
  }

  function handleStop() { activeAbortController?.abort(); activeAbortController = null; isGenerating = false; messageViewport?.resetScrollState(); }
  function renameSession(id: string, newTitle: string) { sessions = sessions.map((s) => (s.id === id ? { ...s, title: newTitle } : s)); }
  function deleteSession(id: string) { sessions = sessions.filter((s) => s.id !== id); if (activeSessionId === id) { startNewChat(); } }
  function togglePin(target: Session) { sessions = sessions.map((s) => (s.id === target.id ? { ...s, is_pinned: !s.is_pinned } : s)); }
</script>

<ErrorBoundary>
  <!-- Route: Login Screen -->
  {#if router.path === "/login"}
    <LoginPage
      {brandTitle}
      onLoginSuccess={async (uname) => {
        const auth = await api.getMe().catch(() => null);
        user = auth?.authenticated && auth.user ? auth.user : { id: uname, username: uname, role: "member" };
        await loadSessions();
      }}
    />
  <!-- Route: Standalone Markdown Preview -->
  {:else if router.path === "/markdown"}
    <MarkdownScratchpad />
  <!-- Main Application Layout -->
  {:else}
    <div class="flex h-[100dvh] max-h-[100dvh] w-screen bg-background text-foreground antialiased font-sans overflow-hidden {isDark ? 'dark' : 'light'}" style="height: var(--app-height, 100dvh); max-height: var(--app-height, 100dvh);">
      <!-- Sidebar -->
      <Sidebar
        {brandTitle} {sessions} {activeSessionId} {user} isOpenMobile={isMobileDrawerOpen} onSelectSession={selectSession}
        onCreateSession={startNewChat}
        onCloseMobile={() => (isMobileDrawerOpen = false)}
        onRenameSession={renameSession} onDeleteSession={deleteSession} onTogglePinSession={togglePin}
        onOpenSettings={() => { if (typeof window !== "undefined" && window.innerWidth < 768) isMobileDrawerOpen = false; isSettingsOpen = true; }}
      />

      <!-- Main Chat Area -->
      <main class="flex-1 flex flex-col justify-between bg-background overflow-hidden">
        <ChatHeader
          title={activeSession ? activeSession.title : "Percakapan Baru"}
          {selectedModelId}
          {isMobileDrawerOpen}
          onToggleMobileDrawer={() => (isMobileDrawerOpen = !isMobileDrawerOpen)}
          onOpenScratchpad={() => router.navigate("/markdown")}
        />

        <!-- Error Notification Banner -->
        {#if errorMessage}
          <div class="bg-destructive/20 border-b border-destructive/40 text-red-400 text-xs px-6 py-2.5 break-words [overflow-wrap:anywhere]">{errorMessage}</div>
        {/if}

        <!-- Message Viewport -->
        <MessageViewport
          bind:this={messageViewport}
          {messages}
          {sessionTasks}
          {activeSessionId}
          {isGenerating}
          {brandTitle}
          isNewChatRoute={router.path === "/chat/new"}
          onSelectPrompt={(t) => (promptText = t)}
          onEditPrompt={(t) => (promptText = t)}
          onOpenArtifact={(art) => (activeArtifact = art)}
          onRegenerate={() => {
            const lastUserMsg = [...messages].reverse().find((x) => x.role === "user");
            if (lastUserMsg) handleSend(lastUserMsg.content, [], chatMode, selectedModelId);
          }}
        />

        <!-- Input Bar Composer (Floating FAB Style) -->
        <footer class="px-2.5 pb-2.5 pt-0 sm:px-5 sm:pb-3 md:px-6 md:pb-4 bg-transparent shrink-0">
          <MessageInput
            bind:promptText {isGenerating} {selectedModelId} {chatMode}
            onSend={handleSend} onStop={handleStop}
            onModelChange={(m) => {
              selectedModelId = m;
              if (typeof localStorage !== "undefined") localStorage.setItem("lumis_selected_model", m);
              if (activeSessionId) sessions = sessions.map((s) => (s.id === activeSessionId ? { ...s, model_id: m } : s));
            }}
            onModeChange={(md) => (chatMode = md)}
          />
        </footer>
      </main>

      <!-- Artifact Split-Pane Canvas -->
      {#if activeArtifact}
        <div class="fixed md:static inset-0 md:inset-auto z-40 md:z-20 w-full md:w-[480px] lg:w-[560px] xl:w-[640px] h-full shrink-0">
          <Canvas artifact={activeArtifact} sessionId={activeSessionId || undefined} onClose={() => (activeArtifact = null)} />
        </div>
      {/if}

      <!-- Settings Modal -->
      <SettingsModal
        isOpen={isSettingsOpen} {user} {isDark} palette={themePalette}
        onClose={() => (isSettingsOpen = false)}
        onLogout={async () => { await api.logout().catch(() => {}); user = null; isSettingsOpen = false; router.navigate("/login"); }}
        onToggleDark={applyThemeMode} onSelectPalette={applyThemePalette}
      />
    </div>
  {/if}
</ErrorBoundary>
