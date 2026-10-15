<script lang="ts">
  import Send from "lucide-svelte/icons/send";
  import Square from "lucide-svelte/icons/square";
  import Paperclip from "lucide-svelte/icons/paperclip";
  import Mic from "lucide-svelte/icons/mic";
  import Plus from "lucide-svelte/icons/plus";
  import ModelSelector from "./ModelSelector.svelte";
  import ModeSelector, { type ChatMode } from "./ModeSelector.svelte";
  import FileDropzone, { type AttachedFile } from "./FileDropzone.svelte";
  import AudioRecordModal from "./AudioRecordModal.svelte";
  import ComposerActionMenu from "./ComposerActionMenu.svelte";
  import AttachmentFileGroup from "./AttachmentFileGroup.svelte";
  import ConvertToFileModal from "./ConvertToFileModal.svelte";
  import { Button } from "$lib/components/ui/button";

  interface Props {
    promptText: string;
    isGenerating: boolean;
    selectedModelId: string;
    chatMode: ChatMode;
    onSend: (text: string, files: AttachedFile[], mode: ChatMode, modelId: string) => void;
    onStop: () => void;
    onModelChange: (modelId: string) => void;
    onModeChange: (mode: ChatMode) => void;
    onClearChat?: () => void;
  }

  let {
    promptText = $bindable(""),
    isGenerating = false,
    selectedModelId = "my9model-free",
    chatMode = "normal",
    onSend,
    onStop,
    onModelChange,
    onModeChange,
    onClearChat,
  }: Props = $props();

  let attachedFiles = $state<AttachedFile[]>([]);
  let isDragOver = $state(false);
  let isAudioModalOpen = $state(false);
  let isActionMenuOpen = $state(false);
  let isConvertModalOpen = $state(false);
  let fileInputRef: HTMLInputElement | null = null;
  let textareaRef: HTMLTextAreaElement | null = null;

  let rawFileList = $derived(attachedFiles.map((a) => a.file));

  function handleAddFiles(rawFiles: File[]) {
    for (const f of rawFiles) {
      const fileId = `file_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`;
      const previewUrl = f.type.startsWith("image/") ? URL.createObjectURL(f) : undefined;
      attachedFiles = [
        ...attachedFiles,
        {
          id: fileId,
          file: f,
          name: f.name,
          size: f.size,
          type: f.type,
          previewUrl,
        },
      ];
    }
  }

  function handleRemoveFile(id: string) {
    const target = attachedFiles.find((f) => f.id === id);
    if (target?.previewUrl) URL.revokeObjectURL(target.previewUrl);
    attachedFiles = attachedFiles.filter((f) => f.id !== id);
  }

  function handleRemoveFileByIndex(idx: number) {
    if (idx >= 0 && idx < attachedFiles.length) {
      handleRemoveFile(attachedFiles[idx].id);
    }
  }

  function handleFileSelect(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files && input.files.length > 0) {
      handleAddFiles(Array.from(input.files));
      input.value = "";
    }
  }

  function handlePaste(e: ClipboardEvent) {
    if (e.clipboardData && e.clipboardData.files.length > 0) {
      e.preventDefault();
      handleAddFiles(Array.from(e.clipboardData.files));
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDragOver = true;
  }

  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    isDragOver = false;
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragOver = false;
    if (e.dataTransfer && e.dataTransfer.files.length > 0) {
      handleAddFiles(Array.from(e.dataTransfer.files));
    }
  }

  function handleSubmit() {
    if ((!promptText.trim() && attachedFiles.length === 0) || isGenerating) return;
    const textToSend = promptText.trim();
    const filesToSend = [...attachedFiles];
    promptText = "";
    attachedFiles = [];
    if (textareaRef) textareaRef.style.height = "auto";
    onSend(textToSend, filesToSend, chatMode, selectedModelId);
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  }

  function autoResizeTextarea() {
    if (!textareaRef) return;
    textareaRef.style.height = "auto";
    textareaRef.style.height = `${Math.min(textareaRef.scrollHeight, 180)}px`;
  }
</script>

<div
  class="relative max-w-4xl mx-auto w-full rounded-2xl sm:rounded-3xl border border-border/70 bg-card/90 backdrop-blur-xl shadow-xl hover:shadow-2xl transition-all duration-200 focus-within:border-primary/60 focus-within:ring-2 focus-within:ring-primary/20 focus-within:shadow-2xl font-sans"
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
  role="region"
  aria-label="Input composer"
