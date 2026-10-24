import type {
  ArtifactSpecification,
  CursorPage,
  Message,
  ModelDescriptor,
  Session,
  SessionTask,
  User,
} from "../types";

export interface TransportConfig {
  baseUrl?: string;
  getAuthToken?: () => Promise<string | null> | string | null;
  embedToken?: string;
  headers?: Record<string, string>;
}

export interface StreamEvent {
  event: string;
  session_id?: string;
  delta?: string;
  model_id?: string;
  is_done?: boolean;
  message?: string;
  tool?: string;
  args?: Record<string, unknown>;
  tool_call_id?: string;
  result?: unknown;
  error?: string;
  from_model?: string;
  to_model?: string;
  reason?: string;
  tasks?: SessionTask[];
}

export interface ILumisTransport {
  fetchApi<T>(path: string, options?: RequestInit): Promise<T>;
  getMe(): Promise<{ authenticated: boolean; user?: User; role: string }>;
  getModels(): Promise<ModelDescriptor[]>;
  listSessions(cursorTime?: string, cursorId?: string, limit?: number): Promise<CursorPage<Session>>;
  createSession(title: string, modelId?: string): Promise<Session>;
  listMessages(sessionId: string, cursorTime?: string, cursorId?: string, limit?: number): Promise<CursorPage<Message>>;
  listSessionTasks(sessionId: string): Promise<SessionTask[]>;
  sendMessage(sessionId: string, content: string, modelId?: string, metadata?: Record<string, unknown>): Promise<{ status: string }>;
  connectStream(onToken: (data: string) => void): EventSource | (() => void);
  parseArtifacts(text: string): Promise<ArtifactSpecification[]>;
}

export interface FrontendToolDefinition {
  name: string;
  description: string;
  parameters: Record<string, unknown>;
  execute: (args: Record<string, unknown>) => Promise<unknown> | unknown;
}

export interface LumisEventEnvelope<T = unknown> {
  source: "lumis-widget" | "host-app";
  event: string;
  correlation_id?: string;
  timestamp: number;
  payload?: T;
}

export interface ChatEngineOptions {
  initialSessionId?: string | null;
  defaultModelId?: string;
  contextData?: Record<string, unknown>;
  onMessageReceived?: (message: Message) => void;
  onToolCallRequested?: (toolName: string, args: Record<string, unknown>) => Promise<unknown>;
}

export interface LumisWidgetConfig {
  serverUrl?: string;
  embedToken?: string;
  getAuthToken?: () => Promise<string | null> | string | null;
  title?: string;
  position?: "bottom-right" | "bottom-left";
  theme?: "dark" | "light" | "auto";
  defaultOpen?: boolean;
  contextData?: Record<string, unknown>;
}
