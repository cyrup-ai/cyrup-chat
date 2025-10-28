//! Menu building and rendering to native muda components

use super::structures::{ContextMenuItem, ContextMenuKind};
use super::types::{MenuEventId, Payload};
use std::collections::HashMap;

impl ContextMenuItem {
    pub fn build(self, into: &mut muda::Submenu, actions: &mut HashMap<MenuEventId, Payload>) {
        use muda::{CheckMenuItem, MenuItem, PredefinedMenuItem, Submenu};
        match self.kind {
            ContextMenuKind::Checkbox {
                title,
                checked,
                enabled,
                payload,
            } => {
                let item = CheckMenuItem::new(title, enabled, checked, None);
                // Convert MenuId to u32 using hash for consistent mapping
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                item.id().hash(&mut hasher);
                let id_as_u32 = (hasher.finish() & 0xFFFF_FFFF) as u32;
                actions.insert(id_as_u32, payload);
                let _ = into.append(&item);
            }
            ContextMenuKind::Item {
                title,
                enabled,
                payload,
            } => {
                let item = MenuItem::new(title, enabled, None);
                // Convert MenuId to u32 using hash for consistent mapping
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                item.id().hash(&mut hasher);
                let id_as_u32 = (hasher.finish() & 0xFFFF_FFFF) as u32;
                actions.insert(id_as_u32, payload);
                let _ = into.append(&item);
            }
            ContextMenuKind::Submenu {
                title,
                enabled,
                children,
            } => {
                let mut sub_menu = Submenu::new(title, enabled);
                for child in children {
                    child.build(&mut sub_menu, actions);
                }
                let _ = into.append(&sub_menu);
            }
            ContextMenuKind::Separator => {
                let _ = into.append(&PredefinedMenuItem::separator());
            }
        }
    }
}
