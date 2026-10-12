#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const SCIM_USER_SCHEMA: &str = "urn:ietf:params:scim:schemas:core:2.0:User";
pub const SCIM_GROUP_SCHEMA: &str = "urn:ietf:params:scim:schemas:core:2.0:Group";
pub const SCIM_LIST_SCHEMA: &str = "urn:ietf:params:scim:api:messages:2.0:ListResponse";
pub const SCIM_ERROR_SCHEMA: &str = "urn:ietf:params:scim:api:messages:2.0:Error";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScimEmail {
    pub value: String,
    #[serde(default)]
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScimUser {
    #[serde(default = "default_user_schemas")]
    pub schemas: Vec<String>,
    pub id: String,
    pub user_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default)]
    pub emails: Vec<ScimEmail>,
    #[serde(default = "default_active")]
    pub active: bool,
    #[serde(default)]
    pub roles: Vec<String>,
}

fn default_user_schemas() -> Vec<String> {
    vec![SCIM_USER_SCHEMA.to_string()]
}

fn default_active() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScimMemberRef {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScimGroup {
    #[serde(default = "default_group_schemas")]
    pub schemas: Vec<String>,
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub members: Vec<ScimMemberRef>,
}

fn default_group_schemas() -> Vec<String> {
    vec![SCIM_GROUP_SCHEMA.to_string()]
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScimListResponse<T> {
    pub schemas: Vec<String>,
    pub total_results: usize,
    pub items_per_page: usize,
    pub start_index: usize,
    #[serde(rename = "Resources")]
    pub resources: Vec<T>,
}

impl<T> ScimListResponse<T> {
    pub fn new(resources: Vec<T>) -> Self {
        let total = resources.len();
        Self {
            schemas: vec![SCIM_LIST_SCHEMA.to_string()],
            total_results: total,
            items_per_page: total,
            start_index: 1,
            resources,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScimErrorResponse {
    pub schemas: Vec<String>,
    pub status: String,
    pub detail: String,
}

impl ScimErrorResponse {
    pub fn new(status: u16, detail: impl Into<String>) -> Self {
        Self {
            schemas: vec![SCIM_ERROR_SCHEMA.to_string()],
            status: status.to_string(),
            detail: detail.into(),
        }
    }
}
