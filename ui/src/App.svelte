<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { api } from "./lib/api";
  import { router } from "./lib/router.svelte";
  import { defaultTransport } from "./lib/sdk/transport";
  import { createLumisChat } from "./lib/sdk/chatEngine.svelte";
  import { setLumisChatContext } from "./lib/sdk/context";
  import type { Session, User } from "./lib/types";
  import ErrorBoundary from "./lib/components/common/ErrorBoundary.svelte";
  import LoginPage from "./lib/components/auth/LoginPage.svelte";
  import MarkdownScratchpad from "./lib/components/chat/MarkdownScratchpad.svelte";
  import Sidebar from "./lib/components/sidebar/Sidebar.svelte";
  import ChatContainer from "./lib/components/embed/ChatContainer.svelte";
  import EmbedChatPage from "./lib/components/embed/EmbedChatPage.svelte";
  import SettingsModal from "./lib/components/settings/SettingsModal.svelte";
  import type { ThemePalette } from "./lib/components/settings/ThemesTab.svelte";

  const engine = createLumisChat(defaultTransport);
  setLumisChatContext(engine);

  let brandTitle = $state("Lumis");
  let user = $state<User | null>(null);
  let isMobileDrawerOpen = $state(typeof window !== "undefined" ? window.innerWidth >= 768 : true);
  let isSettingsOpen = $state(false);
  let isAuthChecking = $state(true);

  let isDark = $state(
    typeof localStorage !== "undefined" ? localStorage.getItem("lumis_theme_mode") !== "light" : true
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
    if (typeof localStorage !== "undefined") localStorage.setItem("lumis_theme_mode", dark ? "dark" : "light");
  }

  function applyThemePalette(p: ThemePalette) {
    themePalette = p;
    if (typeof localStorage !== "undefined") localStorage.setItem("lumis_theme_palette", p);
  }

  function updateAppHeight() {
    if (typeof window !== "undefined" && window.visualViewport) {
      document.documentElement.style.setProperty("--app-height", `${window.visualViewport.height}px`);
    }
  }

  onMount(async () => {
    router.beforeEach(async (to) => {
      if (isAuthChecking) return true;
      const clean = to.split("?")[0];
      const isPublic = clean === "/login" || clean === "/markdown" || clean.startsWith("/embed");
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
      if (router.path !== "/login" && router.path !== "/markdown" && !router.path.startsWith("/embed")) {
        router.navigate(`/login?redirect=${encodeURIComponent(router.path)}`, true);
      }
    } else {
      await engine.loadSessions();
      if (router.params.id) {
        handleSelectSession(router.params.id);
      } else if (router.path === "/" && engine.sessions.length > 0) {
        handleSelectSession(engine.sessions[0].id);
      }
    }

    engine.initStream();

    if (typeof window !== "undefined" && window.visualViewport) {
      window.visualViewport.addEventListener("resize", updateAppHeight);
      window.visualViewport.addEventListener("scroll", updateAppHeight);
      updateAppHeight();
    }
  });

  onDestroy(() => {
    engine.disconnectStream();
    if (typeof window !== "undefined" && window.visualViewport) {
      window.visualViewport.removeEventListener("resize", updateAppHeight);
      window.visualViewport.removeEventListener("scroll", updateAppHeight);
    }
  });

  async function startNewChat() {
    if (typeof window !== "undefined" && window.innerWidth < 768) isMobileDrawerOpen = false;
    if (engine.isGenerating) engine.stop();
    await router.navigate("/chat/new");
    engine.activeSessionId = null;
    engine.messages = [];
  }

  async function handleSelectSession(id: string) {
    if (id === "new") {
      await startNewChat();
      return;
    }
    if (typeof window !== "undefined" && window.innerWidth < 768) isMobileDrawerOpen = false;
    await router.navigate(`/chat/${id}`);
    await engine.selectSession(id);
  }

  $effect(() => {
    const currentPath = router.path;
    const targetId = router.params.id;
    if (currentPath === "/chat/new" || (!targetId && currentPath === "/")) {
      if (engine.activeSessionId !== null && !engine.isGenerating) {
        engine.activeSessionId = null;
        engine.messages = [];
      }
    } else if (targetId && targetId !== "new" && targetId !== engine.activeSessionId && !engine.isGenerating) {
      handleSelectSession(targetId);
    }
  });
</script>

<ErrorBoundary>
  {#if router.path.startsWith("/embed")}
    <EmbedChatPage />
  {:else if router.path === "/login"}
    <LoginPage
      {brandTitle}
      onLoginSuccess={async (uname) => {
        const auth = await api.getMe().catch(() => null);
        user = auth?.authenticated && auth.user ? auth.user : { id: uname, username: uname, role: "member" };
        await engine.loadSessions();
      }}
    />
  {:else if router.path === "/markdown"}
    <MarkdownScratchpad />
  {:else}
    <div
      class="flex h-[100dvh] max-h-[100dvh] w-screen bg-background text-foreground antialiased font-sans overflow-hidden md:flex-row flex-col"
      style="height: var(--app-height, 100dvh); max-height: var(--app-height, 100dvh);"
    >
      <Sidebar
        {brandTitle}
        sessions={engine.sessions}
        activeSessionId={engine.activeSessionId}
        {user}
        isOpenMobile={isMobileDrawerOpen}
        onSelectSession={handleSelectSession}
        onCreateSession={startNewChat}
        onCloseMobile={() => (isMobileDrawerOpen = false)}
        onRenameSession={(id, title) => engine.renameSession(id, title)}
        onDeleteSession={(id) => {
          engine.deleteSession(id);
          if (engine.activeSessionId === id) startNewChat();
        }}
        onTogglePinSession={(sess) => engine.togglePinSession(sess)}
        onOpenSettings={() => {
          if (typeof window !== "undefined" && window.innerWidth < 768) isMobileDrawerOpen = false;
          isSettingsOpen = true;
        }}
      />

      <ChatContainer
        {engine}
        {brandTitle}
        {isMobileDrawerOpen}
        onToggleMobileDrawer={() => (isMobileDrawerOpen = !isMobileDrawerOpen)}
        onOpenScratchpad={() => router.navigate("/markdown")}
        class="flex-1 min-w-0"
      />

      <SettingsModal
        isOpen={isSettingsOpen}
        {user}
        {isDark}
        palette={themePalette}
        onClose={() => (isSettingsOpen = false)}
        onLogout={async () => {
          await api.logout().catch(() => {});
          user = null;
          isSettingsOpen = false;
          router.navigate("/login");
        }}
        onToggleDark={applyThemeMode}
        onSelectPalette={applyThemePalette}
      />
    </div>
  {/if}
</ErrorBoundary>
