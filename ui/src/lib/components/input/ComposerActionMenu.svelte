<script lang="ts">
  import Paperclip from "lucide-svelte/icons/paperclip";
  import FileCode from "lucide-svelte/icons/file-code";
  import Trash2 from "lucide-svelte/icons/trash-2";
  import Sparkles from "lucide-svelte/icons/sparkles";

  interface Props {
    open: boolean;
    onClose?: () => void;
    onAttachFile?: () => void;
    onConvertToFile?: () => void;
    onClearChat?: () => void;
    onInsertPreset?: (prompt: string) => void;
  }

  let {
    open = false,
    onClose,
    onAttachFile,
    onConvertToFile,
    onClearChat,
    onInsertPreset
  }: Props = $props();

  function selectAction(fn?: () => void) {
    fn?.();
    onClose?.();
  }

  const presets = [
    "Jelaskan kode ini baris per baris secara mendalam.",
    "Lakukan refactoring arsitektur sesuai prinsip SOLID & Old-Coder.",
    "Buatkan unit test suite lengkap dengan edge-cases."
  ];
</script>

{#if open}
  <!-- Backdrop dismiss -->
  <button
    type="button"
    class="fixed inset-0 z-30 w-full h-full bg-transparent border-0 cursor-default"
    onclick={() => onClose?.()}
    aria-label="Tutup menu aksi"
  ></button>

  <!-- Popover Panel -->
  <div
    class="absolute bottom-12 left-0 z-40 w-64 p-1.5 rounded-xl bg-popover/95 backdrop-blur-md border border-border shadow-xl font-sans text-xs animate-in fade-in zoom-in-95 duration-100"
    role="menu"
    aria-label="Composer Fast Action Menu"
  >
    <div class="px-2 py-1 text-[10px] font-semibold text-muted-foreground uppercase tracking-wider select-none">
      Aksi Cepat
    </div>

    <button
      type="button"
      class="w-full flex items-center gap-2.5 px-2.5 py-2 rounded-lg text-foreground hover:bg-muted/80 transition-colors cursor-pointer text-left"
      onclick={() => selectAction(onAttachFile)}
    >
      <Paperclip class="w-3.5 h-3.5 text-cyan-400 shrink-0" />
      <span>Unggah Dokumen / Gambar</span>
    </button>

    <button
      type="button"
      class="w-full flex items-center gap-2.5 px-2.5 py-2 rounded-lg text-foreground hover:bg-muted/80 transition-colors cursor-pointer text-left"
      onclick={() => selectAction(onConvertToFile)}
    >
      <FileCode class="w-3.5 h-3.5 text-cyan-400 shrink-0" />
      <span>Konversi Prompt ke Berkas</span>
    </button>

    <div class="my-1 border-t border-border/60"></div>

    <div class="px-2 py-1 text-[10px] font-semibold text-muted-foreground uppercase tracking-wider select-none flex items-center gap-1">
      <Sparkles class="w-3 h-3 text-cyan-400" />
      <span>Preset Prompt</span>
    </div>

    {#each presets as pr}
      <button
        type="button"
        class="w-full px-2.5 py-1.5 rounded-lg text-[11px] text-muted-foreground hover:text-foreground hover:bg-muted/80 transition-colors cursor-pointer text-left truncate"
        onclick={() => selectAction(() => onInsertPreset?.(pr))}
        title={pr}
      >
        {pr}
      </button>
    {/each}

    <div class="my-1 border-t border-border/60"></div>

    <button
      type="button"
      class="w-full flex items-center gap-2.5 px-2.5 py-2 rounded-lg text-destructive hover:bg-destructive/10 transition-colors cursor-pointer text-left"
      onclick={() => selectAction(onClearChat)}
    >
      <Trash2 class="w-3.5 h-3.5 shrink-0" />
      <span>Bersihkan Percakapan</span>
    </button>
  </div>
{/if}