>
  <!-- Hidden File Input -->
  <input
    type="file"
    multiple
    bind:this={fileInputRef}
    onchange={handleFileSelect}
    class="hidden"
    aria-hidden="true"
  />

  <!-- Multi-File Attachment Chip Group Preview -->
  {#if attachedFiles.length > 0}
    <div class="px-3 pt-2">
      <AttachmentFileGroup
        files={rawFileList}
        onRemove={handleRemoveFileByIndex}
      />
    </div>
  {/if}

  <!-- File Dropzone View -->
  <FileDropzone
    files={attachedFiles}
    onAddFiles={handleAddFiles}
    onRemoveFile={handleRemoveFile}
    isOver={isDragOver}
  />

  <!-- Audio Recording Modal -->
  <AudioRecordModal
    isOpen={isAudioModalOpen}
    onClose={() => (isAudioModalOpen = false)}
    onSaveAudio={(blob) => {
      const audioFile = new File([blob], `voice_${Date.now()}.webm`, { type: "audio/webm" });
      handleAddFiles([audioFile]);
    }}
  />

  <!-- Convert to File Modal -->
  <ConvertToFileModal
    open={isConvertModalOpen}
    initialText={promptText}
    onClose={() => (isConvertModalOpen = false)}
    onConverted={(file) => {
      handleAddFiles([file]);
      promptText = "";
    }}
  />

  <!-- Textarea Area -->
  <div class="p-3">
    <textarea
      bind:this={textareaRef}
      bind:value={promptText}
      onkeydown={handleKeyDown}
      oninput={autoResizeTextarea}
      onpaste={handlePaste}
      onfocus={() => {
        setTimeout(() => {
          textareaRef?.scrollIntoView({ block: "nearest", behavior: "smooth" });
        }, 120);
      }}
      placeholder="Tanyakan apa saja atau ketik prompt... (Enter kirim, Shift+Enter baris baru)"
      rows={1}
      class="w-full resize-none bg-transparent text-sm text-foreground placeholder:text-muted-foreground focus:outline-hidden max-h-44 min-h-[44px] leading-relaxed [overflow-wrap:anywhere]"
    ></textarea>
  </div>

  <!-- Composer Toolbar Bottom -->
  <div class="relative flex items-center justify-between gap-1.5 px-2.5 pb-2.5 pt-1 border-t border-border/30 select-none">
    <!-- Fast Action Menu Popover -->
    <ComposerActionMenu
      open={isActionMenuOpen}
      onClose={() => (isActionMenuOpen = false)}
      onAttachFile={() => fileInputRef?.click()}
      onConvertToFile={() => (isConvertModalOpen = true)}
      onClearChat={onClearChat}
      onInsertPreset={(preset) => {
        promptText = promptText ? `${promptText}\n${preset}` : preset;
        autoResizeTextarea();
      }}
    />

    <!-- Left Controls (Dense, horizontally fluid without awkward line breaks) -->
    <div class="flex items-center gap-1 min-w-0 overflow-x-auto no-scrollbar py-0.5">
      <!-- Fast Actions Trigger Button (+) -->
      <Button
        variant="ghost"
        size="icon"
        class="h-7 w-7 sm:h-8 sm:w-8 shrink-0 text-muted-foreground hover:text-cyan-400 hover:bg-muted cursor-pointer rounded-lg"
        onclick={() => (isActionMenuOpen = !isActionMenuOpen)}
        title="Aksi Cepat (+)"
        aria-label="Aksi Cepat"
      >
        <Plus class="h-4 w-4" />
      </Button>

      <div class="shrink-0">
        <ModelSelector {selectedModelId} onSelectModel={onModelChange} />
      </div>

      <div class="shrink-0">
        <ModeSelector activeMode={chatMode} onSelectMode={onModeChange} />
      </div>

      <Button
        variant="ghost"
        size="icon"
        class="h-7 w-7 sm:h-8 sm:w-8 shrink-0 text-muted-foreground hover:text-cyan-400 hover:bg-muted cursor-pointer rounded-lg"
        onclick={() => fileInputRef?.click()}
        title="Lampirkan Dokumen atau Gambar"
        aria-label="Lampirkan Dokumen atau Gambar"
      >
        <Paperclip class="h-3.5 w-3.5 sm:h-4 sm:w-4" />
      </Button>

      <Button
        variant="ghost"
        size="icon"
        class="h-7 w-7 sm:h-8 sm:w-8 shrink-0 text-muted-foreground hover:text-cyan-400 hover:bg-muted cursor-pointer rounded-lg"
        onclick={() => (isAudioModalOpen = true)}
        title="Rekam Suara"
        aria-label="Rekam Suara"
      >
        <Mic class="h-3.5 w-3.5 sm:h-4 sm:w-4" />
      </Button>
    </div>

    <!-- Right Action Button (Send / Stop) -->
    <div class="flex items-center shrink-0 ml-1">
      {#if isGenerating}
        <Button
          variant="destructive"
          size="touch"
          onclick={onStop}
          class="h-8 px-2.5 sm:px-3.5 gap-1.5 font-semibold text-xs shadow-md animate-pulse cursor-pointer rounded-xl"
          aria-label="Stop Generation"
        >
          <Square class="h-3.5 w-3.5 fill-current" />
          <span class="hidden sm:inline">Stop</span>
        </Button>
      {:else}
        <Button
          variant="cyan"
          size="touch"
          onclick={handleSubmit}
          disabled={!promptText.trim() && attachedFiles.length === 0}
          class="h-8 w-8 sm:w-auto sm:px-3.5 p-0 sm:py-1 gap-1.5 font-semibold text-xs shadow-md cursor-pointer rounded-xl flex items-center justify-center"
          aria-label="Kirim Pesan"
        >
          <Send class="h-3.5 w-3.5" />
          <span class="hidden sm:inline">Kirim</span>
        </Button>
      {/if}
    </div>
  </div>
</div>
