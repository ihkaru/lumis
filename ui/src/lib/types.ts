export type MessageRole = "user" | "assistant" | "system" | "tool";

export type MessagePart =
  | {
      type: "text";
      text: string;
    }
  | {
      type: "tool_call";
      call_id?: string;
      tool: string;
      server?: string;
      args: any;
      result?: any;
      error?: string;
      isRunning?: boolean;
    };

export interface Message {
  id: string;
  session_id: string;
  role: MessageRole;
  content: string;
  model_id?: string;
  token_count?: number;
  metadata?: Record<string, unknown>;
  created_at: string;
}

export interface Session {
  id: string;
  user_id: string;
  title: string;
  model_id: string;
  system_prompt?: string;
  is_pinned: boolean;
  created_at: string;
  updated_at: string;
}

export interface CursorPage<T> {
  items: T[];
  next_cursor_time?: string;
  next_cursor_id?: string;
  has_more: boolean;
}

export interface UserMemory {
  id: string;
  user_id: string;
  category: string;
  content: string;
  created_at: string;
  updated_at: string;
}

export interface User {
  id: string;
  username: string;
  email?: string;
  role: "super_admin" | "tenant_admin" | "power_user" | "member" | "guest";
  tenant_id?: string;
}

export type ModelClearanceTier =
  | "tier_basic"
  | "tier_standard"
  | "tier_advanced"
  | "tier_reasoning";

export interface DriverHealth {
  dialect: "sqlite" | "postgres";
  is_alive: boolean;
  latency_ms: number;
  active_connections: number;
  max_connections: number;
}

export interface RoleHierarchyInfo {
  role: string;
  level: number;
  canonical_name: string;
  clearance_tier: ModelClearanceTier;
}

export interface BrandingConfig {
  app_title: string;
  logo_url: string | null;
  primary_color: string | null;
  version: string;
  upgrade_channel: string;
}

export interface ArtifactSpecification {
  identifier: string;
  artifact_type: string;
  title: string;
  content: string;
  version: number;
}

export interface ArtifactIpcResponse {
  request_id: string;
  success: boolean;
  data?: Record<string, unknown>;
  error?: string;
}

export interface ModelCapabilities {
  vision: boolean;
  tools: boolean;
  reasoning: boolean;
  context_length?: number;
  max_output?: number;
}

export interface ModelDescriptor {
  id: string;
  name: string;
  provider: string;
  owned_by?: string;
  tier: string;
  capabilities?: ModelCapabilities;
  is_default: boolean;
  can_access: boolean;
}

export interface KillswitchStatus {
  active: boolean;
  reason: string | null;
  activated_by: string | null;
}

export interface SiemAuditEvent {
  event_id?: string;
  id?: string;
  timestamp: string;
  tenant_id: string;
  actor_id: string;
  actor_role: string;
  action?: string;
  event_type?: string;
  status: string;
  resource_id?: string;
  details: Record<string, unknown>;
}

export interface PendingToolApproval {
  token: string;
  tool_name: string;
  caller_id: string;
  caller_role: string;
  risk_level: string;
  args: Record<string, unknown>;
  created_at: string;
}

export interface McpOAuthConfig {
  client_id: string;
  authorization_endpoint: string;
  token_endpoint: string;
  scopes: string[];
  resource_indicator?: string;
  registration_mode: string;
}

export interface McpServerItem {
  id: string;
  tenant_id: string;
  name: string;
  display_title: string;
  description?: string;
  transport_type: "sse" | "streamable_http" | "stdio";
  endpoint_url?: string;
  auth_type: "none" | "bearer" | "basic" | "oauth2" | "obo";
  is_enabled: boolean;
  risk_tier: number;
  scope_type: "system" | "user";
  is_mandatory: boolean;
  allowed_roles?: string[];
  oauth_config?: McpOAuthConfig | null;
  ssrf_policy?: "strict" | "allow_private_network" | "custom_allowlist";
  egress_allowlist?: string[];
  sandbox_type?: "none" | "restricted_process" | "docker_ephemeral";
  max_memory_mb?: number;
  timeout_seconds?: number;
  dlp_enabled?: boolean;
  dlp_rules?: string[];
  rate_limit_rpm?: number;
  daily_request_limit?: number;
  kms_provider?: "local" | "vault" | "aws_kms";
  kms_key_id?: string;
  created_at: string;
  updated_at: string;
}

export interface McpUserPreferenceItem {
  id: string;
  name: string;
  display_title: string;
  description: string | null;
  transport_type: string;
  auth_type: string;
  scope_type: "system" | "user";
  is_mandatory: boolean;
  is_enabled: boolean;
  is_oauth_connected: boolean;
  tools_count: number;
  rate_limit_rpm?: number;
  daily_request_limit?: number;
  quota_used_today?: number;
}

export interface McpToolItem {
  id: string;
  server_id: string;
  name: string;
  description?: string;
  input_schema_json: string;
  risk_tier: number;
  requires_approval: boolean;
  updated_at: string;
}

export type TaskStatus = "pending" | "in_progress" | "completed" | "cancelled";

export interface SessionTask {
  id: string;
  session_id: string;
  parent_task_id: string | null;
  title: string;
  description: string | null;
  status: TaskStatus;
  order_index: number;
  result_summary: string | null;
  created_at: string;
  updated_at: string;
}




