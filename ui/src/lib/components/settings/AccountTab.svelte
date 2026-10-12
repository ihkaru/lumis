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
  <div class="flex items-center justify-between p-4 rounded-xl border border-border bg-card">
    <div class="flex items-center gap-3">
      <div class="p-2.5 rounded-xl bg-cyan-950/60 border border-cyan-800/40 text-cyan-400">
        <UserIcon class="h-6 w-6" />
      </div>
      <div>
        <div class="font-bold text-sm text-foreground">
          {user ? user.username : "Pengguna Tamu"}
        </div>
        <div class="flex items-center gap-1.5 text-muted-foreground text-[11px] mt-0.5">
          <Shield class="h-3 w-3 text-cyan-400" />
          <span class="capitalize">{user ? user.role : "Guest"}</span>
        </div>
      </div>
    </div>
    <Button variant="outline" size="sm" onclick={onLogout} class="gap-1.5 text-xs text-destructive hover:bg-destructive/10">
      <LogOut class="h-3.5 w-3.5" />
      <span>Keluar</span>
    </Button>
  </div>

  <!-- Password Update Section -->
  <div class="p-4 rounded-xl border border-border bg-card space-y-4">
    <div class="font-semibold text-sm text-foreground flex items-center gap-2">
      <KeyRound class="h-4 w-4 text-cyan-400" />
      <span>Ubah Kata Sandi Mandiri</span>
    </div>

    {#if statusMessage}
      <div class="p-2.5 rounded-lg bg-emerald-950/40 border border-emerald-800/40 text-emerald-400 text-xs flex items-center gap-2">
        <Check class="h-4 w-4" />
        <span>{statusMessage}</span>
      </div>
    {/if}

    <form onsubmit={handleUpdatePassword} class="space-y-3">
      <div class="space-y-1">
        <label for="oldPassword" class="text-[11px] text-muted-foreground">Kata Sandi Lama</label>
        <Input id="oldPassword" type="password" bind:value={oldPassword} placeholder="••••••••" class="h-9 text-xs" />
      </div>
      <div class="space-y-1">
        <label for="newPassword" class="text-[11px] text-muted-foreground">Kata Sandi Baru</label>
        <Input id="newPassword" type="password" bind:value={newPassword} placeholder="••••••••" class="h-9 text-xs" />
      </div>
      <div class="space-y-1">
        <label for="confirmPassword" class="text-[11px] text-muted-foreground">Ulangi Kata Sandi Baru</label>
        <Input id="confirmPassword" type="password" bind:value={confirmPassword} placeholder="••••••••" class="h-9 text-xs" />
      </div>
      <div class="flex justify-end pt-1">
        <Button type="submit" variant="cyan" size="sm" class="gap-1.5 text-xs">
          <Check class="h-3.5 w-3.5" />
          <span>Simpan Kata Sandi</span>
        </Button>
      </div>
    </form>
  </div>
</div>
