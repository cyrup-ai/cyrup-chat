//! Core types and global state for context menu system

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::environment::platform::AppWindow;

pub type MenuEventId = u32;

pub type ContextMenuEventIdMapHandler =
    Arc<RwLock<HashMap<String, Arc<dyn Fn(MenuEventId) + Send + Sync>>>>;
pub type ContextMenuEventIdMapAction =
    Arc<RwLock<HashMap<String, HashMap<MenuEventId, Box<dyn Any + Send + Sync>>>>>;

lazy_static::lazy_static! {
    /// Action to handler
    pub static ref CTX_STATE_A: ContextMenuEventIdMapHandler = Arc::default();

    /// Action to MenuId to Action
    pub static ref CTX_STATE: ContextMenuEventIdMapAction = Arc::default();
}

pub trait ScopeExt {
    fn window(&self) -> &AppWindow;
}

pub type Payload = Box<dyn Any + Send + Sync>;
