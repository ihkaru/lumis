// ==============================================================================
// Reactive Client-Side Router for Lumis UI (Svelte 5 Runes)
// Zero-Reload Stateful Deep-Linking & History Management (<350 lines)
// ==============================================================================

function parseLocation(): { path: string; params: Record<string, string>; query: Record<string, string> } {
  if (typeof window === "undefined") {
    return { path: "/", params: {}, query: {} };
  }

  const hash = window.location.hash.replace(/^#/, "");
  // Support either HTML5 path or hash-based routing for static hosting
  const rawPath = hash.startsWith("/") ? hash : window.location.pathname;
  const [pathname, search] = rawPath.split("?");

  const query: Record<string, string> = {};
  if (search) {
    const sp = new URLSearchParams(search);
    sp.forEach((v, k) => {
      query[k] = v;
    });
  }

  const params: Record<string, string> = {};
  const chatMatch = pathname.match(/^\/chat\/([a-zA-Z0-9_-]+)$/);
  if (chatMatch) {
    params.id = chatMatch[1];
  }

  return { path: pathname || "/", params, query };
}

class RouterState {
  path = $state("/");
  params = $state<Record<string, string>>({});
  query = $state<Record<string, string>>({});

  constructor() {
    if (typeof window !== "undefined") {
      const init = parseLocation();
      this.path = init.path;
      this.params = init.params;
      this.query = init.query;

      window.addEventListener("popstate", () => {
        this.sync();
      });
      window.addEventListener("hashchange", () => {
        this.sync();
      });
    }
  }

  private sync() {
    const cur = parseLocation();
    this.path = cur.path;
    this.params = cur.params;
    this.query = cur.query;
  }

  navigate(to: string, replace = false) {
    if (typeof window === "undefined") return;

    // Use hash routing if current window uses hash, or pushState
    const targetUrl = window.location.hash.startsWith("#/") ? `#${to}` : to;

    if (replace) {
      window.history.replaceState({}, "", targetUrl);
    } else {
      window.history.pushState({}, "", targetUrl);
    }

    this.sync();
  }
}

export const router = new RouterState();
