//! Main application UI components with action dispatching

use crate::app::reducer::AppAction;
use crate::auth::AuthState;
use crate::components::chat::ChatComponent;
use crate::environment::Environment;
use chrono::{DateTime, Local, Utc};
use dioxus::prelude::*;
use surrealdb_types::{RecordId, ToSql};

#[derive(Clone, Copy, PartialEq, Debug)]
enum ViewMode {
    Chat,
    Bookmarks,
    Timeline,
    Notifications,
    Rooms,
    More,
    Templates,
}

#[component]
pub fn MainView(auth_state: AuthState) -> Element {
    let mut view_mode = use_signal(|| ViewMode::Chat);
    use_context_provider(|| view_mode);
    
    let environment = use_context::<Environment>();
    let dispatch = use_context::<Callback<AppAction>>();
    
    // Register menu event handler using channel for thread safety
    let (menu_sender, menu_receiver) = use_signal(flume::unbounded::<crate::environment::types::AppEvent>)();
    
    // Set up menu event handler with thread-safe channel
    let env_for_menu = environment.clone();
    use_effect(move || {
        use std::sync::Arc;
        let sender_clone = menu_sender.clone();
        env_for_menu.platform.handle_menu_events(Arc::new(move |event| {
            let _ = sender_clone.send(event);
        }));
    });
    
    // Process menu events from channel
    use_future(move || {
        let receiver = menu_receiver.clone();
        async move {
            while let Ok(event) = receiver.recv_async().await {
                use crate::environment::types::{AppEvent, MainMenuEvent};
                
                if let AppEvent::MenuEvent(menu_event) = event {
                    log::debug!("[MainView] Received menu event: {:?}", menu_event);
                    
                    // Map menu events to view mode changes
                    match menu_event {
                        MainMenuEvent::Timeline => {
                            view_mode.set(ViewMode::Timeline);
                        }
                        MainMenuEvent::Mentions => {
                            view_mode.set(ViewMode::Notifications);
                        }
                        MainMenuEvent::Messages => {
                            view_mode.set(ViewMode::Rooms);
                        }
                        MainMenuEvent::More => {
                            view_mode.set(ViewMode::More);
                        }
                        MainMenuEvent::Logout => {
                            dispatch(AppAction::LogoutRequested);
                        }
                        MainMenuEvent::NewPost => {
                            log::warn!("[MainView] NewPost not yet implemented");
                        }
                        MainMenuEvent::Reload => {
                            log::debug!("[MainView] Reload requested for current view");
                        }
                        _ => {
                            log::debug!("[MainView] Unhandled menu event: {:?}", menu_event);
                        }
                    }
                }
            }
        }
    });
    
    // Update menu enabled state when logged in
    use_effect(move || {
        let env = environment.clone();
        env.platform.update_menu(|config| {
            config.logged_in = true; // MainView only renders when authenticated
        });
    });
    
    rsx! {
        div {
            class: "flex h-screen bg-gradient-to-br from-[#1a1a2e]/80 via-[#16213e]/80 to-[#0f0f1e]/80 bg-[length:400%_400%] animate-[gradientShift_45s_ease-in-out_infinite] relative",
            div {
                ChatHistorySidebar { auth_state: auth_state.clone() }
            }
            div {
                class: "flex-1",
                match *view_mode.read() {
                    ViewMode::Chat => rsx! { ChatComponent {} },
                    ViewMode::Bookmarks => rsx! { 
                        crate::components::bookmarks::BookmarksView {}
                    },
                    ViewMode::Timeline => rsx! {
                        crate::app::views::TimelineView { auth_state: auth_state.clone() }
                    },
                    ViewMode::Notifications => rsx! {
                        crate::app::views::NotificationsView {}
                    },
                    ViewMode::Rooms => rsx! {
                        crate::app::views::RoomsView { auth_state: auth_state.clone() }
                    },
                    ViewMode::More => rsx! {
                        crate::app::views::MoreView { auth_state: auth_state.clone() }
                    },
                    ViewMode::Templates => rsx! {
                        crate::components::template_manager::TemplateManagerComponent {}
                    },
                }
            }
        }
    }
}

