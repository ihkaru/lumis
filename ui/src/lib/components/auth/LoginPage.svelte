<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { router } from "$lib/router.svelte";
  import Shield from "lucide-svelte/icons/shield";
  import LogIn from "lucide-svelte/icons/log-in";
  import Loader2 from "lucide-svelte/icons/loader-2";
  import KeyRound from "lucide-svelte/icons/key-round";
  import UserIcon from "lucide-svelte/icons/user";

  import { api } from "$lib/api";

  interface Props {
    brandTitle?: string;
    onLoginSuccess?: (username: string) => void;
  }

  let { brandTitle = "Lumis Enterprise", onLoginSuccess }: Props = $props();

  let username = $state("");
  let password = $state("");
  let isPending = $state(false);
  let errorMessage = $state<string | null>(null);

  async function handleLogin(e: SubmitEvent) {
    e.preventDefault();
    if (!username.trim() || isPending) return;

    isPending = true;
    errorMessage = null;

    try {
      const authRes = await api.login(username.trim(), password || undefined);
      if (onLoginSuccess && authRes.user) {
        onLoginSuccess(authRes.user.username);
      }
      const redirectTarget = router.query.redirect
        ? decodeURIComponent(router.query.redirect)
        : "/chat/new";
      router.navigate(redirectTarget);
    } catch (err: any) {
      errorMessage = err.message || "Autentikasi gagal. Periksa username dan password.";
    } finally {
      isPending = false;
    }
  }

  function handleSsoRedirect() {
    window.location.href = "/api/auth/sso";
  }
</script>

<div class="flex h-screen w-screen items-center justify-center bg-background px-4 font-sans select-none">
  <div class="w-full max-w-md rounded-2xl border border-border bg-card p-8 shadow-2xl space-y-6">
    <div class="text-center space-y-2">
      <div class="mx-auto flex h-12 w-12 items-center justify-center rounded-xl bg-cyan-950/60 border border-cyan-800/40 text-cyan-400">
        <Shield class="h-6 w-6" />
      </div>
      <h1 class="text-2xl font-bold tracking-tight text-foreground">
        Masuk ke {brandTitle}
      </h1>
      <p class="text-xs text-muted-foreground">
        Akses antarmuka percakapan AI berkeamanan tinggi dan terintegrasi rIDM SAML 2.0
      </p>
    </div>

    {#if errorMessage}
      <div class="rounded-lg bg-destructive/15 border border-destructive/30 px-3.5 py-2.5 text-xs text-red-400 break-words [overflow-wrap:anywhere]">
        {errorMessage}
      </div>
    {/if}

    <form onsubmit={handleLogin} class="space-y-4">
      <div class="space-y-1.5">
        <label for="username" class="text-xs font-medium text-foreground flex items-center gap-1.5">
          <UserIcon class="h-3.5 w-3.5 text-muted-foreground" />
          <span>Username / NIP</span>
        </label>
        <Input
          id="username"
          type="text"
          placeholder="admin atau nama pengguna"
          bind:value={username}
          required
          disabled={isPending}
          class="h-11"
        />
      </div>

      <div class="space-y-1.5">
        <label for="password" class="text-xs font-medium text-foreground flex items-center gap-1.5">
          <KeyRound class="h-3.5 w-3.5 text-muted-foreground" />
          <span>Kata Sandi</span>
        </label>
        <Input
          id="password"
          type="password"
          placeholder="••••••••"
          bind:value={password}
          disabled={isPending}
          class="h-11"
        />
      </div>

      <Button
        type="submit"
        variant="cyan"
        size="touch"
        disabled={isPending || !username.trim()}
        class="w-full h-11 text-sm font-semibold gap-2 mt-2"
      >
        {#if isPending}
          <Loader2 class="h-4 w-4 animate-spin" />
          <span>Memverifikasi...</span>
        {:else}
          <LogIn class="h-4 w-4" />
          <span>Masuk Aplikasi</span>
        {/if}
      </Button>
    </form>

    <div class="relative flex items-center justify-center">
      <div class="absolute inset-0 flex items-center">
        <div class="w-full border-t border-border"></div>
      </div>
      <span class="relative bg-card px-2 text-[11px] text-muted-foreground uppercase font-mono tracking-wider">
        Atau
      </span>
    </div>

    <Button
      variant="outline"
      size="touch"
      onclick={handleSsoRedirect}
      class="w-full h-11 text-xs font-medium gap-2 border-border/80 hover:bg-muted/50"
    >
      <Shield class="h-4 w-4 text-cyan-400" />
      <span>Masuk via Single Sign-On (SAML 2.0 / OIDC)</span>
    </Button>
  </div>
</div>
