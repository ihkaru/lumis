<script lang="ts">
  import Pencil from "lucide-svelte/icons/pencil";
  import X from "lucide-svelte/icons/x";
  import Check from "lucide-svelte/icons/check";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";

  interface Props {
    isOpen: boolean;
    initialTitle: string;
    onClose: () => void;
    onRename: (newTitle: string) => void;
  }

  let { isOpen = false, initialTitle = "", onClose, onRename }: Props = $props();

  // svelte-ignore state_referenced_locally
  let titleValue = $state(initialTitle);

  $effect(() => {
    if (isOpen) {
      titleValue = initialTitle;
    }
  });

  function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!titleValue.trim()) return;
    onRename(titleValue.trim());
    onClose();
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm p-4 font-sans select-none">
    <div class="w-full max-w-sm rounded-2xl border border-border bg-card p-6 shadow-2xl space-y-4">
      <div class="flex items-center justify-between">
        <h3 class="text-sm font-semibold text-foreground flex items-center gap-2">
          <Pencil class="h-4 w-4 text-cyan-400" />
          <span>Ubah Nama Obrolan</span>
        </h3>
        <button
          type="button"
          onclick={onClose}
          class="text-muted-foreground hover:text-foreground cursor-pointer"
          aria-label="Tutup"
        >
          <X class="h-4 w-4" />
        </button>
      </div>

      <form onsubmit={handleSubmit} class="space-y-4">
        <Input
          type="text"
          bind:value={titleValue}
          placeholder="Judul percakapan..."
          required
          class="h-10 text-sm"
        />

        <div class="flex justify-end gap-2 pt-1">
          <Button variant="ghost" onclick={onClose} class="text-xs">
            Batal
          </Button>
          <Button type="submit" variant="cyan" class="gap-1.5 text-xs">
            <Check class="h-3.5 w-3.5" />
            <span>Simpan</span>
          </Button>
        </div>
      </form>
    </div>
  </div>
{/if}
