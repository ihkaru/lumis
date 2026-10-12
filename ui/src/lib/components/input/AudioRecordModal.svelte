<script lang="ts">
  import Mic from "lucide-svelte/icons/mic";
  import Square from "lucide-svelte/icons/square";
  import X from "lucide-svelte/icons/x";
  import Check from "lucide-svelte/icons/check";
  import AlertCircle from "lucide-svelte/icons/circle-alert";
  import { Button } from "$lib/components/ui/button";

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onSaveAudio: (audioBlob: Blob) => void;
  }

  let { isOpen = false, onClose, onSaveAudio }: Props = $props();

  let isRecording = $state(false);
  let durationSec = $state(0);
  let timerInterval: ReturnType<typeof setInterval> | null = null;
  let mediaRecorder: MediaRecorder | null = null;
  let audioChunks: Blob[] = [];
  let error = $state<string | null>(null);

  async function startRecording() {
    error = null;
    audioChunks = [];
    durationSec = 0;

    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      mediaRecorder = new MediaRecorder(stream);

      mediaRecorder.ondataavailable = (e) => {
        if (e.data.size > 0) audioChunks.push(e.data);
      };

      mediaRecorder.start();
      isRecording = true;

      timerInterval = setInterval(() => {
        durationSec += 1;
      }, 1000);
    } catch (e: any) {
      error = e.message || "Izin mikrofon ditolak atau tidak tersedia.";
    }
  }

  function stopRecording(save = true) {
    if (timerInterval) clearInterval(timerInterval);
    isRecording = false;

    if (mediaRecorder && mediaRecorder.state !== "inactive") {
      mediaRecorder.onstop = () => {
        if (save && audioChunks.length > 0) {
          const blob = new Blob(audioChunks, { type: "audio/webm" });
          onSaveAudio(blob);
        }
        mediaRecorder?.stream.getTracks().forEach((t) => t.stop());
        onClose();
      };
      mediaRecorder.stop();
    } else {
      onClose();
    }
  }

  function formatTime(sec: number) {
    const m = Math.floor(sec / 60);
    const s = sec % 60;
    return `${m}:${s < 10 ? "0" : ""}${s}`;
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm p-4 font-sans select-none">
    <div class="w-full max-w-sm rounded-2xl border border-border bg-card p-6 shadow-2xl space-y-4 text-center">
      <div class="flex items-center justify-between">
        <h3 class="text-sm font-semibold text-foreground flex items-center gap-2">
          <Mic class="h-4 w-4 text-cyan-400" />
          <span>Rekam Pesan Suara</span>
        </h3>
        <button
          type="button"
          onclick={() => stopRecording(false)}
          class="text-muted-foreground hover:text-foreground cursor-pointer"
          aria-label="Tutup"
        >
          <X class="h-4 w-4" />
        </button>
      </div>

      {#if error}
        <div class="flex items-center gap-2 rounded-lg bg-destructive/15 border border-destructive/30 p-3 text-xs text-red-400 text-left">
          <AlertCircle class="h-4 w-4 shrink-0" />
          <span>{error}</span>
        </div>
      {/if}

      <!-- Wave Visualizer Simulation -->
      <div class="flex items-center justify-center h-20 gap-1.5 py-4">
        {#each [40, 70, 95, 60, 85, 50, 90, 65, 45] as h, i}
          <div
            class="w-1.5 rounded-full bg-cyan-500 transition-all duration-150 {isRecording ? 'animate-pulse' : 'opacity-40'}"
            style="height: {isRecording ? `${h}%` : '20%'}; animation-delay: {i * 100}ms"
          ></div>
        {/each}
      </div>

      <div class="text-2xl font-mono font-bold text-foreground">
        {formatTime(durationSec)}
      </div>

      <div class="flex justify-center gap-3 pt-2">
        {#if !isRecording}
          <Button variant="cyan" onclick={startRecording} class="gap-2 px-6">
            <Mic class="h-4 w-4" />
            <span>Mulai Rekam</span>
          </Button>
        {:else}
          <Button variant="destructive" onclick={() => stopRecording(false)} class="gap-2">
            <Square class="h-4 w-4" />
            <span>Batal</span>
          </Button>
          <Button variant="cyan" onclick={() => stopRecording(true)} class="gap-2">
            <Check class="h-4 w-4" />
            <span>Kirim Rekaman</span>
          </Button>
        {/if}
      </div>
    </div>
  </div>
{/if}
