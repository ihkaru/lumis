// ==============================================================================
// Reactive Client-Side Router for Lumis UI (Svelte 5 Runes)
// Zero-Reload Stateful Deep-Linking & History Management (<350 lines)
// ==============================================================================

export type RouteGuard = (
  to: string,
  from: string
) => boolean | string | void | Promise<boolean | string | void>;

function parseLocation(): {
  path: string;
  params: Record<string, string>;
  query: Record<string, string>;
} {
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
  if (chatMatch && chatMatch[1] !== "new") {
    params.id = chatMatch[1];
  }

  return { path: pathname || "/", params, query };
}

class RouterState {
  path = $state("/");
  params = $state<Record<string, string>>({});
  query = $state<Record<string, string>>({});
  private guards: RouteGuard[] = [];

  constructor() {
    if (typeof window !== "undefined") {
      const init = parseLocation();
      this.path = init.path;
      this.params = init.params;
      this.query = init.query;

      window.addEventListener("popstate", () => {
        this.handleNavigation(parseLocation().path, false);
      });
      window.addEventListener("hashchange", () => {
        this.handleNavigation(parseLocation().path, false);
      });
    }
  }

  beforeEach(guard: RouteGuard) {
    this.guards.push(guard);
  }

  private isNavigating = false;

  private async handleNavigation(to: string, replace = false) {
    if (this.isNavigating) return;
    this.isNavigating = true;
    try {
      const from = this.path;
      for (const guard of this.guards) {
        const verdict = await guard(to, from);
        if (verdict === false) return;
        if (typeof verdict === "string" && verdict !== to) {
          const cleanVerdict = verdict.split("?")[0];
          const cleanTo = to.split("?")[0];
          if (cleanVerdict !== cleanTo) {
            this.isNavigating = false;
            return this.navigate(verdict, true);
          }
        }
      }

      if (typeof window !== "undefined") {
        const targetUrl = window.location.hash.startsWith("#/") ? `#${to}` : to;
        if (replace) {
          window.history.replaceState({}, "", targetUrl);
        } else {
          window.history.pushState({}, "", targetUrl);
        }
      }

      const cur = parseLocation();
      this.path = cur.path;
      this.params = cur.params;
      this.query = cur.query;
    } finally {
      this.isNavigating = false;
    }
  }

  async navigate(to: string, replace = false) {
    if (typeof window === "undefined") return;
    await this.handleNavigation(to, replace);
  }
}

export const router = new RouterState();
