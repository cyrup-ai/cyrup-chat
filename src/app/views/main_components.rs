//! Main application UI components with action dispatching

use crate::app::reducer::AppAction;
use crate::auth::AuthState;
use crate::components::chat::ChatComponent;
use crate::environment::Environment;
use chrono::{DateTime, Local, Utc};
use dioxus::prelude::*;

#[component]
pub fn MainView(auth_state: AuthState) -> Element {
    rsx! {
        div {
            class: "flex h-screen bg-gradient-to-br from-[#1a1a2e]/80 via-[#16213e]/80 to-[#0f0f1e]/80 bg-[length:400%_400%] animate-[gradientShift_45s_ease-in-out_infinite] relative",
            div {
                ChatHistorySidebar { auth_state: auth_state.clone() }
            }
            div {
                class: "flex-1",
                ChatComponent {}
            }
        }
    }
}

#[component]
pub fn ChatHistorySidebar(auth_state: AuthState) -> Element {
    let dispatch = use_context::<Callback<AppAction>>();
    let environment = use_context::<Environment>();
    
    // Get or create selected conversation ID context
    let mut selected_conversation_id = match try_use_context::<Signal<String>>() {
        Some(id) => id,
        None => {
            // Context not provided, create default
            let id = use_signal(|| "conversation:default_chat".to_string());
            use_context_provider(|| id);
            id
        }
    };

    // Clone environment for use in different closures
    let environment_for_button = environment.clone();

    // Load conversations from database
    let conversations = use_resource(move || {
        let database = environment.database.clone();
        async move { database.list_conversations().await }
    });

    let handle_logout = move |_| {
        dispatch(AppAction::LogoutRequested);
    };

    rsx! {
        div {
            class: "w-[280px] bg-gradient-to-b from-[#1a1a2e]/90 to-[#0f0f1e]/90 glass border-r border-white/10 flex flex-col shadow-lg",

            // Header
            div {
                class: "p-6 border-b border-white/5",
                h3 { class: "m-0 text-[var(--g-labelColor)] text-[1.1em] font-semibold", "Chat History" }
            }

            // User profile
            div {
                class: "p-5 border-b border-white/5 flex items-center gap-4",
                img {
                    src: if !auth_state.user.picture.is_empty() {
                        auth_state.user.picture.as_str()
                    } else {
                        "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='40' height='40'%3E%3Ccircle cx='20' cy='20' r='20' fill='%2364748b'/%3E%3Ctext x='50%25' y='50%25' text-anchor='middle' dy='.3em' fill='white' font-size='16' font-family='sans-serif'%3EU%3C/text%3E%3C/svg%3E"
                    },
                    alt: "Profile",
                    class: "w-12 h-12 rounded-full object-cover ring-2 ring-white/10"
                }
                div {
                    class: "flex-1 overflow-hidden space-y-1",
                    div { class: "font-semibold text-[var(--g-labelColor)] whitespace-nowrap overflow-hidden text-ellipsis", "{auth_state.user.name}" }
                    div { class: "text-[0.85em] text-[var(--g-secondaryLabelColor)] whitespace-nowrap overflow-hidden text-ellipsis", "{auth_state.user.email}" }
                }
                button {
                    class: "px-3 py-1 bg-white/5 border border-white/10 rounded text-white/70 text-[0.85em] cursor-pointer transition-all duration-200 hover:bg-white/10 hover:border-white/20 hover:text-white",
                    onclick: handle_logout,
                    "Logout"
                }
            }

            // New conversation button
            div {
                class: "p-4 border-b border-white/5",
                button {
                    class: "w-full px-4 py-2 bg-[#00a8ff]/20 border border-[#00a8ff]/50 rounded-lg text-white font-semibold cursor-pointer transition-all duration-200 hover:bg-[#00a8ff]/30 hover:border-[#00a8ff] hover:-translate-y-px active:translate-y-0",
                    onclick: move |_| {
                        let env = environment_for_button.clone();
                        let selected_id = selected_conversation_id;
                        spawn(async move {
                            if let Err(e) = create_new_conversation(env, selected_id).await {
                                log::error!("[ChatHistory] Failed to create conversation: {}", e);
                            }
                        });
                    },
                    "+ New Conversation"
                }
            }

            // Conversation list
            div {
                class: "flex-1 overflow-y-auto p-4",
                match &*conversations.read() {
                    Some(Ok(convos)) => rsx! {
                        if convos.is_empty() {
                            div {
                                class: "p-8 text-center text-white/50 text-sm",
                                "No conversations yet"
                            }
                        } else {
                            {convos.iter().map(|convo| {
                                let convo_id = convo.id.0.clone();
                                let is_selected = selected_conversation_id.read().as_str() == convo_id.as_str();
                                let convo_id_for_click = convo_id.clone();
                                let convo_key = convo_id.clone();
                                let timestamp_str = format_timestamp(&convo.last_message_timestamp);
                                let title_str = convo.title.clone();
                                let unread = convo.unread_count;
                                
                                rsx! {
                                    div {
                                        key: "{convo_key}",
                                        class: if is_selected {
                                            "px-4 py-3 mb-2 rounded-lg cursor-pointer bg-white/10 border border-white/20 transition-all duration-200"
                                        } else {
                                            "px-4 py-3 mb-2 rounded-lg cursor-pointer transition-all duration-200 hover:bg-white/8 hover:border-white/15"
                                        },
                                        onclick: move |_| {
                                            log::debug!("[ChatHistory] Selected conversation: {}", convo_id_for_click);
                                            selected_conversation_id.set(convo_id_for_click.clone());
                                        },
                                        div {
                                            class: "text-[0.75em] text-[var(--g-secondaryLabelColor)] mb-1",
                                            "{timestamp_str}"
                                        }
                                        div {
                                            class: "text-[0.9em] text-[var(--g-labelColor)] whitespace-nowrap overflow-hidden text-ellipsis",
                                            "{title_str}"
                                        }
                                        if unread > 0 {
                                            span {
                                                class: "inline-block ml-2 px-2 py-0.5 bg-red-500 text-white text-xs rounded-full font-bold",
                                                "{unread}"
                                            }
                                        }
                                    }
                                }
                            })}
                        }
                    },
                    Some(Err(e)) => rsx! {
                        div { class: "p-4 text-red-400 text-sm", "Error: {e}" }
                    },
                    None => rsx! {
                        div { class: "p-8 text-center text-white/50 text-sm", "Loading..." }
                    }
                }
            }
        }
    }
}

