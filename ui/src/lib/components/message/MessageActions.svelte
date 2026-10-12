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
    class="h-7 w-7 hover:text-foreground hover:bg-muted/50"
    onclick={handleCopy}
    title="Salin teks pesan"
    aria-label="Salin teks pesan"
  >
    {#if isCopied}
      <Check class="h-3.5 w-3.5 text-emerald-400" />
    {:else}
      <Copy class="h-3.5 w-3.5" />
    {/if}
  </Button>

  {#if role === "user" && onEdit}
    <Button
      variant="ghost"
      size="icon"
      class="h-7 w-7 hover:text-foreground hover:bg-muted/50"
      onclick={onEdit}
      title="Edit prompt dan kirim ulang"
      aria-label="Edit prompt"
    >
      <Pencil class="h-3.5 w-3.5" />
    </Button>
  {/if}

  {#if role === "assistant"}
    {#if onRegenerate}
      <Button
        variant="ghost"
        size="icon"
        class="h-7 w-7 hover:text-cyan-400 hover:bg-muted/50"
        onclick={onRegenerate}
        title="Regenerasi respon (coba lagi)"
        aria-label="Regenerasi respon"
      >
        <RotateCw class="h-3.5 w-3.5" />
      </Button>
    {/if}
    {#if onFork}
      <Button
        variant="ghost"
        size="icon"
        class="h-7 w-7 hover:text-cyan-400 hover:bg-muted/50"
        onclick={onFork}
        title="Cabangkan percakapan (Fork)"
        aria-label="Cabangkan percakapan"
      >
        <GitFork class="h-3.5 w-3.5" />
      </Button>
    {/if}
  {/if}
</div>
