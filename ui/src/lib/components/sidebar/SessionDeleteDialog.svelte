<script lang="ts">
  import Trash2 from "lucide-svelte/icons/trash-2";
  import X from "lucide-svelte/icons/x";
  import AlertTriangle from "lucide-svelte/icons/triangle-alert";
  import { Button } from "$lib/components/ui/button";

  interface Props {
    isOpen: boolean;
    sessionTitle: string;
    onClose: () => void;
    onConfirmDelete: () => void;
  }

  let { isOpen = false, sessionTitle = "", onClose, onConfirmDelete }: Props = $props();
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm p-4 font-sans select-none">
    <div class="w-full max-w-sm rounded-2xl border border-destructive/40 bg-card p-6 shadow-2xl space-y-4">
      <div class="flex items-center justify-between">
        <h3 class="text-sm font-semibold text-destructive flex items-center gap-2">
          <AlertTriangle class="h-4 w-4" />
          <span>Hapus Percakapan</span>
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

      <p class="text-xs text-muted-foreground leading-relaxed break-words [overflow-wrap:anywhere]">
        Apakah Anda yakin ingin menghapus permanen obrolan <strong class="text-foreground">"{sessionTitle}"</strong>? Semua riwayat pesan di dalamnya tidak dapat dipulihkan.
      </p>

      <div class="flex justify-end gap-2 pt-2">
        <Button variant="ghost" onclick={onClose} class="text-xs">
          Batal
        </Button>
        <Button
          variant="destructive"
          onclick={() => {
            onConfirmDelete();
            onClose();
          }}
          class="gap-1.5 text-xs font-semibold"
        >
          <Trash2 class="h-3.5 w-3.5" />
          <span>Ya, Hapus</span>
        </Button>
      </div>
    </div>
  </div>
{/if}
