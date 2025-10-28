//! User settings and preferences module
//!
//! Manages UI configuration, timeline markers, and favorites
//! using JSON file persistence (similar to legacy Repository)

mod file_io;

use crate::environment::types::{Marker, UiConfig, User};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard};

// Use local file I/O helpers
use file_io::{read, write};

const MARKERS_PATH: &str = "markers.json";
const UICONFIG_PATH: &str = "uiconfig.json";
const FAVORITES_PATH: &str = "favorites.json";
const USERS_PATH: &str = "users.json";

type UserId = String;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
struct MarkersData {
    timeline_markers: HashMap<UserId, Marker>,
}

/// Settings manager for user preferences
#[derive(Clone)]
pub struct Settings {
    markers: Arc<Mutex<MarkersData>>,
    ui_config: Arc<Mutex<UiConfig>>,
    favorites: Arc<Mutex<HashSet<String>>>,
    users: Arc<Mutex<Vec<User>>>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            markers: Arc::new(Mutex::new(MarkersData::default())),
            ui_config: Arc::new(Mutex::new(UiConfig::default())),
            favorites: Arc::new(Mutex::new(HashSet::new())),
            users: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Settings {
    /// Create Settings with loaded data
    pub async fn new() -> Self {
        let markers = read(MARKERS_PATH).await.ok().flatten().unwrap_or_default();
        let ui_config = read(UICONFIG_PATH).await.ok().flatten().unwrap_or_default();
        let favorites = read(FAVORITES_PATH)
            .await
            .ok()
            .flatten()
            .unwrap_or_default();
        let users = read(USERS_PATH).await.ok().flatten().unwrap_or_default();

        Self {
            markers: Arc::new(Mutex::new(markers)),
            ui_config: Arc::new(Mutex::new(ui_config)),
            favorites: Arc::new(Mutex::new(favorites)),
            users: Arc::new(Mutex::new(users)),
        }
    }

    // Timeline marker methods
    pub fn get_timeline_marker(&self, account: &str) -> Option<(String, DateTime<Utc>)> {
        let markers = self.markers.lock().ok()?;
        markers
            .timeline_markers
            .get(account)
            .map(|m| (m.id.clone(), m.set))
    }

    pub async fn set_timeline_marker(&self, account: &str, status: &str) -> Option<()> {
        let markers_to_save = {
            let mut markers = self.markers.lock().ok()?;
            markers.timeline_markers.insert(
                account.to_string(),
                Marker {
                    set: chrono::Utc::now(),
                    id: status.to_string(),
                    marker_id: account.to_string(),
                },
            );
            markers.clone()
        };

        if let Err(e) = write(MARKERS_PATH, &markers_to_save).await {
            log::error!("Could not save markers: {e:?}");
        }
        Some(())
    }

    // UI config methods
    pub fn config(&self) -> Result<UiConfig, String> {
        Ok(self
            .ui_config
            .lock()
            .map_err(|e| format!("UiConfig lock error: {e:?}"))?
            .clone())
    }

    pub async fn set_config(&self, config: &UiConfig) -> Option<()> {
        {
            let mut ui_config = self.ui_config.lock().ok()?;
            *ui_config = config.clone();
        }

        if let Err(e) = write(UICONFIG_PATH, config).await {
            log::error!("Could not save config: {e:?}");
        }
        Some(())
    }

    pub async fn map_config<T>(
        &self,
        action: impl FnOnce(&mut MutexGuard<UiConfig>) -> T,
    ) -> Result<T, String> {
        let (result, updated_config) = {
            let mut ui_config = self
                .ui_config
                .lock()
                .map_err(|e| format!("UiConfig lock error: {e:?}"))?;
            let result = action(&mut ui_config);
            (result, ui_config.clone())
        };

        if let Err(e) = write(UICONFIG_PATH, &updated_config).await {
            log::error!("Could not save config: {e:?}");
        }

        Ok(result)
    }

    // Favorites methods
    pub fn favorites(&self) -> Option<HashSet<String>> {
        let favorites = self.favorites.lock().ok()?.clone();
        if favorites.is_empty() {
            None
        } else {
            Some(favorites)
        }
    }

    pub async fn toggle_favorite(&self, id: String) -> Result<(), String> {
        let updated_favorites = {
            let mut favs = self
                .favorites
                .lock()
                .map_err(|e| format!("Favorites lock error: {e:?}"))?;
            if favs.contains(&id) {
                favs.remove(&id);
            } else {
                favs.insert(id);
            }
            favs.clone()
        };

        if let Err(e) = write(FAVORITES_PATH, &updated_favorites).await {
            log::error!("Could not save favorites: {e:?}");
        }

        Ok(())
    }

    pub fn is_favorite(&self, id: &str) -> Result<bool, String> {
        Ok(self
            .favorites
            .lock()
            .map_err(|e| format!("Favorites lock error: {e:?}"))?
            .contains(id))
    }

    /// Access to favorites data for reactive UI updates
    pub fn favorites_data(&self) -> Option<HashSet<String>> {
        self.favorites()
    }

    // User management methods (account metadata only, NOT tokens)
    pub async fn update_or_insert_user(&self, new_user: User) -> Result<(), String> {
        let users_to_save = {
            let mut users = self
                .users
                .lock()
                .map_err(|e| format!("Users lock error: {e:?}"))?;
            let mut found = false;
            for user in users.iter_mut() {
                if user.id == new_user.id {
                    *user = new_user.clone();
                    found = true;
                    break;
                }
            }

            if !found {
                users.push(new_user);
            }

            users.clone()
        };

        if let Err(e) = write(USERS_PATH, &users_to_save).await {
            log::error!("Could not save users: {e:?}");
        }

        Ok(())
    }

    pub async fn remove_user(&self, id: String) -> Result<(), String> {
        let users_to_save = {
            let mut users = self
                .users
                .lock()
                .map_err(|e| format!("Users lock error: {e:?}"))?;
            let Some(user_index) = users.iter().position(|user| user.id == id) else {
                return Err(format!("Unknown User {id}"));
            };

            users.remove(user_index);
            users.clone()
        };

        if let Err(e) = write(USERS_PATH, &users_to_save).await {
            log::error!("Could not save users: {e:?}");
        }

        Ok(())
    }

    pub fn users(&self) -> Result<Vec<User>, String> {
        Ok(self
            .users
            .lock()
            .map_err(|e| format!("Users lock error: {e:?}"))?
            .clone())
    }
}
