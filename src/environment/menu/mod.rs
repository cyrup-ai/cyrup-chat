//! Context menu system for cross-platform native menu support

pub mod builder;
pub mod display;
pub mod events;
pub mod interface;
pub mod structures;
pub mod types;

// Re-export main public API
pub use events::{resolve_current_action, setup_menu_handler};
pub use interface::{ViewStoreContextMenu, context_menu};
pub use structures::{ContextMenu, ContextMenuItem, ContextMenuKind};
pub use types::{MenuEventId, ScopeExt};
