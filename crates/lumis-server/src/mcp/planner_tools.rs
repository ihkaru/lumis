#![deny(unsafe_code)]

use std::str::FromStr;
use std::sync::Arc;
use async_trait::async_trait;
use bytes::Bytes;
use lumis_core::error::Result;
use lumis_core::models::{
    CreateTaskInput, EnterpriseRole, McpToolCallResult, McpToolDefinition, TaskStatus,
    UpdateTaskInput,
};
use lumis_core::traits::{ChatRepository, McpRegistry, McpToolExecutor};
use serde_json::json;
use tokio::sync::broadcast;

/// Tool: plan_task_create
pub struct PlanTaskCreateTool {
    repo: Arc<dyn ChatRepository>,
    stream_tx: broadcast::Sender<Bytes>,
}

impl PlanTaskCreateTool {
    pub fn new(repo: Arc<dyn ChatRepository>, stream_tx: broadcast::Sender<Bytes>) -> Self {
        Self { repo, stream_tx }
    }
}

#[async_trait]
impl McpToolExecutor for PlanTaskCreateTool {
    fn name(&self) -> &str {
        "plan_task_create"
    }

    fn definition(&self) -> McpToolDefinition {
        McpToolDefinition {
            name: "plan_task_create".to_string(),
            description: "Adds a task or subtask to the dynamic plan. Long-running goals converge when all tasks reach completion.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Current conversation session ID" },
                    "title": { "type": "string", "description": "Concise summary of the task" },
                    "description": { "type": "string", "description": "Acceptance criteria, rationale, or steps" },
                    "parent_task_id": { "type": "string", "description": "Optional parent task ID for subtasks" },
                    "order_index": { "type": "integer", "description": "Optional ordering sequence position" }
                },
                "required": ["session_id", "title"]
            }),
            min_role: EnterpriseRole::Member,
        }
    }

    async fn execute(&self, _caller_role: EnterpriseRole, arguments: &serde_json::Value) -> Result<McpToolCallResult> {
        let session_id = arguments.get("session_id").and_then(|v| v.as_str()).unwrap_or_default();
        let title = arguments.get("title").and_then(|v| v.as_str()).unwrap_or_default();
        if session_id.is_empty() || title.is_empty() {
            return Ok(McpToolCallResult {
                content: "Error: session_id and title are required".to_string(),
                is_error: true,
            });
        }

        let input = CreateTaskInput {
            id: None,
            session_id: session_id.to_string(),
            parent_task_id: arguments.get("parent_task_id").and_then(|v| v.as_str()).map(|s| s.to_string()),
            title: title.to_string(),
            description: arguments.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
            status: TaskStatus::Pending,
            order_index: arguments.get("order_index").and_then(|v| v.as_i64()).map(|n| n as i32),
        };

        match self.repo.create_session_task(&input).await {
            Ok(task) => {
                let evt = json!({
                    "event": "task_created",
                    "session_id": session_id,
                    "task": task,
                });
                let _ = self.stream_tx.send(Bytes::from(evt.to_string()));
                Ok(McpToolCallResult {
                    content: serde_json::to_string(&task).unwrap_or_default(),
                    is_error: false,
                })
            }
            Err(e) => Ok(McpToolCallResult {
                content: format!("Error creating task: {}", e),
                is_error: true,
            }),
        }
    }
}

/// Tool: plan_task_update
pub struct PlanTaskUpdateTool {
    repo: Arc<dyn ChatRepository>,
    stream_tx: broadcast::Sender<Bytes>,
}

impl PlanTaskUpdateTool {
    pub fn new(repo: Arc<dyn ChatRepository>, stream_tx: broadcast::Sender<Bytes>) -> Self {
        Self { repo, stream_tx }
    }
}

#[async_trait]
impl McpToolExecutor for PlanTaskUpdateTool {
    fn name(&self) -> &str {
        "plan_task_update"
    }

    fn definition(&self) -> McpToolDefinition {
        McpToolDefinition {
            name: "plan_task_update".to_string(),
            description: "Updates status (pending, in_progress, completed, cancelled) or details of an existing task.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": { "type": "string", "description": "The unique ID of the task to update" },
                    "status": { "type": "string", "enum": ["pending", "in_progress", "completed", "cancelled"], "description": "New task status" },
                    "result_summary": { "type": "string", "description": "Empirical evidence, test result, or output summary" },
                    "title": { "type": "string", "description": "Updated title if refined" },
                    "description": { "type": "string", "description": "Updated description if refined" },
                    "order_index": { "type": "integer", "description": "Updated sequence position" }
                },
                "required": ["task_id"]
            }),
            min_role: EnterpriseRole::Member,
        }
    }

    async fn execute(&self, _caller_role: EnterpriseRole, arguments: &serde_json::Value) -> Result<McpToolCallResult> {
        let task_id = arguments.get("task_id").and_then(|v| v.as_str()).unwrap_or_default();
        if task_id.is_empty() {
            return Ok(McpToolCallResult {
                content: "Error: task_id is required".to_string(),
                is_error: true,
            });
        }

        let status = arguments.get("status").and_then(|v| v.as_str()).and_then(|s| TaskStatus::from_str(s).ok());
        let update = UpdateTaskInput {
            title: arguments.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()),
            description: arguments.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
            status,
            order_index: arguments.get("order_index").and_then(|v| v.as_i64()).map(|n| n as i32),
            result_summary: arguments.get("result_summary").and_then(|v| v.as_str()).map(|s| s.to_string()),
        };

        match self.repo.update_session_task(task_id, &update).await {
            Ok(Some(task)) => {
                let evt = json!({
                    "event": "task_updated",
                    "session_id": task.session_id,
                    "task": task,
                });
                let _ = self.stream_tx.send(Bytes::from(evt.to_string()));
                Ok(McpToolCallResult {
                    content: serde_json::to_string(&task).unwrap_or_default(),
                    is_error: false,
                })
            }
            Ok(None) => Ok(McpToolCallResult {
                content: format!("Error: Task '{}' not found", task_id),
                is_error: true,
            }),
            Err(e) => Ok(McpToolCallResult {
                content: format!("Error updating task: {}", e),
                is_error: true,
            }),
        }
    }
}

