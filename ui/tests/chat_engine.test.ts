import { describe, expect, it } from "bun:test";

// Polyfill Svelte 5 compiler runes for Bun test environment
if (typeof (globalThis as any).$state === "undefined") {
  (globalThis as any).$state = <T>(v: T): T => v;
  (globalThis as any).$derived = <T>(v: T): T => v;
  (globalThis as any).$effect = (fn: () => void) => fn();
}

import { createLumisChat } from "../src/lib/sdk/chatEngine.svelte";
import { HttpSseTransport } from "../src/lib/sdk/transport";
import type { CursorPage, Message, ModelDescriptor, Session, SessionTask, User } from "../src/lib/types";
import type { ILumisTransport } from "../src/lib/sdk/types";

class TestTransport implements ILumisTransport {
  sentPayloads: Array<{ sessionId: string; content: string; metadata?: Record<string, unknown> }> = [];
  createdSessions: string[] = [];

  async fetchApi<T>(path: string, options?: RequestInit): Promise<T> {
    return {} as T;
  }
  async getMe(): Promise<{ authenticated: boolean; user?: User; role: string }> {
    return { authenticated: true, role: "member" };
  }
  async getModels(): Promise<ModelDescriptor[]> {
    return [];
  }
  async listSessions(): Promise<CursorPage<Session>> {
    return { items: [], has_more: false };
  }
  async createSession(title: string, modelId = "test-model"): Promise<Session> {
    const id = "sess_" + Date.now();
    this.createdSessions.push(title);
    return {
      id,
      user_id: "usr_1",
      title,
      model_id: modelId,
      is_pinned: false,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
  }
  async listMessages(sessionId: string): Promise<CursorPage<Message>> {
    return {
      items: [
        {
          id: "m1",
          session_id: sessionId,
          role: "user",
          content: "Hello from previous chat",
          created_at: new Date().toISOString(),
        },
      ],
      has_more: false,
    };
  }
  async listSessionTasks(sessionId: string): Promise<SessionTask[]> {
    return [];
  }
  async sendMessage(sessionId: string, content: string, modelId?: string, metadata?: Record<string, unknown>): Promise<{ status: string }> {
    this.sentPayloads.push({ sessionId, content, metadata });
    return { status: "queued" };
  }
  connectStream(onToken: (data: string) => void): () => void {
    return () => {};
  }
  async parseArtifacts(text: string) {
    return [];
  }
}

describe("LumisChatEngine & Transport Unit Tests (Phase 19)", () => {
  it("should initialize engine with empty state and custom context data", () => {
    const transport = new TestTransport();
    const engine = createLumisChat(transport, {
      contextData: { activeRowId: "ROW-881", hostApp: "ERP-Core" },
    });

    expect(engine.messages.length).toBe(0);
    expect(engine.isGenerating).toBe(false);
    expect(engine.contextData.activeRowId).toBe("ROW-881");
  });

  it("should send message, append optimistic messages and inject host context", async () => {
    const transport = new TestTransport();
    const engine = createLumisChat(transport, {
      contextData: { tenant: "bps-testing" },
    });

    await engine.sendMessage("Berapa jumlah penduduk tahun 2025?");

    expect(engine.messages.length).toBe(2);
    expect(engine.messages[0].role).toBe("user");
    expect(engine.messages[0].content).toBe("Berapa jumlah penduduk tahun 2025?");
    expect(engine.messages[0].metadata?.host_context).toEqual({ tenant: "bps-testing" });
    expect(engine.messages[1].role).toBe("assistant");
    expect(transport.sentPayloads.length).toBe(1);
    expect(transport.sentPayloads[0].content).toBe("Berapa jumlah penduduk tahun 2025?");
  });

  it("should process streaming chunk events and update assistant message content", () => {
    const transport = new TestTransport();
    const engine = createLumisChat(transport);
    engine.activeSessionId = "sess_100";

    engine.handleStreamEvent(
      JSON.stringify({
        event: "chunk",
        session_id: "sess_100",
        delta: "Halo! ",
        is_done: false,
      })
    );

    expect(engine.messages.length).toBe(1);
    expect(engine.messages[0].content).toBe("Halo! ");

    engine.handleStreamEvent(
      JSON.stringify({
        event: "chunk",
        session_id: "sess_100",
        delta: "Ada yang bisa dibantu?",
        is_done: true,
      })
    );

    expect(engine.messages[0].content).toBe("Halo! Ada yang bisa dibantu?");
    expect(engine.isGenerating).toBe(false);
  });

  it("should execute registered frontend tools when tool_call_start matches", async () => {
    const transport = new TestTransport();
    const engine = createLumisChat(transport);
    engine.activeSessionId = "sess_200";

    let toolExecutedWith: any = null;
    engine.registerFrontendTool({
      name: "host_navigate",
      description: "Navigate host router",
      parameters: { route: "string" },
      execute: (args) => {
        toolExecutedWith = args;
        return { success: true };
      },
    });

    engine.handleStreamEvent(
      JSON.stringify({
        event: "tool_call_start",
        session_id: "sess_200",
        tool: "host_navigate",
        args: { route: "/dashboard/metrics" },
      })
    );

    // Yield microtask queue
    await new Promise((r) => setTimeout(r, 10));

    expect(toolExecutedWith).toEqual({ route: "/dashboard/metrics" });
  });

  it("should stop generating and abort active controller", () => {
    const transport = new TestTransport();
    const engine = createLumisChat(transport);
    engine.activeSessionId = "sess_300";

    engine.sendMessage("Prompt yang panjang");
    expect(engine.isGenerating).toBe(true);

    engine.stop();
    expect(engine.isGenerating).toBe(false);
  });

  it("should configure HttpSseTransport with custom baseUrl and embed token", () => {
    const transport = new HttpSseTransport({
      baseUrl: "https://lumis.internal:8443/",
      embedToken: "lumis_embed_v1.token.signature",
    });

    // Access private resolveUrl via cast
    const resolved = (transport as any).resolveUrl("/api/stream");
    expect(resolved).toBe("https://lumis.internal:8443/api/stream");
  });
});