/// Helper function for date formatting
fn format_timestamp(timestamp: &DateTime<Utc>) -> String {
    let local_time = timestamp.with_timezone(&Local);
    let now = Local::now();

    if local_time.date_naive() == now.date_naive() {
        "Today".to_string()
    } else if local_time.date_naive() == (now - chrono::Duration::days(1)).date_naive() {
        "Yesterday".to_string()
    } else {
        local_time.format("%B %d").to_string()
    }
}

/// Create a new conversation and select it
async fn create_new_conversation(
    environment: Environment,
    mut selected_conversation_id: Signal<String>,
) -> Result<(), String> {
    use crate::view_model::agent::AgentTemplateId;
    use crate::view_model::conversation::{Conversation, ConversationId};
    
    let db = environment.database;

    // Get available templates
    let templates = db.list_templates().await?;

    // For MVP: Use first template or default ID
    let template_id = templates
        .first()
        .map(|t| t.id.0.clone())
        .unwrap_or_else(|| "template:default".to_string());

    // Generate new conversation ID
    let new_id = format!("conversation:{}", uuid::Uuid::new_v4());

    // Create conversation
    let now = chrono::Utc::now();
    let conversation = Conversation {
        id: ConversationId(new_id.clone()),
        title: "New Conversation".to_string(),
        template_id: AgentTemplateId(template_id),
        summary: String::new(),
        agent_session_id: None,
        last_summarized_message_id: None,
        last_message_at: now,
        created_at: now,
    };

    db.create_conversation(&conversation).await?;
    log::info!("[ChatHistory] Created new conversation: {}", new_id);

    // Auto-select newly created conversation
    selected_conversation_id.set(new_id);

    Ok(())
}
