<script lang="ts">
  import FileCode from "lucide-svelte/icons/file-code";
  import { Dialog, DialogOverlay, DialogContent, DialogHeader, DialogTitle, DialogFooter, DialogClose } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";

  interface Props {
    open: boolean;
    initialText?: string;
    onClose?: () => void;
    onConverted?: (file: File) => void;
  }

  let {
    open = false,
    initialText = "",
    onClose,
    onConverted
  }: Props = $props();

  let filename = $state("prompt-attachment");
  let extension = $state("md");

  let fullFilename = $derived(`${filename.trim() || "attachment"}.${extension}`);
  let byteSize = $derived(new Blob([initialText]).size);

  function handleConvert() {
    const mime = extension === "json" ? "application/json" : "text/plain";
    const file = new File([initialText], fullFilename, { type: mime });
    onConverted?.(file);
    onClose?.();
  }
</script>

<Dialog {open}>
  <DialogOverlay {open} onclick={onClose} />
  <DialogContent {open} {onClose} class="max-w-md">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        <div class="p-1 rounded-md bg-primary/10 border border-primary/25 text-primary">
          <FileCode class="w-4 h-4" />
        </div>
        <span>Konversi Teks ke Berkas</span>
      </DialogTitle>
    </DialogHeader>

    <DialogClose onclick={onClose} />

    <div class="py-4 space-y-3 text-xs">
      <div>
        <label for="conv-filename" class="block font-medium text-muted-foreground mb-1">
          Nama Berkas
        </label>
        <div class="flex items-center gap-2">
          <input
            id="conv-filename"
            type="text"
            bind:value={filename}
            class="flex-1 px-3 py-2 rounded-lg bg-background border border-border text-foreground font-mono focus:outline-hidden focus:ring-1 focus:ring-primary min-h-[44px]"
            placeholder="nama-berkas"
          />
          <select
            bind:value={extension}
            class="px-3 py-2 rounded-lg bg-background border border-border text-foreground font-mono cursor-pointer focus:outline-hidden min-h-[44px]"
          >
            <option value="md">.md</option>
            <option value="txt">.txt</option>
            <option value="json">.json</option>
            <option value="ts">.ts</option>
          </select>
        </div>
      </div>

      <div class="p-2.5 rounded-xl bg-muted/40 border border-border/40 text-[11px] text-muted-foreground space-y-1">
        <div class="flex justify-between">
          <span>Target Berkas:</span>
          <span class="font-mono text-foreground">{fullFilename}</span>
        </div>
        <div class="flex justify-between">
          <span>Ukuran Konten:</span>
          <span class="font-mono text-foreground">{(byteSize / 1024).toFixed(1)} KB ({initialText.length} karakter)</span>
        </div>
      </div>
    </div>

    <DialogFooter>
      <Button variant="ghost" size="sm" onclick={onClose} class="min-h-[44px] cursor-pointer">
        Batal
      </Button>
      <Button
        size="sm"
        variant="default"
        class="min-h-[44px] cursor-pointer"
        onclick={handleConvert}
        disabled={!initialText.trim()}
      >
        Konversi & Lampirkan
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
