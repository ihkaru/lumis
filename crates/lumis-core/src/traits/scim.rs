#![deny(unsafe_code)]

use async_trait::async_trait;

use crate::error::Result;
use crate::models::scim::{ScimGroup, ScimUser};

#[async_trait]
pub trait ScimProvisioner: Send + Sync {
    // Users
    async fn list_users(&self) -> Result<Vec<ScimUser>>;
    async fn get_user(&self, id: &str) -> Result<Option<ScimUser>>;
    async fn create_user(&self, user: ScimUser) -> Result<ScimUser>;
    async fn update_user(&self, id: &str, user: ScimUser) -> Result<ScimUser>;
    async fn update_user_status(&self, id: &str, active: bool) -> Result<bool>;
    async fn delete_user(&self, id: &str) -> Result<bool>;

    // Groups
    async fn list_groups(&self) -> Result<Vec<ScimGroup>>;
    async fn get_group(&self, id: &str) -> Result<Option<ScimGroup>>;
    async fn create_group(&self, group: ScimGroup) -> Result<ScimGroup>;
    async fn update_group(&self, id: &str, group: ScimGroup) -> Result<ScimGroup>;
    async fn delete_group(&self, id: &str) -> Result<bool>;
}
