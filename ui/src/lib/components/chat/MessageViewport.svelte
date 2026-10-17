<script lang="ts">
  import type { ArtifactSpecification, Message, SessionTask } from "$lib/types";
  import NewChatWelcome from "./NewChatWelcome.svelte";
  import UserBubble from "../message/UserBubble.svelte";
  import AssistantBubble from "../message/AssistantBubble.svelte";

  interface Props {
    messages: Message[];
    sessionTasks: SessionTask[];
    activeSessionId: string | null;
    isGenerating: boolean;
    brandTitle: string;
    isNewChatRoute: boolean;
    onSelectPrompt: (prompt: string) => void;
    onEditPrompt: (prompt: string) => void;
    onOpenArtifact: (artifact: ArtifactSpecification) => void;
    onRegenerate: (modelId?: string) => void;
  }

  let {
    messages,
    sessionTasks,
    activeSessionId,
    isGenerating,
    brandTitle,
    isNewChatRoute,
    onSelectPrompt,
    onEditPrompt,
    onOpenArtifact,
    onRegenerate
  }: Props = $props();

  let messageContainerEl = $state<HTMLDivElement | null>(null);
  let isUserScrolledUp = $state(false);
  let lastScrollTop = 0;

  function handleContainerScroll() {
    if (!messageContainerEl) return;
    const current = messageContainerEl.scrollTop;
    if (isGenerating) {
      if (current < lastScrollTop - 15) {
        isUserScrolledUp = true;
      } else {
        const last = messageContainerEl.querySelector<HTMLElement>('[data-role="assistant"]:last-of-type');
        if (last && last.getBoundingClientRect().bottom <= messageContainerEl.getBoundingClientRect().bottom + 60) {
          isUserScrolledUp = false;
        }
      }
    }
    lastScrollTop = current;
  }

  export function scrollUserMessageToTop(msgId: string) {
    requestAnimationFrame(() => setTimeout(() => {
      if (!messageContainerEl) return;
      const el = messageContainerEl.querySelector<HTMLElement>(`[data-message-id="${msgId}"]`);
      if (el) {
        const cRect = messageContainerEl.getBoundingClientRect();
        const tRect = el.getBoundingClientRect();
        messageContainerEl.scrollTo({
          top: Math.max(0, messageContainerEl.scrollTop + (tRect.top - cRect.top) - 16),
          behavior: "smooth"
        });
        isUserScrolledUp = false;
      } else {
        messageContainerEl.scrollTo({ top: 0, behavior: "smooth" });
        isUserScrolledUp = false;
      }
    }, 40));
  }

  export function autoScrollDuringStream() {
    if (isUserScrolledUp || !messageContainerEl) return;
    const last = messageContainerEl.querySelector<HTMLElement>('[data-role="assistant"]:last-of-type');
    if (!last) return;
    const cRect = messageContainerEl.getBoundingClientRect();
    const bRect = last.getBoundingClientRect();

    // On the first turn, keep the user message visible unless the assistant text overflows the viewport
    if (messages.length <= 2) {
      if (bRect.bottom > cRect.bottom - 16) {
        messageContainerEl.scrollTo({
          top: messageContainerEl.scrollTop + (bRect.bottom - cRect.bottom + 16),
          behavior: "smooth"
        });
      }
      return;
    }

    if (bRect.bottom > cRect.bottom - 48) {
      messageContainerEl.scrollTo({
        top: messageContainerEl.scrollTop + (bRect.bottom - cRect.bottom + 48),
        behavior: "smooth"
      });
    }
  }

  export function scrollToBottom() {
    setTimeout(() => {
      if (!messageContainerEl) return;
      const last = messageContainerEl.querySelector<HTMLElement>('[data-role="assistant"]:last-of-type, [data-role="user"]:last-of-type');
      if (last) last.scrollIntoView({ behavior: "instant", block: "end" });
      else messageContainerEl.scrollTop = messageContainerEl.scrollHeight;
    }, 50);
  }

  export function resetScrollState() {
    isUserScrolledUp = false;
  }
</script>

<div
  bind:this={messageContainerEl}
  onscroll={handleContainerScroll}
  class="flex-1 overflow-y-auto px-5 py-4 sm:px-8 sm:py-5 md:px-10 md:py-6 lg:px-12 space-y-6 smooth-momentum-scroll"
>
  {#if (!activeSessionId || isNewChatRoute) && messages.length === 0}
    <NewChatWelcome {brandTitle} {onSelectPrompt} />
  {:else}
    {#each messages as m, idx (m.id)}
      {#if m.role === "user"}
        <UserBubble message={m} {onEditPrompt} />
      {:else}
        <AssistantBubble
          message={m}
          isStreaming={isGenerating && m.id === messages[messages.length - 1]?.id}
          tasks={idx === messages.length - 1 ? sessionTasks : []}
          {onOpenArtifact}
          {onRegenerate}
        />
      {/if}
    {/each}
    {#if messages.length > 0}
      <div
        class="{isGenerating ? 'h-[25vh]' : 'h-6'} shrink-0 pointer-events-none transition-all duration-500 ease-out"
        aria-hidden="true"
      ></div>
    {/if}
  {/if}
</div>
