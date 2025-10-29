//! Timeline view component wrapper

use crate::auth::AuthState;
use crate::environment::Environment;
use dioxus::prelude::*;

#[component]
pub fn TimelineView(auth_state: AuthState) -> Element {
    use crate::components::status_timeline::{
        Action, State as TimelineState, TimelineComponent, handle_action,
    };
    use crate::components::status_timeline::{AnyTimelineProvider, ConversationListProvider};
    use crate::environment::types::UiConfig;
    use crate::view_model::AccountViewModel;
    
    let environment = use_context::<Environment>();
    
    // Create account view model from auth state
    let account = AccountViewModel {
        id: crate::view_model::AccountId(auth_state.user.id.clone()),
        image: auth_state.user.picture.clone(),
        image_header: String::new(),
        username: auth_state.user.username.clone().unwrap_or_else(|| auth_state.user.name.clone()),
        display_name: auth_state.user.name.clone(),
        display_name_html: auth_state.user.name.clone(),
        acct: auth_state.user.username.clone().unwrap_or_else(|| auth_state.user.name.clone()),
        note_plain: String::new(),
        note_html: Vec::new(),
        joined_human: String::new(),
        joined_full: String::new(),
        joined: chrono::Utc::now(),
        url: String::new(),
        followers: 0,
        followers_str: "0".to_string(),
        following: 0,
        following_str: "0".to_string(),
        statuses: 0,
        statuses_str: "0".to_string(),
        header: String::new(),
        fields: Vec::new(),
        locked: false,
        bot: false,
    };
    
    // Create conversation list provider
    let provider = AnyTimelineProvider::new(
        ConversationListProvider::new(environment.clone()),
        &account.id
    );
    
    // Initialize timeline state
    let timeline_signal = use_signal(|| {
        TimelineState::new(
            provider,
            UiConfig::default(),
            Some(account)
        )
    });
    
    // Load initial data
    use_effect(move || {
        let mut env = environment.clone();
        handle_action(timeline_signal, Action::Initial, &mut env);
        handle_action(timeline_signal, Action::LoadData, &mut env);
    });
    
    rsx! {
        div {
            class: "flex-1 overflow-y-auto",
            h2 {
                class: "text-2xl font-bold text-[var(--g-labelColor)] p-4 border-b border-white/10 sticky top-0 bg-gradient-to-r from-[#1a1a2e]/95 to-[#16213e]/95 backdrop-blur-md",
                "Timeline"
            }
            TimelineComponent { store: timeline_signal }
        }
    }
}