/// Tool: plan_task_list
pub struct PlanTaskListTool {
    repo: Arc<dyn ChatRepository>,
}

impl PlanTaskListTool {
    pub fn new(repo: Arc<dyn ChatRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl McpToolExecutor for PlanTaskListTool {
    fn name(&self) -> &str {
        "plan_task_list"
    }

    fn definition(&self) -> McpToolDefinition {
        McpToolDefinition {
            name: "plan_task_list".to_string(),
            description: "Retrieves the current dynamic task plan state machine and completion progress for a session.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Current conversation session ID" }
                },
                "required": ["session_id"]
            }),
            min_role: EnterpriseRole::Member,
        }
    }

    async fn execute(&self, _caller_role: EnterpriseRole, arguments: &serde_json::Value) -> Result<McpToolCallResult> {
        let session_id = arguments.get("session_id").and_then(|v| v.as_str()).unwrap_or_default();
        if session_id.is_empty() {
            return Ok(McpToolCallResult {
                content: "Error: session_id is required".to_string(),
                is_error: true,
            });
        }

        match self.repo.list_session_tasks(session_id).await {
            Ok(tasks) => Ok(McpToolCallResult {
                content: serde_json::to_string(&tasks).unwrap_or_default(),
                is_error: false,
            }),
            Err(e) => Ok(McpToolCallResult {
                content: format!("Error listing tasks: {}", e),
                is_error: true,
            }),
        }
    }
}

/// Tool: plan_task_delete
pub struct PlanTaskDeleteTool {
    repo: Arc<dyn ChatRepository>,
    stream_tx: broadcast::Sender<Bytes>,
}

impl PlanTaskDeleteTool {
    pub fn new(repo: Arc<dyn ChatRepository>, stream_tx: broadcast::Sender<Bytes>) -> Self {
        Self { repo, stream_tx }
    }
}

#[async_trait]
impl McpToolExecutor for PlanTaskDeleteTool {
    fn name(&self) -> &str {
        "plan_task_delete"
    }

    fn definition(&self) -> McpToolDefinition {
        McpToolDefinition {
            name: "plan_task_delete".to_string(),
            description: "Prunes an obsolete or cancelled task from the dynamic plan.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Current conversation session ID" },
                    "task_id": { "type": "string", "description": "The unique ID of the task to delete" }
                },
                "required": ["session_id", "task_id"]
            }),
            min_role: EnterpriseRole::Member,
        }
    }

    async fn execute(&self, _caller_role: EnterpriseRole, arguments: &serde_json::Value) -> Result<McpToolCallResult> {
        let session_id = arguments.get("session_id").and_then(|v| v.as_str()).unwrap_or_default();
        let task_id = arguments.get("task_id").and_then(|v| v.as_str()).unwrap_or_default();
        if task_id.is_empty() {
            return Ok(McpToolCallResult {
                content: "Error: task_id is required".to_string(),
                is_error: true,
            });
        }

        match self.repo.delete_session_task(task_id).await {
            Ok(deleted) => {
                if deleted {
                    let evt = json!({
                        "event": "task_deleted",
                        "session_id": session_id,
                        "task_id": task_id,
                    });
                    let _ = self.stream_tx.send(Bytes::from(evt.to_string()));
                }
                Ok(McpToolCallResult {
                    content: json!({ "deleted": deleted, "task_id": task_id }).to_string(),
                    is_error: false,
                })
            }
            Err(e) => Ok(McpToolCallResult {
                content: format!("Error deleting task: {}", e),
                is_error: true,
            }),
        }
    }
}

/// Helper function to register all dynamic task planner tools into MCP registry.
pub async fn register_planner_tools(
    registry: &dyn McpRegistry,
    repo: Arc<dyn ChatRepository>,
    stream_tx: broadcast::Sender<Bytes>,
) -> Result<()> {
    registry.register_tool(Arc::new(PlanTaskCreateTool::new(repo.clone(), stream_tx.clone()))).await?;
    registry.register_tool(Arc::new(PlanTaskUpdateTool::new(repo.clone(), stream_tx.clone()))).await?;
    registry.register_tool(Arc::new(PlanTaskListTool::new(repo.clone()))).await?;
    registry.register_tool(Arc::new(PlanTaskDeleteTool::new(repo, stream_tx))).await?;
    Ok(())
}
