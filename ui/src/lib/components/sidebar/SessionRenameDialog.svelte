<script lang="ts">
  import Pencil from "lucide-svelte/icons/pencil";
  import Check from "lucide-svelte/icons/check";
  import { Dialog, DialogOverlay, DialogContent, DialogHeader, DialogTitle, DialogFooter, DialogClose } from "$lib/components/ui/dialog";
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

<Dialog open={isOpen}>
  <DialogOverlay open={isOpen} onclick={onClose} />
  <DialogContent open={isOpen} {onClose} class="max-w-sm">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        <Pencil class="h-4 w-4 text-cyan-400" />
        <span>Ubah Nama Obrolan</span>
      </DialogTitle>
    </DialogHeader>

    <DialogClose onclick={onClose} />

    <form onsubmit={handleSubmit} class="space-y-4 pt-2">
      <Input
        type="text"
        bind:value={titleValue}
        placeholder="Judul percakapan..."
        required
        class="h-10 text-sm font-sans"
      />

      <DialogFooter>
        <Button variant="ghost" onclick={onClose} class="text-xs min-h-[44px] cursor-pointer">
          Batal
        </Button>
        <Button type="submit" variant="cyan" class="gap-1.5 text-xs min-h-[44px] cursor-pointer">
          <Check class="h-3.5 w-3.5" />
          <span>Simpan</span>
        </Button>
      </DialogFooter>
    </form>
  </DialogContent>
</Dialog>
