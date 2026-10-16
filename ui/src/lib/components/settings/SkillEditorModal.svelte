<script lang="ts">
  import Sparkles from "lucide-svelte/icons/sparkles";
  import X from "lucide-svelte/icons/x";
  import Check from "lucide-svelte/icons/check";
  import { Dialog, DialogOverlay, DialogContent, DialogTitle, DialogClose } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { skillsApi } from "$lib/skills-api";
  import type { Skill, SkillSummary } from "$lib/types";

  interface Props {
    open: boolean;
    skillToEdit?: SkillSummary | null;
    onClose: () => void;
    onSaved: () => void;
  }

  let { open = false, skillToEdit = null, onClose, onSaved }: Props = $props();

  let name = $state("");
  let displayTitle = $state("");
  let category = $state("general");
  let description = $state("");
  let body = $state("");
  let alwaysApply = $state(false);
  let userInvocable = $state(true);

  let isSaving = $state(false);
  let isLoadingDetail = $state(false);
  let errorMsg = $state<string | null>(null);

  // Sync state when opening modal
  $effect(() => {
    if (!open) {
      name = "";
      displayTitle = "";
      category = "general";
      description = "";
      body = "";
      alwaysApply = false;
      userInvocable = true;
      errorMsg = null;
      return;
    }

    if (skillToEdit) {
      name = skillToEdit.name;
      displayTitle = skillToEdit.display_title;
      category = skillToEdit.category || "general";
      description = skillToEdit.description;
      alwaysApply = skillToEdit.always_apply;
      userInvocable = skillToEdit.user_invocable;
      isLoadingDetail = true;

      skillsApi
        .getSkill(skillToEdit.id)
        .then((res) => {
          body = res.skill.body;
          isLoadingDetail = false;
        })
        .catch(() => {
          body = "";
          isLoadingDetail = false;
        });
    }
  });

  function formatKebabCase(val: string) {
    return val
      .toLowerCase()
      .replace(/[^a-z0-9_-]/g, "-")
      .replace(/-+/g, "-");
  }

  async function handleSave() {
    errorMsg = null;
    const cleanName = formatKebabCase(name.trim());
    if (!cleanName || cleanName.length < 2) {
      errorMsg = "Nama skill harus minimal 2 karakter (hanya huruf kecil, angka, dan '-') ";
      return;
    }
    if (!description.trim()) {
      errorMsg = "Deskripsi skill wajib diisi untuk pengindeksan FTS5.";
      return;
    }
    if (!body.trim()) {
      errorMsg = "Instruksi prompt / isi keahlian (body) wajib diisi.";
      return;
    }

    isSaving = true;
    try {
      if (skillToEdit) {
        await skillsApi.updateSkill(skillToEdit.id, {
          expected_version: skillToEdit.version,
          display_title: displayTitle.trim() || undefined,
          description: description.trim(),
          body: body.trim(),
          category: category.trim() || undefined,
          always_apply: alwaysApply,
          user_invocable: userInvocable,
        });
      } else {
        await skillsApi.createSkill({
          name: cleanName,
          display_title: displayTitle.trim() || cleanName,
          description: description.trim(),
          body: body.trim(),
          category: category.trim() || "general",
          always_apply: alwaysApply,
          user_invocable: userInvocable,
        });
      }
      onSaved();
      onClose();
    } catch (err: any) {
      errorMsg = err.message || "Gagal menyimpan skill.";
    } finally {
      isSaving = false;
    }
  }
</script>

