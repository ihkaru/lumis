#![deny(unsafe_code)]

use std::sync::Arc;
use async_trait::async_trait;
use chrono::Utc;
use lumis_core::error::Result;
use lumis_core::models::scim::{ScimGroup, ScimUser};
use lumis_core::models::{EnterpriseRole, User, UserIdentity};
use lumis_core::traits::{ChatRepository, ScimProvisioner};
use uuid::Uuid;

pub struct DatabaseScimProvisioner {
    repo: Arc<dyn ChatRepository>,
}

impl DatabaseScimProvisioner {
    pub fn new(repo: Arc<dyn ChatRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl ScimProvisioner for DatabaseScimProvisioner {
    async fn list_users(&self) -> Result<Vec<ScimUser>> {
        self.repo.list_scim_users().await
    }

    async fn get_user(&self, id: &str) -> Result<Option<ScimUser>> {
        self.repo.get_scim_user(id).await
    }

    async fn create_user(&self, mut user: ScimUser) -> Result<ScimUser> {
        if user.id.is_empty() {
            user.id = format!("usr-{}", Uuid::new_v4());
        }

        let created = self.repo.upsert_scim_user(&user).await?;

        // Synchronize internal User
        let email = user.emails.first().map(|e| e.value.clone());
        let now = Utc::now();
        let internal_user = User {
            id: user.id.clone(),
            username: user.user_name.clone(),
            email,
            role: EnterpriseRole::Member,
            tenant_id: None,
            preferences: serde_json::json!({}),
            is_active: user.active,
            password_hash: None,
            created_at: now,
            updated_at: now,
        };
        let _ = self.repo.upsert_user(&internal_user).await;

        // Link federated identity
        let identity = UserIdentity {
            id: Uuid::new_v4().to_string(),
            user_id: user.id.clone(),
            provider: "scim".to_string(),
            provider_subject_id: user.id.clone(),
            credential_data: Some(serde_json::json!({
                "user_name": user.user_name,
                "display_name": user.display_name,
            })),
            last_login_at: None,
            created_at: now,
            updated_at: now,
        };
        let _ = self.repo.link_identity(&identity).await;

        Ok(created)
    }

    async fn update_user(&self, id: &str, mut user: ScimUser) -> Result<ScimUser> {
        user.id = id.to_string();
        let updated = self.repo.upsert_scim_user(&user).await?;

        if let Ok(Some(mut u)) = self.repo.get_user_by_id(id).await {
            u.is_active = user.active;
            u.updated_at = Utc::now();
            let _ = self.repo.upsert_user(&u).await;
        }

        Ok(updated)
    }

    async fn update_user_status(&self, id: &str, active: bool) -> Result<bool> {
        let scim_res = self.repo.update_scim_user_status(id, active).await?;
        if scim_res {
            if let Ok(Some(mut u)) = self.repo.get_user_by_id(id).await {
                u.is_active = active;
                u.updated_at = Utc::now();
                let _ = self.repo.upsert_user(&u).await;
            }
        }
        Ok(scim_res)
    }

    async fn delete_user(&self, id: &str) -> Result<bool> {
        let deleted = self.repo.delete_scim_user(id).await?;
        if deleted {
            let _ = self.repo.unlink_identity(id, "scim").await;
            if let Ok(Some(mut u)) = self.repo.get_user_by_id(id).await {
                u.is_active = false;
                u.updated_at = Utc::now();
                let _ = self.repo.upsert_user(&u).await;
            }
        }
        Ok(deleted)
    }

    async fn list_groups(&self) -> Result<Vec<ScimGroup>> {
        self.repo.list_scim_groups().await
    }

    async fn get_group(&self, id: &str) -> Result<Option<ScimGroup>> {
        self.repo.get_scim_group(id).await
    }

    async fn create_group(&self, mut group: ScimGroup) -> Result<ScimGroup> {
        if group.id.is_empty() {
            group.id = format!("grp-{}", Uuid::new_v4());
        }
        self.repo.upsert_scim_group(&group).await
    }

    async fn update_group(&self, id: &str, mut group: ScimGroup) -> Result<ScimGroup> {
        group.id = id.to_string();
        self.repo.upsert_scim_group(&group).await
    }

    async fn delete_group(&self, id: &str) -> Result<bool> {
        self.repo.delete_scim_group(id).await
    }
}
