//! Notifications view component wrapper

use crate::environment::Environment;
use dioxus::prelude::*;

#[component]
pub fn NotificationsView() -> Element {
    use crate::components::sidebar::{SidebarAction, SidebarState, handle_action};
    use crate::components::sidebar::view::notifications::SidebarNotificationsComponent;
    
    let environment = use_context::<Environment>();
    
    // Initialize sidebar state for notifications
    let sidebar_signal = use_signal(SidebarState::default);
    
    // Load notifications on mount
    use_effect(move || {
        let env = environment.clone();
        handle_action(sidebar_signal, SidebarAction::LoadNotifications, &env);
    });
    
    rsx! {
        div {
            class: "flex-1 overflow-y-auto",
            h2 {
                class: "text-2xl font-bold text-[var(--g-labelColor)] p-4 border-b border-white/10 sticky top-0 bg-gradient-to-r from-[#1a1a2e]/95 to-[#16213e]/95 backdrop-blur-md",
                "Notifications"
            }
            div {
                class: "p-4",
                SidebarNotificationsComponent { store: sidebar_signal }
            }
        }
    }
}