<Dialog {open}>
  <DialogOverlay {open} onclick={onClose} />
  <DialogContent
    {open}
    {onClose}
    class="w-[95vw] sm:w-[90vw] md:max-w-2xl h-[90vh] max-h-[680px] p-0 overflow-hidden flex flex-col rounded-2xl border border-border bg-card shadow-2xl font-sans"
  >
    <header class="flex items-center justify-between border-b border-border px-5 py-3.5 bg-card shrink-0">
      <DialogTitle class="text-sm font-semibold flex items-center gap-2">
        <Sparkles class="size-4 text-primary" />
        <span>{skillToEdit ? `Edit Skill: $${skillToEdit.name}` : "Tambah Skill Enterprise Baru"}</span>
      </DialogTitle>
      <DialogClose onclick={onClose} />
    </header>

    <div class="flex-1 overflow-y-auto p-5 space-y-4 smooth-momentum-scroll">
      {#if errorMsg}
        <div class="p-3 bg-destructive/10 border border-destructive/30 rounded-xl text-xs text-destructive">
          {errorMsg}
        </div>
      {/if}

      {#if isLoadingDetail}
        <div class="py-8 text-center text-xs text-muted-foreground flex items-center justify-center gap-2">
          <div class="size-4 border-2 border-primary border-t-transparent rounded-full animate-spin"></div>
          <span>Memuat detail keahlian...</span>
        </div>
      {:else}
        <!-- Name & Display Title -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div>
            <label for="skill-name" class="block text-xs font-medium text-foreground mb-1">
              Nama Singkat ($identifier) <span class="text-destructive">*</span>
            </label>
            <div class="relative">
              <span class="absolute left-3 top-1/2 -translate-y-1/2 font-mono text-xs text-muted-foreground pointer-events-none">$</span>
              <input
                id="skill-name"
                type="text"
                bind:value={name}
                oninput={(e) => (name = formatKebabCase((e.target as HTMLInputElement).value))}
                disabled={!!skillToEdit}
                placeholder="misal: data-analyst"
                class="w-full pl-6 pr-3 py-2 bg-muted/40 border border-border/80 rounded-xl text-xs font-mono text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary min-h-[44px] disabled:opacity-60"
              />
            </div>
            <p class="text-[10px] text-muted-foreground mt-1">Digunakan untuk memanggil skill dengan tanda $ di chat.</p>
          </div>

          <div>
            <label for="skill-title" class="block text-xs font-medium text-foreground mb-1">
              Judul Tampilan (Display Title)
            </label>
            <input
              id="skill-title"
              type="text"
              bind:value={displayTitle}
              placeholder="misal: Enterprise Data Analyst"
              class="w-full px-3 py-2 bg-muted/40 border border-border/80 rounded-xl text-xs text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary min-h-[44px]"
            />
          </div>
        </div>

        <!-- Category & Options -->
        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
          <div>
            <label for="skill-cat" class="block text-xs font-medium text-foreground mb-1">Kategori</label>
            <input
              id="skill-cat"
              type="text"
              bind:value={category}
              placeholder="analysis, code, writing..."
              class="w-full px-3 py-2 bg-muted/40 border border-border/80 rounded-xl text-xs text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary min-h-[44px]"
            />
          </div>

          <div class="flex items-center gap-2 pt-4">
            <input
              id="skill-invocable"
              type="checkbox"
              bind:checked={userInvocable}
              class="size-4 rounded border-border text-primary focus:ring-primary cursor-pointer"
            />
            <label for="skill-invocable" class="text-xs text-foreground cursor-pointer select-none">
              Dapat Dipanggil ($)
            </label>
          </div>

          <div class="flex items-center gap-2 pt-4">
            <input
              id="skill-always"
              type="checkbox"
              bind:checked={alwaysApply}
              class="size-4 rounded border-border text-primary focus:ring-primary cursor-pointer"
            />
            <label for="skill-always" class="text-xs text-foreground cursor-pointer select-none">
              Selalu Aktif (Always Apply)
            </label>
          </div>
        </div>

        <!-- Description (Indexed for FTS5 discovery) -->
        <div>
          <label for="skill-desc" class="block text-xs font-medium text-foreground mb-1">
            Deskripsi Singkat (FTS5 Searchable) <span class="text-destructive">*</span>
          </label>
          <textarea
            id="skill-desc"
            bind:value={description}
            rows={2}
            placeholder="Jelaskan kapan skill ini relevan. Deskripsi ini dicocokkan otomatis oleh mesin FTS5 saat pengguna mengirim prompt..."
            class="w-full px-3 py-2 bg-muted/40 border border-border/80 rounded-xl text-xs text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary resize-none leading-relaxed"
          ></textarea>
        </div>

        <!-- Body / Prompt Instructions -->
        <div>
          <label for="skill-body" class="block text-xs font-medium text-foreground mb-1">
            Instruksi / Prompt Sistem (Markdown Body) <span class="text-destructive">*</span>
          </label>
          <textarea
            id="skill-body"
            bind:value={body}
            rows={7}
            placeholder="Tuliskan petunjuk, aturan, persona, atau template yang wajib diikuti AI ketika skill ini diaktifkan..."
            class="w-full px-3 py-2 bg-muted/40 border border-border/80 rounded-xl text-xs font-mono text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary resize-y leading-relaxed"
          ></textarea>
          <p class="text-[10px] text-muted-foreground mt-1">Hanya diinjeksi saat di-invoke atau aktif, sehingga menghemat kuota token.</p>
        </div>
      {/if}
    </div>

    <footer class="flex items-center justify-end gap-2 border-t border-border px-5 py-3 bg-muted/20 shrink-0">
      <Button variant="ghost" size="touch" onclick={onClose} disabled={isSaving} class="text-xs min-h-[44px]">
        Batal
      </Button>
      <Button
        variant="cyan"
        size="touch"
        onclick={handleSave}
        disabled={isSaving || isLoadingDetail}
        class="gap-1.5 text-xs font-semibold min-h-[44px] px-4 cursor-pointer"
      >
        <Check class="size-3.5" />
        <span>{isSaving ? "Menyimpan..." : "Simpan Skill"}</span>
      </Button>
    </footer>
  </DialogContent>
</Dialog>
