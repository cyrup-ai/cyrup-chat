//! Model container utility for login state management

use crate::environment::model::Model;

#[derive(Debug, Clone)]
#[allow(dead_code)] // Model container - pending login integration
pub struct ModelContainer {
    pub id: String,
    pub model: Model,
}

impl ModelContainer {
    #[allow(dead_code)] // Model container constructor - pending login integration
    pub fn new(id: String, model: Model) -> Self {
        Self { id, model }
    }

    #[allow(dead_code)] // Model cloning utility - pending login integration
    pub fn cloned(&self) -> Model {
        self.model.clone()
    }

    #[allow(dead_code)] // Model URL extraction - MVP: hardcoded single instance
    pub fn url(&self) -> String {
        // Q9: MVP hardcoded auth - no URL needed
        String::new()
    }
}

impl PartialEq for ModelContainer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ModelContainer {}
