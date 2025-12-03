use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio;

/// Current settings version - increment this when adding new models or changing settings structure
/// This triggers automatic migration for existing users
const SETTINGS_VERSION: u32 = 2;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings_version: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_openai_compatible: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProjectSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<HashMap<String, serde_json::Value>>,
}

pub struct SettingsManager {
    user_settings_path: PathBuf,
    project_settings_path: PathBuf,
}

impl SettingsManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
        let user_settings_path = home_dir.join(".grok").join("user-settings.json");

        let project_settings_path = std::env::current_dir()?.join(".grok").join("settings.json");

        // Create .grok directory in home if it doesn't exist
        if let Some(parent) = user_settings_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(SettingsManager {
            user_settings_path,
            project_settings_path,
        })
    }

    pub async fn load_user_settings(&self) -> Result<UserSettings, Box<dyn std::error::Error>> {
        if self.user_settings_path.exists() {
            let content = tokio::fs::read_to_string(&self.user_settings_path).await?;
            let mut settings: UserSettings = serde_json::from_str(&content)?;

            // Check if migration is needed
            let current_version = settings.settings_version.unwrap_or(1);
            if current_version < SETTINGS_VERSION {
                settings = self.migrate_settings(settings, current_version).await?;
                self.save_user_settings(&settings).await?;
            }

            Ok(settings)
        } else {
            // Create default settings
            let default_settings = self.create_default_user_settings();
            self.save_user_settings(&default_settings).await?;
            Ok(default_settings)
        }
    }

    pub async fn migrate_settings(&self, mut settings: UserSettings, from_version: u32) -> Result<UserSettings, Box<dyn std::error::Error>> {
        let mut migrated = settings;

        // Migration from version 1 to 2: Add new Grok 4.1 and Grok 4 Fast models
        if from_version < 2 {
            let default_models = self.get_default_models();
            let existing_models: std::collections::HashSet<String> =
                migrated.models.clone().unwrap_or_default().into_iter().collect();

            // Add any new models that don't exist in user's current list
            let new_models: Vec<String> = default_models
                .iter()
                .filter(|model| !existing_models.contains(*model))
                .cloned()
                .collect();

            // Prepend new models to the list (newest models first)
            let mut updated_models = new_models;
            updated_models.extend(migrated.models.unwrap_or_default());

            migrated.models = Some(updated_models);
        }

        migrated.settings_version = Some(SETTINGS_VERSION);
        Ok(migrated)
    }

    fn create_default_user_settings(&self) -> UserSettings {
        UserSettings {
            api_key: None,
            base_url: Some("https://api.x.ai/v1".to_string()),
            default_model: Some("grok-code-fast-1".to_string()),
            models: Some(self.get_default_models()),
            settings_version: Some(SETTINGS_VERSION),
            is_openai_compatible: Some(false),
        }
    }

    fn get_default_models(&self) -> Vec<String> {
        vec![
            // Grok 4.1 Fast models (2M context, latest - November 2025)
            "grok-4-1-fast-reasoning".to_string(),
            "grok-4-1-fast-non-reasoning".to_string(),
            // Grok 4 Fast models (2M context)
            "grok-4-fast-reasoning".to_string(),
            "grok-4-fast-non-reasoning".to_string(),
            // Grok 4 flagship (256K context)
            "grok-4".to_string(),
            "grok-4-latest".to_string(),
            // Grok Code (optimized for coding, 256K context)
            "grok-code-fast-1".to_string(),
            // Grok 3 models (131K context)
            "grok-3".to_string(),
            "grok-3-latest".to_string(),
            "grok-3-fast".to_string(),
            "grok-3-mini".to_string(),
            "grok-3-mini-fast".to_string(),
        ]
    }

    pub async fn save_user_settings(&self, settings: &UserSettings) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(settings)?;
        tokio::fs::write(&self.user_settings_path, content).await?;
        Ok(())
    }

    pub async fn load_project_settings(&self) -> Result<ProjectSettings, Box<dyn std::error::Error>> {
        if self.project_settings_path.exists() {
            let content = tokio::fs::read_to_string(&self.project_settings_path).await?;
            let settings: ProjectSettings = serde_json::from_str(&content)?;
            Ok(settings)
        } else {
            // Create default project settings if file doesn't exist
            self.save_project_settings(&ProjectSettings {
                model: Some("grok-code-fast-1".to_string()),
                mcp_servers: None,
            }).await?;
            Ok(ProjectSettings {
                model: Some("grok-code-fast-1".to_string()),
                mcp_servers: None,
            })
        }
    }

    pub async fn save_project_settings(&self, settings: &ProjectSettings) -> Result<(), Box<dyn std::error::Error>> {
        // Create .grok directory if it doesn't exist
        if let Some(parent) = self.project_settings_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(settings)?;
        tokio::fs::write(&self.project_settings_path, content).await?;
        Ok(())
    }

    pub async fn get_api_key(&self) -> Option<String> {
        // First check environment variable
        if let Ok(api_key) = std::env::var("GROK_API_KEY") {
            return Some(api_key);
        }

        // Then check user settings
        match self.load_user_settings().await {
            Ok(settings) => settings.api_key,
            Err(_) => None,
        }
    }

    pub async fn get_base_url(&self) -> String {
        // First check environment variable
        if let Ok(base_url) = std::env::var("GROK_BASE_URL") {
            return base_url;
        }

        // Then check user settings
        match self.load_user_settings().await {
            Ok(settings) => {
                settings.base_url.unwrap_or_else(|| "https://api.x.ai/v1".to_string())
            }
            Err(_) => "https://api.x.ai/v1".to_string(),
        }
    }

    pub async fn get_current_model(&self) -> String {
        // First check project-specific model setting
        if let Ok(project_settings) = self.load_project_settings().await {
            if let Some(model) = project_settings.model {
                return model;
            }
        }

        // Then check user's default model
        if let Ok(user_settings) = self.load_user_settings().await {
            if let Some(model) = user_settings.default_model {
                return model;
            }
        }

        // Fallback to system default
        "grok-code-fast-1".to_string()
    }

    pub async fn set_current_model(&self, model: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Update project settings with the new model
        let mut project_settings = self.load_project_settings().await.unwrap_or(ProjectSettings {
            model: None,
            mcp_servers: None,
        });

        project_settings.model = Some(model.to_string());
        self.save_project_settings(&project_settings).await?;

        Ok(())
    }

    pub async fn get_available_models(&self) -> Vec<String> {
        match self.load_user_settings().await {
            Ok(settings) => settings.models.unwrap_or(self.get_default_models()),
            Err(_) => self.get_default_models(),
        }
    }

    pub async fn update_user_setting<K>(&self, key: &str, value: K) -> Result<(), Box<dyn std::error::Error>>
    where
        K: serde::Serialize,
    {
        let mut settings = self.load_user_settings().await.unwrap_or_else(|_| self.create_default_user_settings());

        match key {
            "apiKey" => {
                if let Ok(api_key_val) = serde_json::to_value(value) {
                    settings.api_key = api_key_val.as_str().map(|s| s.to_string());
                }
            },
            "baseURL" => {
                if let Ok(base_url_val) = serde_json::to_value(value) {
                    settings.base_url = base_url_val.as_str().map(|s| s.to_string());
                }
            },
            "defaultModel" => {
                if let Ok(default_model_val) = serde_json::to_value(value) {
                    settings.default_model = default_model_val.as_str().map(|s| s.to_string());
                }
            },
            _ => {}
        }

        self.save_user_settings(&settings).await?;
        Ok(())
    }
}

pub async fn get_settings_manager() -> Result<SettingsManager, Box<dyn std::error::Error>> {
    SettingsManager::new()
}

pub async fn get_api_key() -> Option<String> {
    if let Ok(manager) = get_settings_manager().await {
        manager.get_api_key().await
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_settings_manager_creation() -> Result<(), Box<dyn std::error::Error>> {
        let manager = SettingsManager::new()?;
        assert!(manager.user_settings_path.ends_with("user-settings.json"));
        Ok(())
    }
}