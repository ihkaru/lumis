<script lang="ts">
  import MessageActions from "./MessageActions.svelte";
  import MessagePagination from "./MessagePagination.svelte";
  import { formatLocalTime } from "$lib/dateTime";
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
      <span>{formatLocalTime(message.created_at)}</span>
      <span class="font-medium text-foreground">Anda</span>
    </div>

    <div class="px-4 py-2.5 rounded-2xl rounded-tr-sm bg-secondary text-secondary-foreground text-sm leading-relaxed border border-border/60 shadow-xs break-words whitespace-pre-wrap [overflow-wrap:anywhere] transition-colors hover:border-primary/30">
      {message.content}
    </div>

    <MessageActions
      content={message.content}
      role="user"
      onEdit={() => onEditPrompt?.(message.content)}
    />
  </div>
</div>
