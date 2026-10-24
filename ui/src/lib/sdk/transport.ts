import type {
  ArtifactSpecification,
  CursorPage,
  Message,
  ModelDescriptor,
  Session,
  SessionTask,
  User,
} from "../types";
import type { ILumisTransport, TransportConfig } from "./types";

export class HttpSseTransport implements ILumisTransport {
  private baseUrl: string;
  private config: TransportConfig;

  constructor(config: TransportConfig = {}) {
    this.config = config;
    this.baseUrl = (config.baseUrl || "").replace(/\/+$/, "");
  }

  private async getHeaders(extraHeaders?: HeadersInit): Promise<Headers> {
    const headers = new Headers(extraHeaders);
    if (!headers.has("Content-Type")) {
      headers.set("Content-Type", "application/json");
    }

    if (this.config.headers) {
      for (const [k, v] of Object.entries(this.config.headers)) {
        if (!headers.has(k)) headers.set(k, v);
      }
    }

    let token = this.config.embedToken;
    if (!token && this.config.getAuthToken) {
      const resolved = await this.config.getAuthToken();
      if (resolved) token = resolved;
    }

    if (token && !headers.has("Authorization")) {
      headers.set("Authorization", `Bearer ${token}`);
    }

    return headers;
  }

  private resolveUrl(path: string): string {
    const cleanPath = path.startsWith("/") ? path : `/${path}`;
    return this.baseUrl ? `${this.baseUrl}${cleanPath}` : cleanPath;
  }

  async fetchApi<T>(path: string, options: RequestInit = {}): Promise<T> {
    const url = this.resolveUrl(path);
    const headers = await this.getHeaders(options.headers);
    const res = await fetch(url, { ...options, headers });

    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: `HTTP ${res.status}: ${res.statusText}` }));
      throw new Error(err.error || `Request failed with status ${res.status}`);
    }
    return res.json();
  }

  async getMe(): Promise<{ authenticated: boolean; user?: User; role: string }> {
    return this.fetchApi("/api/auth/me");
  }

  async getModels(): Promise<ModelDescriptor[]> {
    return this.fetchApi("/api/models");
  }

  async listSessions(cursorTime?: string, cursorId?: string, limit = 20): Promise<CursorPage<Session>> {
    const params = new URLSearchParams();
    if (cursorTime) params.set("cursor_time", cursorTime);
    if (cursorId) params.set("cursor_id", cursorId);
    params.set("limit", String(limit));
    return this.fetchApi(`/api/sessions?${params.toString()}`);
  }

  async createSession(title: string, modelId = "my9model-free"): Promise<Session> {
    return this.fetchApi("/api/sessions", {
      method: "POST",
      body: JSON.stringify({ title, model_id: modelId }),
    });
  }

  async listMessages(sessionId: string, cursorTime?: string, cursorId?: string, limit = 20): Promise<CursorPage<Message>> {
    const params = new URLSearchParams();
    if (cursorTime) params.set("cursor_time", cursorTime);
    if (cursorId) params.set("cursor_id", cursorId);
    params.set("limit", String(limit));
    return this.fetchApi(`/api/sessions/${sessionId}/messages?${params.toString()}`);
  }

  async listSessionTasks(sessionId: string): Promise<SessionTask[]> {
    try {
      return await this.fetchApi(`/api/sessions/${sessionId}/tasks`);
    } catch {
      return [];
    }
  }

  async sendMessage(
    sessionId: string,
    content: string,
    modelId?: string,
    metadata?: Record<string, unknown>
  ): Promise<{ status: string }> {
    return this.fetchApi(`/api/sessions/${sessionId}/messages`, {
      method: "POST",
      body: JSON.stringify({ content, model_id: modelId, metadata }),
    });
  }

  connectStream(onToken: (data: string) => void): EventSource {
    let streamUrl = this.resolveUrl("/api/stream");
    if (this.config.embedToken) {
      const sep = streamUrl.includes("?") ? "&" : "?";
      streamUrl = `${streamUrl}${sep}token=${encodeURIComponent(this.config.embedToken)}`;
    }

    const es = new EventSource(streamUrl);
    es.addEventListener("token", (e) => onToken(e.data));
    return es;
  }

  async parseArtifacts(text: string): Promise<ArtifactSpecification[]> {
    const data = await this.fetchApi<{ artifacts: ArtifactSpecification[] }>("/api/artifacts/parse", {
      method: "POST",
      body: JSON.stringify({ text }),
    });
    return data.artifacts || [];
  }
}

export const defaultTransport = new HttpSseTransport();
