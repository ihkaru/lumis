<script lang="ts">
  import UserIcon from "lucide-svelte/icons/user";
  import MessageActions from "./MessageActions.svelte";
  import MessagePagination from "./MessagePagination.svelte";
  import type { Message } from "$lib/types";

  interface Props {
    message: Message;
    branchIndex?: number;
    branchTotal?: number;
    onPrevBranch?: () => void;
    onNextBranch?: () => void;
    onEditPrompt?: (content: string) => void;
  }

  let {
    message,
    branchIndex = 1,
    branchTotal = 1,
    onPrevBranch,
    onNextBranch,
    onEditPrompt
  }: Props = $props();

  function formatTime(iso: string): string {
    try {
      const d = new Date(iso);
      return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    } catch {
      return "";
    }
  }
</script>

<div class="flex justify-end font-sans group" data-role="user" data-message-id={message.id}>
  <div class="flex flex-col items-end max-w-2xl">
    <div class="flex items-center gap-2 mb-1 text-[11px] text-muted-foreground select-none">
      {#if branchTotal > 1}
        <MessagePagination
          current={branchIndex}
          total={branchTotal}
          onPrev={onPrevBranch}
          onNext={onNextBranch}
          class="mr-1"
        />
      {/if}
      <span>{formatTime(message.created_at)}</span>
      <span class="font-medium text-foreground">Anda</span>
      <div class="p-1 rounded-full bg-cyan-950 border border-cyan-800 text-cyan-400">
        <UserIcon class="h-3 w-3" />
      </div>
    </div>

    <div class="px-4 py-3 rounded-2xl rounded-tr-none bg-cyan-600 text-white text-sm leading-relaxed shadow-sm break-words whitespace-pre-wrap [overflow-wrap:anywhere]">
      {message.content}
    </div>

    <MessageActions
      content={message.content}
      role="user"
      onEdit={() => onEditPrompt?.(message.content)}
    />
  </div>
</div>
