//! Menu event handling, registration, and action resolution

use super::types::{CTX_STATE, CTX_STATE_A, MenuEventId};
use std::collections::HashMap;
use std::sync::Arc;

pub fn setup_menu_handler<A>(
    id: usize,
    schedule_update: Option<Arc<dyn Fn(MenuEventId) + Send + Sync>>,
) {
    let action_key = format!("{}-{}", id, std::any::type_name::<A>());
    // If the `HashMap` is still empty, set up the event handler, otherwise
    // insert into the hashmap
    let Some(mut m) = CTX_STATE.write().ok() else {
        log::error!("CRITICAL: Could not get a write handle into the menu context map");
        log::error!("This indicates a severe concurrency issue with the menu system");
        // Return early rather than crashing - menu registration will fail
        // but the application can continue without this specific menu functionality
        return;
    };

    if m.is_empty() {
        use muda::MenuEvent;
        MenuEvent::set_event_handler(Some(move |event: muda::MenuEvent| {
            // iterate over all actions and call them. only those with a matching
            // event will trigger. this is a bit expensive, but only happens on
            // menu events.
            let Ok(r) = CTX_STATE_A.read() else {
                return;
            };
            for (_, v) in r.iter() {
                // Convert MenuId to u32 using hash for consistent mapping
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                event.id.hash(&mut hasher);
                let id_as_u32 = (hasher.finish() & 0xFFFF_FFFF) as u32;
                (v)(id_as_u32)
            }
        }));
    }

    // register
    m.insert(action_key.clone(), HashMap::new());
    let Ok(mut actions) = CTX_STATE_A.write() else {
        log::error!("CRITICAL: Could not get a write handle into the menu action map");
        log::error!("This indicates a severe concurrency issue with the menu system");
        // Return early rather than crashing - menu registration will be incomplete
        // but the application can continue without this specific menu action
        return;
    };
    if let Some(n) = schedule_update {
        actions.insert(action_key, n);
    } else {
        actions.remove(&action_key);
    }
}

pub fn resolve_current_action<Action: std::fmt::Debug + Clone + 'static>(
    id: usize,
    menu_event_id: MenuEventId,
) -> Option<Action> {
    let action_key = format!("{}-{}", id, std::any::type_name::<Action>());
    let mut s = CTX_STATE.write().ok()?;
    let m = s.get_mut(&action_key)?;
    let mx = m.remove(&menu_event_id)?;

    let Ok(value) = mx.downcast::<Action>() else {
        return None;
    };
    Some(*value)
}
