<script lang="ts">
  import Trash2 from "lucide-svelte/icons/trash-2";
  import AlertTriangle from "lucide-svelte/icons/triangle-alert";
  import { Dialog, DialogOverlay, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter, DialogClose } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";

  interface Props {
    isOpen: boolean;
    sessionTitle: string;
    onClose: () => void;
    onConfirmDelete: () => void;
  }

  let { isOpen = false, sessionTitle = "", onClose, onConfirmDelete }: Props = $props();
</script>

<Dialog open={isOpen}>
  <DialogOverlay open={isOpen} onclick={onClose} />
  <DialogContent open={isOpen} {onClose} class="max-w-sm border-destructive/40">
    <DialogHeader>
      <DialogTitle class="text-destructive flex items-center gap-2">
        <AlertTriangle class="h-4 w-4" />
        <span>Hapus Percakapan</span>
      </DialogTitle>
    </DialogHeader>

    <DialogClose onclick={onClose} />

    <DialogDescription class="leading-relaxed py-2">
      Apakah Anda yakin ingin menghapus permanen obrolan <strong class="text-foreground">"{sessionTitle}"</strong>? Semua riwayat pesan di dalamnya tidak dapat dipulihkan.
    </DialogDescription>

    <DialogFooter>
      <Button variant="ghost" onclick={onClose} class="text-xs min-h-[44px] cursor-pointer">
        Batal
      </Button>
      <Button
        variant="destructive"
        onclick={() => {
          onConfirmDelete();
          onClose();
        }}
        class="gap-1.5 text-xs font-semibold min-h-[44px] cursor-pointer"
      >
        <Trash2 class="h-3.5 w-3.5" />
        <span>Ya, Hapus</span>
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
