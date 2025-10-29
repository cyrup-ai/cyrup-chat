//! More view component wrapper

use crate::auth::AuthState;
use crate::environment::Environment;
use dioxus::prelude::*;

#[component]
pub fn MoreView(auth_state: AuthState) -> Element {
    use crate::components::more::{Action, State as MoreState, MoreSignal, MoreViewComponent, handle_action};
    use crate::components::sidebar::MoreSelection;
    use crate::environment::model::Account;
    
    let environment = use_context::<Environment>();
    
    // Convert AuthState to Account (wraps megalodon::entities::Account)
    let megalodon_account = megalodon::entities::Account {
        id: auth_state.user.id.clone(),
        username: auth_state.user.username.clone().unwrap_or_else(|| auth_state.user.name.clone()),
        acct: auth_state.user.username.clone().unwrap_or_else(|| auth_state.user.name.clone()),
        display_name: auth_state.user.name.clone(),
        locked: false,
        discoverable: Some(true),
        group: None,
        noindex: None,
        moved: None,
        suspended: None,
        limited: None,
        created_at: chrono::Utc::now(),
        followers_count: 0,
        following_count: 0,
        statuses_count: 0,
        note: String::new(),
        url: String::new(),
        avatar: auth_state.user.picture.clone(),
        avatar_static: auth_state.user.picture.clone(),
        header: String::new(),
        header_static: String::new(),
        emojis: Vec::new(),
        fields: Vec::new(),
        bot: false,
        source: None,
        role: None,
        mute_expires_at: None,
    };
    
    let account = Account::new(megalodon_account, None);
    
    // Initialize more state with Classic selection as default
    let more_signal: MoreSignal = use_signal(|| {
        MoreState::new(MoreSelection::Classic, account.clone())
    });
    
    // Initialize providers on mount
    use_effect(move || {
        let env = environment.clone();
        handle_action(more_signal, Action::Initial, &env);
    });
    
    rsx! {
        div {
            class: "flex-1 overflow-y-auto",
            h2 {
                class: "text-2xl font-bold text-[var(--g-labelColor)] p-4 border-b border-white/10 sticky top-0 bg-gradient-to-r from-[#1a1a2e]/95 to-[#16213e]/95 backdrop-blur-md",
                "More Options"
            }
            MoreViewComponent { store: more_signal }
        }
    }
}