#[component]
pub fn ChatHistorySidebar(auth_state: AuthState) -> Element {
    let dispatch = use_context::<Callback<AppAction>>();
    let environment = use_context::<Environment>();
    
    // Get or create selected conversation ID context
    let mut selected_conversation_id = match try_use_context::<Signal<RecordId>>() {
        Some(id) => id,
        None => {
            // Context not provided, create default
            let id = use_signal(|| RecordId::new("conversation", "default_chat"));
            use_context_provider(|| id);
            id
        }
    };

    // Clone environment for use in different closures
    let environment_for_button = environment.clone();

    // Load recent conversations from database (limit=10 for sidebar)
    // Full conversation list available in Timeline view
    let conversations = use_resource(move || {
        let database = environment.database.clone();
        async move { database.list_recent_conversations(10).await }
    });

    // Calculate total unread count across all conversations
    let total_unread = use_memo(move || {
        if let Some(Ok(convos)) = conversations.read().as_ref() {
            convos.iter().map(|c| c.unread_count).sum::<u32>()
        } else {
            0
        }
    });

    // Update window title when unread count changes
    use_effect(move || {
        let count = *total_unread.read();
        
        spawn(async move {
            let title = if count > 0 {
                format!("({}) CYRUP Chat", count)
            } else {
                "CYRUP Chat".to_string()
            };
            
            let window = dioxus::desktop::window();
            window.set_title(&title);
            log::debug!("[App] Updated window title: {}", title);
        });
    });

    let handle_logout = move |_| {
        dispatch(AppAction::LogoutRequested);
    };

    rsx! {
        div {
            class: "w-[280px] pt-16 bg-gradient-to-b from-[#1a1a2e]/90 to-[#0f0f1e]/90 glass border-r border-white/10 flex flex-col shadow-lg",

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

            // Navigation section
            NavigationSection {}

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
                                let convo_id = convo.id.clone();
                                let is_selected = *selected_conversation_id.read() == convo_id;
                                let convo_id_for_click = convo_id.clone();
                                let convo_key = convo_id.to_sql();
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
                                            class: "text-[0.9em] text-[var(--g-labelColor)] whitespace-nowrap overflow-hidden text-ellipsis flex items-center justify-between",
                                            span {
                                                class: "flex-1",
                                                "{title_str}"
                                            }
                                            // Add agent count badge for multi-agent conversations
                                            if convo.participants.len() > 1 {
                                                span {
                                                    class: "text-xs text-white/50 ml-2 px-2 py-0.5 bg-white/10 rounded-full",
                                                    "{convo.participants.len()} agents"
                                                }
                                            }
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
    mut selected_conversation_id: Signal<RecordId>,
) -> Result<(), String> {
    use crate::view_model::conversation::Conversation;
    
    let db = environment.database;

    // Get available templates
    let templates = db.list_templates().await?;

    // For MVP: Use first template or default ID
    let template_id = templates
        .first()
        .map(|t| t.id.clone())
        .unwrap_or_else(|| RecordId::new("agent_template", "default"));

    // Generate new conversation ID
    let new_id_key = uuid::Uuid::new_v4().to_string();
    let new_id = RecordId::new("conversation", new_id_key.as_str());

    // Create conversation
    let now = chrono::Utc::now();
    let conversation = Conversation {
        id: new_id.clone(),
        title: "New Conversation".to_string(),
        participants: vec![template_id],
        summary: String::new(),
        agent_sessions: std::collections::HashMap::new(),
        last_summarized_message_id: None,
        last_message_at: now.into(),
        created_at: now.into(),
    };

    let created_id = db.create_conversation(&conversation).await?;
    log::info!("[ChatHistory] Created new conversation: {}", created_id.to_sql());

    // Auto-select newly created conversation
    selected_conversation_id.set(created_id);

    Ok(())
}

/// Navigation section component for switching between all views
#[component]
fn NavigationSection() -> Element {
    let environment = use_context::<Environment>();
    let mut view_mode = use_context::<Signal<ViewMode>>();
    
    // Load bookmark count
    let bookmark_count = use_resource(move || {
        let database = environment.database.clone();
        async move {
            let user_id = "hardcoded-david-maple";
            match database.get_bookmarked_messages(user_id).await {
                Ok(messages) => Some(messages.len()),
                Err(e) => {
                    log::error!("[Navigation] Failed to load bookmark count: {}", e);
                    None
                }
            }
        }
    });
    
    let current_view = *view_mode.read();
    
    // Helper function to create nav button
    let create_button = |label: &str, icon: &str, target: ViewMode, count: Option<usize>| {
        let is_active = current_view == target;
        rsx! {
            button {
                key: "{label}",
                class: if is_active {
                    "w-full flex items-center gap-3 px-4 py-3 rounded-lg cursor-pointer bg-white/10 border border-white/20 transition-all duration-200"
                } else {
                    "w-full flex items-center gap-3 px-4 py-3 rounded-lg cursor-pointer transition-all duration-200 hover:bg-white/8"
                },
                onclick: move |_| {
                    view_mode.set(target);
                },
                div {
                    class: "text-base",
                    dangerous_inner_html: icon
                }
                span {
                    class: "text-sm font-medium text-[var(--g-labelColor)]",
                    "{label}"
                }
                if let Some(num) = count {
                    if num > 0 {
                        span {
                            class: "ml-auto px-2 py-0.5 rounded-full bg-white/10 text-xs text-[var(--g-labelColor)]",
                            "{num}"
                        }
                    }
                }
            }
        }
    };
    
    let bookmark_count_val = bookmark_count.read().as_ref().and_then(|c| *c);
    
    rsx! {
        div {
            class: "p-4 border-b border-white/5 space-y-2",
            
            {create_button("Conversations", "💬", ViewMode::Chat, None)}
            {create_button("Timeline", crate::icons::ICON_HOME, ViewMode::Timeline, None)}
            {create_button("Notifications", crate::icons::ICON_BELL, ViewMode::Notifications, None)}
            {create_button("Rooms", crate::icons::ICON_ROOMS, ViewMode::Rooms, None)}
            {create_button("Bookmarks", crate::icons::ICON_BOOKMARK2, ViewMode::Bookmarks, bookmark_count_val)}
            {create_button("More", crate::icons::ICON_MORE, ViewMode::More, None)}
            {create_button("Templates", crate::icons::ICON_OPTIONS, ViewMode::Templates, None)}
        }
    }
}
