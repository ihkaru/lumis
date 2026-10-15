<script lang="ts">
  import FileIcon from "lucide-svelte/icons/file";
  import FileText from "lucide-svelte/icons/file-text";
  import FileCode from "lucide-svelte/icons/file-code";
  import ImageIcon from "lucide-svelte/icons/image";
  import X from "lucide-svelte/icons/x";

  interface Props {
    files: File[];
    onRemove?: (index: number) => void;
    class?: string;
  }

  let { files = [], onRemove, class: className = "" }: Props = $props();

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function getFileIcon(type: string, name: string) {
    if (type.startsWith("image/")) return ImageIcon;
    if (name.endsWith(".ts") || name.endsWith(".js") || name.endsWith(".rs") || name.endsWith(".json")) return FileCode;
    if (type.includes("pdf") || type.includes("text") || name.endsWith(".md")) return FileText;
    return FileIcon;
  }
</script>

{#if files.length > 0}
  <div class="flex items-center gap-2 overflow-x-auto py-1 px-1 mb-2 font-sans text-xs scrollbar-thin {className}">
    {#each files as file, idx}
      {@const IconComponent = getFileIcon(file.type, file.name)}
      <div class="inline-flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-card border border-border text-foreground shadow-2xs shrink-0 max-w-[200px]">
        <IconComponent class="w-4 h-4 text-primary shrink-0" />
        <div class="min-w-0 flex-1">
          <p class="text-[11px] font-medium truncate">{file.name}</p>
          <p class="text-[9px] text-muted-foreground font-mono">{formatBytes(file.size)}</p>
        </div>
        <button
          type="button"
          class="p-0.5 rounded-sm hover:bg-muted text-muted-foreground hover:text-foreground transition-colors cursor-pointer shrink-0"
          onclick={() => onRemove?.(idx)}
          aria-label="Hapus berkas {file.name}"
        >
          <X class="w-3.5 h-3.5" />
        </button>
      </div>
    {/each}
  </div>
{/if}
