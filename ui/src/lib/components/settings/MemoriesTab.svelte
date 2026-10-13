<script lang="ts">
  import { onMount } from "svelte";
  import Brain from "lucide-svelte/icons/brain";
  import Plus from "lucide-svelte/icons/plus";
  import Trash2 from "lucide-svelte/icons/trash-2";
  import Loader2 from "lucide-svelte/icons/loader-2";
  import Sparkles from "lucide-svelte/icons/sparkles";
  import Tag from "lucide-svelte/icons/tag";
  import Clock from "lucide-svelte/icons/clock";
  import { api } from "$lib/api";
  import type { UserMemory } from "$lib/types";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";

  let memories = $state<UserMemory[]>([]);
  let isLoading = $state(true);
  let isSaving = $state(false);
  let deletingId = $state<string | null>(null);
  let newContent = $state("");
  let newCategory = $state("preference");
  let errorMessage = $state<string | null>(null);

  async function loadMemories() {
    try {
      isLoading = true;
      errorMessage = null;
      memories = await api.listUserMemories();
    } catch (err: unknown) {
      errorMessage = err instanceof Error ? err.message : "Gagal memuat daftar memori";
    } finally {
      isLoading = false;
    }
  }

  async function handleAddMemory(e: SubmitEvent) {
    e.preventDefault();
    if (!newContent.trim()) return;

    try {
      isSaving = true;
      errorMessage = null;
      const created = await api.createUserMemory(newContent.trim(), newCategory);
      memories = [created, ...memories];
      newContent = "";
    } catch (err: unknown) {
      errorMessage = err instanceof Error ? err.message : "Gagal menyimpan memori baru";
    } finally {
      isSaving = false;
    }
  }

  async function handleDeleteMemory(id: string) {
    try {
      deletingId = id;
      await api.deleteUserMemory(id);
      memories = memories.filter((m) => m.id !== id);
    } catch (err: unknown) {
      errorMessage = err instanceof Error ? err.message : "Gagal menghapus memori";
    } finally {
      deletingId = null;
    }
  }

  function formatDate(isoString: string): string {
    try {
      const d = new Date(isoString);
      return d.toLocaleDateString("id-ID", {
        day: "numeric",
        month: "short",
        year: "numeric",
      });
    } catch {
      return isoString;
    }
  }

  onMount(() => {
    loadMemories();
  });
</script>

