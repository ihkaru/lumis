<script lang="ts">
  import UserIcon from "lucide-svelte/icons/user";
  import Shield from "lucide-svelte/icons/shield";
  import LogOut from "lucide-svelte/icons/log-out";
  import KeyRound from "lucide-svelte/icons/key-round";
  import Check from "lucide-svelte/icons/check";
  import type { User } from "$lib/types";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";

  interface Props {
    user: User | null;
    onLogout: () => void;
  }

  let { user, onLogout }: Props = $props();

  let oldPassword = $state("");
  let newPassword = $state("");
  let confirmPassword = $state("");
  let statusMessage = $state<string | null>(null);

  function handleUpdatePassword(e: SubmitEvent) {
    e.preventDefault();
    if (!newPassword || newPassword !== confirmPassword) {
      statusMessage = "Kata sandi baru tidak cocok.";
      return;
    }
    statusMessage = "Kata sandi berhasil diperbarui.";
    oldPassword = "";
    newPassword = "";
    confirmPassword = "";
    setTimeout(() => {
      statusMessage = null;
    }, 3000);
  }
</script>

<div class="space-y-6 font-sans text-xs">
  <!-- Profile Summary -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between p-4 rounded-xl border border-border bg-card gap-3">
    <div class="flex items-center gap-3 min-w-0">
      <div class="p-2.5 rounded-xl bg-primary/10 border border-primary/20 text-primary shrink-0">
        <UserIcon class="h-6 w-6" />
      </div>
      <div class="min-w-0 space-y-0.5">
        <div class="font-bold text-sm text-foreground truncate [overflow-wrap:anywhere]">
          {user ? user.username : "Pengguna Tamu"}
        </div>
        <div class="flex items-center gap-1.5 text-muted-foreground text-[11px]">
          <Shield class="h-3 w-3 text-primary shrink-0" />
          <span class="capitalize truncate">{user ? user.role : "Guest"}</span>
        </div>
      </div>
    </div>
    <Button
      variant="outline"
      size="sm"
      onclick={onLogout}
      class="gap-1.5 text-xs text-destructive hover:bg-destructive/10 min-h-[44px] sm:min-h-9 touch-manipulation shrink-0 self-start sm:self-auto"
    >
      <LogOut class="h-3.5 w-3.5" />
      <span>Keluar</span>
    </Button>
  </div>

  <!-- Password Update Section -->
  <div class="p-4 rounded-xl border border-border bg-card space-y-4">
    <div class="font-semibold text-sm text-foreground flex items-center gap-2">
      <KeyRound class="h-4 w-4 text-primary shrink-0" />
      <span class="[overflow-wrap:anywhere] break-words">Ubah Kata Sandi Mandiri</span>
    </div>

    {#if statusMessage}
      <div class="p-2.5 rounded-lg bg-emerald-950/40 border border-emerald-800/40 text-emerald-400 text-xs flex items-center gap-2 [overflow-wrap:anywhere] break-words">
        <Check class="h-4 w-4 shrink-0" />
        <span>{statusMessage}</span>
      </div>
    {/if}

    <form onsubmit={handleUpdatePassword} class="space-y-3">
      <div class="space-y-1">
        <label for="oldPassword" class="text-[11px] text-muted-foreground block">Kata Sandi Lama</label>
        <Input id="oldPassword" type="password" bind:value={oldPassword} placeholder="••••••••" class="h-9 text-xs" />
      </div>
      <div class="space-y-1">
        <label for="newPassword" class="text-[11px] text-muted-foreground block">Kata Sandi Baru</label>
        <Input id="newPassword" type="password" bind:value={newPassword} placeholder="••••••••" class="h-9 text-xs" />
      </div>
      <div class="space-y-1">
        <label for="confirmPassword" class="text-[11px] text-muted-foreground block">Ulangi Kata Sandi Baru</label>
        <Input id="confirmPassword" type="password" bind:value={confirmPassword} placeholder="••••••••" class="h-9 text-xs" />
      </div>
      <div class="flex justify-end pt-1">
        <Button type="submit" variant="default" size="sm" class="gap-1.5 text-xs min-h-[44px] sm:min-h-9 touch-manipulation">
          <Check class="h-3.5 w-3.5" />
          <span>Simpan Kata Sandi</span>
        </Button>
      </div>
    </form>
  </div>
</div>
