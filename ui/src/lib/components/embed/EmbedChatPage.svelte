<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { HttpSseTransport } from "../../sdk/transport";
  import { createLumisChat } from "../../sdk/chatEngine.svelte";
  import ChatContainer from "./ChatContainer.svelte";
  import type { LumisEventEnvelope } from "../../sdk/types";

  let token = $state("");
  let brandTitle = $state("Lumis AI");
  let allowedOrigin = $state("*");
  let isReady = $state(false);

  let engine = $state<ReturnType<typeof createLumisChat> | null>(null);

  function handlePostMessage(event: MessageEvent) {
    if (allowedOrigin !== "*" && event.origin !== allowedOrigin) return;

    const data = event.data as LumisEventEnvelope<any>;
    if (!data || data.source !== "host-app") return;

    switch (data.event) {
      case "lumis:init":
        if (data.payload?.token) {
          initEngine(data.payload.token, data.payload.context);
        }
        break;
      case "lumis:send_prompt":
        if (data.payload?.prompt && engine) {
          engine.sendMessage(data.payload.prompt, [], engine.chatMode, engine.selectedModelId);
        }
        break;
      case "lumis:set_context":
        if (data.payload && engine) {
          engine.setContextData(data.payload);
        }
        break;
      case "lumis:set_theme":
        if (data.payload?.theme) {
          document.documentElement.classList.toggle("dark", data.payload.theme === "dark");
          document.documentElement.classList.toggle("light", data.payload.theme === "light");
        }
        break;
    }
  }

  function initEngine(authToken: string, context?: Record<string, unknown>) {
    token = authToken;
    const transport = new HttpSseTransport({
      embedToken: authToken,
    });
    engine = createLumisChat(transport, {
      contextData: context,
    });
    engine.initStream();
    isReady = true;
  }

  onMount(() => {
    if (typeof window !== "undefined") {
      const sp = new URLSearchParams(window.location.search);
      const urlToken = sp.get("token");
      const urlTitle = sp.get("title");
      const urlOrigin = sp.get("origin");
      const urlTheme = sp.get("theme");

      if (urlTitle) brandTitle = urlTitle;
      if (urlOrigin) allowedOrigin = urlOrigin;
      if (urlTheme) {
        document.documentElement.classList.toggle("dark", urlTheme === "dark");
        document.documentElement.classList.toggle("light", urlTheme === "light");
      }

      window.addEventListener("message", handlePostMessage);

      // If token provided in URL query, init immediately
      if (urlToken) {
        initEngine(urlToken);
      } else {
        // Fallback: Init with default session/cookie transport
        const transport = new HttpSseTransport();
        engine = createLumisChat(transport);
        engine.initStream();
        isReady = true;
      }

      // Signal host that iframe is ready
      if (window.parent && window.parent !== window) {
        window.parent.postMessage(
          {
            source: "lumis-widget",
            event: "lumis:ready",
            timestamp: Date.now(),
          },
          allowedOrigin
        );
      }
    }
  });

  onDestroy(() => {
    if (typeof window !== "undefined") {
      window.removeEventListener("message", handlePostMessage);
    }
    engine?.disconnectStream();
  });
</script>

<div class="h-screen w-screen overflow-hidden bg-background text-foreground antialiased font-sans">
  {#if engine && isReady}
    <ChatContainer
      {engine}
      {brandTitle}
      showHeader={false}
      showModelSelector={false}
      class="h-full w-full"
    />
  {:else}
    <div class="flex h-full w-full items-center justify-center text-muted-foreground text-sm">
      Memuat Lumis Chat...
    </div>
  {/if}
</div>
