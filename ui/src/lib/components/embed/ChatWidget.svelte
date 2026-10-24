<script lang="ts">
  import MessageSquare from "lucide-svelte/icons/message-square";
  import X from "lucide-svelte/icons/x";
  import Minus from "lucide-svelte/icons/minus";
  import type { LumisChatEngine } from "../../sdk/chatEngine.svelte";
  import ChatContainer from "./ChatContainer.svelte";

  interface Props {
    engine?: LumisChatEngine;
    brandTitle?: string;
    position?: "bottom-right" | "bottom-left";
    defaultOpen?: boolean;
    class?: string;
  }

  let {
    engine,
    brandTitle = "Lumis AI",
    position = "bottom-right",
    defaultOpen = false,
    class: customClass = "",
  }: Props = $props();

  let isOpen = $state(false);

  $effect(() => {
    isOpen = defaultOpen;
  });

  function toggleOpen() {
    isOpen = !isOpen;
  }

  const posClasses = $derived(
    position === "bottom-left" ? "left-5 sm:left-8" : "right-5 sm:right-8"
  );
</script>

<div class="fixed bottom-5 {posClasses} z-50 flex flex-col items-end {customClass}">
  {#if isOpen}
    <!-- Expanded Floating Chat Window / Mobile Drawer -->
    <div
      class="fixed inset-0 z-50 flex flex-col bg-background/95 backdrop-blur-md md:static md:w-[420px] md:h-[620px] md:rounded-2xl md:border md:border-border/60 md:shadow-2xl overflow-hidden mb-3 will-change-transform transition-all duration-200"
      style="transform: translate3d(0, 0, 0);"
    >
      <!-- Top Embedded Header with Close Button -->
      <div class="flex items-center justify-between px-4 py-3 border-b border-border/40 bg-muted/30 shrink-0">
        <div class="flex items-center gap-2">
          <div class="w-2.5 h-2.5 rounded-full bg-emerald-500 animate-pulse"></div>
          <span class="text-sm font-semibold tracking-tight text-foreground">{brandTitle}</span>
        </div>
        <div class="flex items-center gap-1">
          <button
            type="button"
            onclick={toggleOpen}
            aria-label="Minimize Chat"
            class="p-2 rounded-lg hover:bg-muted text-muted-foreground hover:text-foreground transition-colors touch-manipulation min-w-[44px] min-h-[44px] flex items-center justify-center"
          >
            <Minus class="w-4 h-4" />
          </button>
          <button
            type="button"
            onclick={toggleOpen}
            aria-label="Close Chat"
            class="p-2 rounded-lg hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors touch-manipulation min-w-[44px] min-h-[44px] flex items-center justify-center"
          >
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Core Chat Viewport -->
      <div class="flex-1 overflow-hidden min-h-0">
        <ChatContainer
          {engine}
          {brandTitle}
          showHeader={false}
          showModelSelector={false}
          class="h-full"
        />
      </div>
    </div>
  {/if}

  <!-- Floating Launcher Action Button (FAB) -->
  <button
    type="button"
    onclick={toggleOpen}
    aria-label="Toggle AI Assistant"
    class="flex items-center justify-center w-14 h-14 rounded-full bg-primary text-primary-foreground shadow-lg hover:shadow-xl hover:scale-105 active:scale-95 transition-all duration-200 touch-manipulation min-w-[56px] min-h-[56px] will-change-transform"
    style="transform: translate3d(0, 0, 0);"
  >
    {#if isOpen}
      <X class="w-6 h-6 transition-transform duration-200 rotate-90" />
    {:else}
      <MessageSquare class="w-6 h-6 transition-transform duration-200" />
    {/if}
  </button>
</div>
