#![deny(unsafe_code)]

pub mod db_provisioner;
pub use db_provisioner::DatabaseScimProvisioner;

use std::collections::HashMap;
use std::sync::RwLock;
use async_trait::async_trait;
use lumis_core::error::{LumisError, Result};
use lumis_core::models::scim::{ScimGroup, ScimUser};
use lumis_core::traits::ScimProvisioner;
use uuid::Uuid;

pub struct InMemoryScimProvisioner {
    users: RwLock<HashMap<String, ScimUser>>,
    groups: RwLock<HashMap<String, ScimGroup>>,
}

impl InMemoryScimProvisioner {
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
            groups: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryScimProvisioner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ScimProvisioner for InMemoryScimProvisioner {
    async fn list_users(&self) -> Result<Vec<ScimUser>> {
        let users = self.users.read().unwrap();
        Ok(users.values().cloned().collect())
    }

    async fn get_user(&self, id: &str) -> Result<Option<ScimUser>> {
        let users = self.users.read().unwrap();
        Ok(users.get(id).cloned())
    }

    async fn create_user(&self, mut user: ScimUser) -> Result<ScimUser> {
        let mut users = self.users.write().unwrap();
        if user.id.is_empty() {
            user.id = format!("usr-{}", Uuid::new_v4());
        }
        users.insert(user.id.clone(), user.clone());
        Ok(user)
    }

    async fn update_user(&self, id: &str, user: ScimUser) -> Result<ScimUser> {
        let mut users = self.users.write().unwrap();
        if users.contains_key(id) {
            users.insert(id.to_string(), user.clone());
            Ok(user)
        } else {
            Err(LumisError::NotFound(format!("SCIM user '{}' not found", id)))
        }
    }

    async fn update_user_status(&self, id: &str, active: bool) -> Result<bool> {
        let mut users = self.users.write().unwrap();
        if let Some(user) = users.get_mut(id) {
            user.active = active;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn delete_user(&self, id: &str) -> Result<bool> {
        let mut users = self.users.write().unwrap();
        Ok(users.remove(id).is_some())
    }

    async fn list_groups(&self) -> Result<Vec<ScimGroup>> {
        let groups = self.groups.read().unwrap();
        Ok(groups.values().cloned().collect())
    }

    async fn get_group(&self, id: &str) -> Result<Option<ScimGroup>> {
        let groups = self.groups.read().unwrap();
        Ok(groups.get(id).cloned())
    }

    async fn create_group(&self, mut group: ScimGroup) -> Result<ScimGroup> {
        let mut groups = self.groups.write().unwrap();
        if group.id.is_empty() {
            group.id = format!("grp-{}", Uuid::new_v4());
        }
        groups.insert(group.id.clone(), group.clone());
        Ok(group)
    }

    async fn update_group(&self, id: &str, group: ScimGroup) -> Result<ScimGroup> {
        let mut groups = self.groups.write().unwrap();
        if groups.contains_key(id) {
            groups.insert(id.to_string(), group.clone());
            Ok(group)
        } else {
            Err(LumisError::NotFound(format!("SCIM group '{}' not found", id)))
        }
    }

    async fn delete_group(&self, id: &str) -> Result<bool> {
        let mut groups = self.groups.write().unwrap();
        Ok(groups.remove(id).is_some())
    }
}
