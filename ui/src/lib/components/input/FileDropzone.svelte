<script lang="ts">
  import FileText from "lucide-svelte/icons/file-text";
  import Image from "lucide-svelte/icons/image";
  import X from "lucide-svelte/icons/x";
  import UploadCloud from "lucide-svelte/icons/upload-cloud";

  export interface AttachedFile {
    id: string;
    file: File;
    name: string;
    size: number;
    type: string;
    previewUrl?: string;
  }

  interface Props {
    files: AttachedFile[];
    onAddFiles: (files: File[]) => void;
    onRemoveFile: (id: string) => void;
    onDrop?: (files: File[]) => void;
    onPaste?: (files: File[]) => void;
    isOver?: boolean;
  }

  let { files = [], onAddFiles, onRemoveFile, onDrop, onPaste, isOver = false }: Props = $props();

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
</script>

<!-- Drag and Drop Overlay -->
{#if isOver}
  <div class="dropzone absolute inset-0 z-30 flex flex-col items-center justify-center rounded-xl border-2 border-dashed border-cyan-400 bg-background/90 backdrop-blur-xs font-sans text-cyan-400 pointer-events-none">
    <UploadCloud class="h-10 w-10 animate-bounce mb-2" />
    <span class="text-sm font-semibold">Lepaskan berkas di sini untuk diunggah</span>
    <span class="text-xs text-muted-foreground mt-1">Mendukung PDF, CSV, gambar, dan dokumen</span>
  </div>
{/if}

<!-- Attached Files Chips Bar -->
{#if files.length > 0}
  <div class="flex flex-wrap gap-2 p-2 border-b border-border/60 bg-muted/20 font-sans">
    {#each files as f (f.id)}
      <div class="flex items-center gap-2 px-2.5 py-1 rounded-lg border border-border bg-card text-xs shadow-xs">
        {#if f.type.startsWith("image/")}
          {#if f.previewUrl}
            <img src={f.previewUrl} alt={f.name} class="h-4 w-4 rounded object-cover" />
          {:else}
            <Image class="h-3.5 w-3.5 text-cyan-400" />
          {/if}
        {:else}
          <FileText class="h-3.5 w-3.5 text-cyan-400" />
        {/if}
        <span class="font-medium max-w-[120px] truncate text-foreground">{f.name}</span>
        <span class="text-[10px] text-muted-foreground font-mono">({formatFileSize(f.size)})</span>
        <button
          type="button"
          onclick={() => onRemoveFile(f.id)}
          class="p-0.5 rounded hover:bg-muted text-muted-foreground hover:text-destructive cursor-pointer"
          title="Hapus lampiran"
          aria-label="Hapus lampiran"
        >
          <X class="h-3 w-3" />
        </button>
      </div>
    {/each}
  </div>
{/if}
