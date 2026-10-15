<script lang="ts">
  import Copy from "lucide-svelte/icons/copy";
  import Check from "lucide-svelte/icons/check";
  import RotateCw from "lucide-svelte/icons/rotate-cw";
  import Pencil from "lucide-svelte/icons/pencil";
  import GitFork from "lucide-svelte/icons/git-fork";
  import { Button } from "$lib/components/ui/button";

  interface Props {
    content: string;
    role: "user" | "assistant" | "system" | "tool";
    onRegenerate?: () => void;
    onEdit?: () => void;
    onFork?: () => void;
  }

  let { content, role, onRegenerate, onEdit, onFork }: Props = $props();

  let isCopied = $state(false);

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(content);
      isCopied = true;
      setTimeout(() => {
        isCopied = false;
      }, 2000);
    } catch (e) {
      console.error("Clipboard copy failed:", e);
    }
  }
</script>

<div class="flex items-center gap-1 text-muted-foreground pt-1.5 opacity-80 hover:opacity-100 transition-opacity select-none">
  <Button
    variant="ghost"
    size="icon"
    class="min-h-[44px] min-w-[44px] sm:min-h-7 sm:min-w-7 h-8 w-8 sm:h-7 sm:w-7 hover:text-foreground hover:bg-muted/50 touch-manipulation cursor-pointer"
    onclick={handleCopy}
    title="Salin teks pesan"
    aria-label="Salin teks pesan"
  >
    {#if isCopied}
      <Check class="h-4 w-4 sm:h-3.5 sm:w-3.5 text-emerald-400" />
    {:else}
      <Copy class="h-4 w-4 sm:h-3.5 sm:w-3.5" />
    {/if}
  </Button>

  {#if role === "user" && onEdit}
    <Button
      variant="ghost"
      size="icon"
      class="min-h-[44px] min-w-[44px] sm:min-h-7 sm:min-w-7 h-8 w-8 sm:h-7 sm:w-7 hover:text-foreground hover:bg-muted/50 touch-manipulation cursor-pointer"
      onclick={onEdit}
      title="Edit prompt dan kirim ulang"
      aria-label="Edit prompt"
    >
      <Pencil class="h-4 w-4 sm:h-3.5 sm:w-3.5" />
    </Button>
  {/if}

  {#if role === "assistant"}
    {#if onRegenerate}
      <Button
        variant="ghost"
        size="icon"
        class="min-h-[44px] min-w-[44px] sm:min-h-7 sm:min-w-7 h-8 w-8 sm:h-7 sm:w-7 hover:text-primary hover:bg-muted/50 touch-manipulation cursor-pointer"
        onclick={onRegenerate}
        title="Regenerasi respon (coba lagi)"
        aria-label="Regenerasi respon"
      >
        <RotateCw class="h-4 w-4 sm:h-3.5 sm:w-3.5" />
      </Button>
    {/if}
    {#if onFork}
      <Button
        variant="ghost"
        size="icon"
        class="min-h-[44px] min-w-[44px] sm:min-h-7 sm:min-w-7 h-8 w-8 sm:h-7 sm:w-7 hover:text-primary hover:bg-muted/50 touch-manipulation cursor-pointer"
        onclick={onFork}
        title="Cabangkan percakapan (Fork)"
        aria-label="Cabangkan percakapan"
      >
        <GitFork class="h-4 w-4 sm:h-3.5 sm:w-3.5" />
      </Button>
    {/if}
  {/if}
</div>
