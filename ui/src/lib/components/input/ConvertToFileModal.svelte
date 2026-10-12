<script lang="ts">
  import X from "lucide-svelte/icons/x";
  import FileCode from "lucide-svelte/icons/file-code";
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

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && open) {
      onClose?.();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs font-sans"
    role="dialog"
    aria-modal="true"
    aria-label="Konversi ke Berkas"
  >
    <div class="w-full max-w-md p-5 rounded-2xl bg-card border border-border shadow-2xl text-foreground animate-in fade-in zoom-in-95 duration-150">
      <div class="flex items-center justify-between pb-3 border-b border-border/60">
        <div class="flex items-center gap-2">
          <div class="p-1 rounded-md bg-cyan-950/80 border border-cyan-800 text-cyan-400">
            <FileCode class="w-4 h-4" />
          </div>
          <h2 class="font-semibold text-sm">Konversi Teks ke Berkas</h2>
        </div>
        <button
          type="button"
          class="p-1 rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted transition-colors cursor-pointer"
          onclick={() => onClose?.()}
        >
          <X class="w-4 h-4" />
        </button>
      </div>

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
              class="flex-1 px-3 py-1.5 rounded-lg bg-background border border-border text-foreground font-mono focus:outline-hidden focus:ring-1 focus:ring-cyan-500"
              placeholder="nama-berkas"
            />
            <select
              bind:value={extension}
              class="px-2.5 py-1.5 rounded-lg bg-background border border-border text-foreground font-mono cursor-pointer focus:outline-hidden"
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

      <div class="flex items-center justify-end gap-2 pt-3 border-t border-border/60">
        <Button variant="ghost" size="sm" onclick={() => onClose?.()} class="cursor-pointer">
          Batal
        </Button>
        <Button
          size="sm"
          class="bg-cyan-600 hover:bg-cyan-500 text-white cursor-pointer"
          onclick={handleConvert}
          disabled={!initialText.trim()}
        >
          Konversi & Lampirkan
        </Button>
      </div>
    </div>
  </div>
{/if}