<div class="space-y-6 font-sans text-xs">
  <!-- Header Intro Card -->
  <div class="flex items-start justify-between p-4 rounded-xl border border-border bg-card">
    <div class="flex items-start gap-3 min-w-0">
      <div class="p-2.5 rounded-xl bg-primary/10 border border-primary/20 text-primary shrink-0">
        <Brain class="h-6 w-6" />
      </div>
      <div class="min-w-0 space-y-1">
        <div class="font-bold text-sm text-foreground [overflow-wrap:anywhere] break-words">
          Memori Jangka Panjang (Durable User Memories)
        </div>
        <p class="text-muted-foreground text-[11px] leading-relaxed [overflow-wrap:anywhere] break-words">
          Asisten AI mengingat preferensi, peran, dan konteks penting tentang Anda lintas seluruh sesi percakapan. Konteks ini diinjeksikan otomatis pada setiap interaksi.
        </p>
      </div>
    </div>
  </div>

  {#if errorMessage}
    <div class="p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs [overflow-wrap:anywhere] break-words">
      {errorMessage}
    </div>
  {/if}

  <!-- Add New Memory Card -->
  <div class="p-4 rounded-xl border border-border bg-card space-y-3">
    <div class="font-semibold text-xs text-foreground flex items-center gap-2">
      <Plus class="h-3.5 w-3.5 text-primary shrink-0" />
      <span class="[overflow-wrap:anywhere] break-words">Tambah Fakta atau Preferensi Baru</span>
    </div>

    <form onsubmit={handleAddMemory} class="flex flex-col sm:flex-row gap-2">
      <div class="flex-1 min-w-0">
        <Input
          type="text"
          bind:value={newContent}
          placeholder="Misal: Saya bekerja di Tim Statistik Sosial BPS..."
          class="h-9 text-xs"
          disabled={isSaving}
        />
      </div>
      <div class="w-full sm:w-36 shrink-0">
        <select
          bind:value={newCategory}
          disabled={isSaving}
          aria-label="Kategori memori"
          class="w-full h-9 px-2.5 rounded-md border border-input bg-background text-xs text-foreground focus-visible:outline-hidden focus-visible:ring-1 focus-visible:ring-ring"
        >
          <option value="preference">Preferensi</option>
          <option value="work">Pekerjaan</option>
          <option value="bio">Biodata</option>
          <option value="custom">Kustom</option>
        </select>
      </div>
      <Button
        type="submit"
        variant="default"
        size="sm"
        disabled={isSaving || !newContent.trim()}
        class="gap-1.5 text-xs shrink-0 min-h-[44px] sm:min-h-9 touch-manipulation whitespace-normal break-words"
      >
        {#if isSaving}
          <Loader2 class="h-3.5 w-3.5 animate-spin shrink-0" />
        {:else}
          <Plus class="h-3.5 w-3.5 shrink-0" />
        {/if}
        <span>Simpan</span>
      </Button>
    </form>
  </div>

  <!-- Memories List Section -->
  <div class="space-y-2">
    <div class="flex items-center justify-between px-1">
      <div class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider truncate">
        Daftar Memori Aktif ({memories.length})
      </div>
      {#if isLoading}
        <div class="flex items-center gap-1.5 text-[11px] text-muted-foreground shrink-0">
          <Loader2 class="h-3 w-3 animate-spin" />
          <span>Memuat...</span>
        </div>
      {/if}
    </div>

    {#if isLoading && memories.length === 0}
      <div class="flex flex-col items-center justify-center p-8 rounded-xl border border-dashed border-border text-center space-y-2">
        <Loader2 class="h-6 w-6 animate-spin text-primary" />
        <span class="text-muted-foreground text-xs">Memuat memori pengguna...</span>
      </div>
    {:else if memories.length === 0}
      <div class="flex flex-col items-center justify-center p-8 rounded-xl border border-dashed border-border text-center space-y-2 bg-muted/10">
        <Sparkles class="h-6 w-6 text-primary" />
        <div class="font-medium text-xs text-foreground">Belum Ada Memori Tersimpan</div>
        <p class="text-[11px] text-muted-foreground max-w-sm [overflow-wrap:anywhere] break-words">
          Asisten AI akan mengingat konteks penting tentang Anda, atau Anda dapat menambahkannya secara manual menggunakan formulir di atas.
        </p>
      </div>
    {:else}
      <div class="space-y-2 max-h-[260px] overflow-y-auto pr-1">
        {#each memories as mem (mem.id)}
          <div class="flex items-start justify-between p-3 rounded-xl border border-border bg-card/60 hover:bg-card transition-colors gap-3">
            <div class="flex-1 min-w-0 space-y-1">
              <div class="flex items-center gap-2 flex-wrap">
                <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-medium bg-primary/10 text-primary border border-primary/20 uppercase">
                  <Tag class="h-2.5 w-2.5 shrink-0" />
                  {mem.category}
                </span>
                <span class="inline-flex items-center gap-1 text-[10px] text-muted-foreground">
                  <Clock class="h-2.5 w-2.5 shrink-0" />
                  {formatDate(mem.created_at)}
                </span>
              </div>
              <p class="text-xs text-foreground [overflow-wrap:anywhere] break-words whitespace-pre-wrap leading-relaxed">
                {mem.content}
              </p>
            </div>
            <button
              type="button"
              onclick={() => handleDeleteMemory(mem.id)}
              disabled={deletingId === mem.id}
              aria-label="Hapus memori"
              class="p-2 rounded-lg text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors shrink-0 min-h-[44px] min-w-[44px] flex items-center justify-center cursor-pointer disabled:opacity-50 touch-manipulation"
            >
              {#if deletingId === mem.id}
                <Loader2 class="h-4 w-4 animate-spin text-destructive" />
              {:else}
                <Trash2 class="h-4 w-4" />
              {/if}
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
