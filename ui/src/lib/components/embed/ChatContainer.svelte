<script lang="ts">
  import type { ArtifactSpecification } from "../../types";
  import type { LumisChatEngine } from "../../sdk/chatEngine.svelte";
  import { getLumisChatContext } from "../../sdk/context";
  import ChatHeader from "../chat/ChatHeader.svelte";
  import MessageViewport from "../chat/MessageViewport.svelte";
  import MessageInput from "../input/MessageInput.svelte";
  import { Canvas } from "../ui/canvas";
  import type { AttachedFile } from "../input/FileDropzone.svelte";
  import type { ChatMode } from "../input/ModeSelector.svelte";

  interface Props {
    engine?: LumisChatEngine;
    brandTitle?: string;
    showHeader?: boolean;
    showModelSelector?: boolean;
    showModeSelector?: boolean;
    isMobileDrawerOpen?: boolean;
    onToggleMobileDrawer?: () => void;
    onOpenScratchpad?: () => void;
    class?: string;
  }

  let {
    engine: propEngine,
    brandTitle = "Lumis",
    showHeader = true,
    showModelSelector = true,
    showModeSelector = true,
    isMobileDrawerOpen = false,
    onToggleMobileDrawer,
    onOpenScratchpad,
    class: customClass = "",
  }: Props = $props();

  const ctxEngine = getLumisChatContext();
  const engine = $derived(propEngine || ctxEngine);

  let promptText = $state("");
  let activeArtifact = $state<ArtifactSpecification | null>(null);
  let messageViewport = $state<ReturnType<typeof MessageViewport> | null>(null);

  function handleSend(text: string, files: AttachedFile[], mode: ChatMode, modelId: string) {
    if (!engine) return;
    engine.sendMessage(text, files, mode, modelId);
  }

  function handleStop() {
    if (!engine) return;
    engine.stop();
    messageViewport?.resetScrollState();
  }
</script>

<div class="relative flex h-full w-full overflow-hidden bg-background text-foreground {customClass}">
  <main class="flex-1 flex flex-col justify-between bg-background overflow-hidden min-w-0">
    {#if showHeader}
      <ChatHeader
        title={engine?.activeSession ? engine.activeSession.title : "Percakapan Baru"}
        selectedModelId={engine?.selectedModelId || "my9model-free"}
        {isMobileDrawerOpen}
        onToggleMobileDrawer={() => onToggleMobileDrawer?.()}
        onOpenScratchpad={() => onOpenScratchpad?.()}
      />
    {/if}

    {#if engine?.errorMessage}
      <div class="bg-destructive/20 border-b border-destructive/40 text-red-400 text-xs px-6 py-2.5 break-words [overflow-wrap:anywhere]">
        {engine.errorMessage}
      </div>
    {/if}

    <MessageViewport
      bind:this={messageViewport}
      messages={engine?.messages || []}
      sessionTasks={engine?.sessionTasks || []}
      activeSessionId={engine?.activeSessionId || null}
      isGenerating={engine?.isGenerating || false}
      {brandTitle}
      isNewChatRoute={!engine?.activeSessionId}
      onSelectPrompt={(t) => (promptText = t)}
      onEditPrompt={(t) => (promptText = t)}
      onOpenArtifact={(art) => (activeArtifact = art)}
      onRegenerate={(retryModel) => {
        engine?.retryLastMessage(retryModel);
      }}
    />

    <footer class="px-5 pb-3 pt-0 sm:px-8 sm:pb-4 md:px-10 md:pb-5 lg:px-12 bg-transparent shrink-0">
      <MessageInput
        bind:promptText
        isGenerating={engine?.isGenerating || false}
        selectedModelId={engine?.selectedModelId || "my9model-free"}
        chatMode={engine?.chatMode || "normal"}
        onSend={handleSend}
        onStop={handleStop}
        onModelChange={(m) => {
          if (engine) engine.selectedModelId = m;
        }}
        onModeChange={(md) => {
          if (engine) engine.chatMode = md;
        }}
      />
    </footer>
  </main>

  {#if activeArtifact}
    <div class="fixed md:static inset-0 md:inset-auto z-40 md:z-20 w-full md:w-[480px] lg:w-[560px] xl:w-[640px] h-full shrink-0">
      <Canvas
        artifact={activeArtifact}
        sessionId={engine?.activeSessionId || undefined}
        onClose={() => (activeArtifact = null)}
      />
    </div>
  {/if}
</div>
