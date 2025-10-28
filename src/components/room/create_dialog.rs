//! Room creation dialog with multi-agent selection
//!
//! Features:
//! - Text input for room title
//! - Multi-select agent picker (checkbox-style)
//! - Async room creation with database.create_room()
//! - Error handling with user-friendly messages
//! - Loading states during creation

use crate::environment::Environment;
use crate::view_model::agent::AgentTemplateId;
use crate::view_model::conversation::{Room, RoomId};
use dioxus::prelude::*;

#[component]
pub fn RoomCreateDialog(on_close: EventHandler<()>) -> Element {
    let environment = use_context::<Environment>();
    let database_for_templates = environment.database.clone();
    let database_for_creation = environment.database.clone();
    let mut room_title = use_signal(|| String::new());
    let mut selected_agents = use_signal(|| Vec::<String>::new());
    let mut is_creating = use_signal(|| false);
    let mut create_error = use_signal(|| Option::<String>::None);

    // Load available agent templates from database
    let templates = use_resource(move || {
        let database = database_for_templates.clone();
        async move {
            log::debug!("[RoomCreate] Loading agent templates");
            database.list_agent_templates().await
        }
    });

    let create_room = move |_| {
        // Validation
        if room_title.read().trim().is_empty() {
            create_error.set(Some("Room title is required".to_string()));
            return;
        }

        if selected_agents.read().is_empty() {
            create_error.set(Some("Select at least one agent".to_string()));
            return;
        }

        is_creating.set(true);
        create_error.set(None);

        spawn({
            let database = database_for_creation.clone();
            let title = room_title.read().clone();
            let agents = selected_agents.read().clone();
            let mut is_creating = is_creating;
            let mut create_error = create_error;
            let on_close = on_close.clone();

            async move {
                log::info!("[RoomCreate] Creating room '{}' with {} agents", title, agents.len());
                
                let room = Room {
                    id: RoomId(format!("room:{}", uuid::Uuid::new_v4())),
                    title,
                    participants: agents.into_iter().map(AgentTemplateId).collect(),
                    summary: String::new(),
                    last_summarized_message_id: None,
                    last_message_at: chrono::Utc::now(),
                    created_at: chrono::Utc::now(),
                };

                match database.create_room(&room).await {
                    Ok(room_id) => {
                        log::info!("[RoomCreate] Successfully created room: {}", room_id);
                        on_close.call(());
                    }
                    Err(e) => {
                        log::error!("[RoomCreate] Failed to create room: {}", e);
                        create_error.set(Some(format!("Failed to create room: {}", e)));
                    }
                }

                is_creating.set(false);
            }
        });
    };

    let mut toggle_agent = move |agent_id: String| {
        let mut agents = selected_agents.write();
        if agents.contains(&agent_id) {
            agents.retain(|id| id != &agent_id);
            log::debug!("[RoomCreate] Deselected agent: {}", agent_id);
        } else {
            agents.push(agent_id.clone());
            log::debug!("[RoomCreate] Selected agent: {}", agent_id);
        }
    };

    rsx! {
        // Modal overlay
        div {
            class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
            onclick: move |_| on_close.call(()),
            
            // Dialog content
            div {
                class: "bg-[#1a1a2e] rounded-lg p-6 w-[500px] max-h-[80vh] overflow-y-auto",
                onclick: move |e| e.stop_propagation(),

                h2 {
                    class: "text-xl font-bold text-white mb-4",
                    "Create Multi-Agent Room"
                }

                // Room title input
                div {
                    class: "mb-4",
                    label {
                        class: "block text-sm text-gray-400 mb-2",
                        "Room Title"
                    }
                    input {
                        class: "w-full px-3 py-2 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-blue-500",
                        r#type: "text",
                        placeholder: "e.g., Code Review Team",
                        value: "{room_title.read()}",
                        oninput: move |e| room_title.set(e.value()),
                        autofocus: true,
                    }
                }

                // Agent selection (multi-select checkboxes)
                div {
                    class: "mb-4",
                    label {
                        class: "block text-sm text-gray-400 mb-2",
                        "Select Agents"
                    }
                    
                    match &*templates.read() {
                        Some(Ok(template_list)) => rsx! {
                            div {
                                class: "space-y-2 max-h-[300px] overflow-y-auto",
                                {template_list.iter().map(|template| {
                                    let template_id = template.id.0.clone();
                                    let is_selected = selected_agents.read().contains(&template_id);
                                    let template_id_for_click = template_id.clone();
                                    
                                    rsx! {
                                        div {
                                            key: "{template_id}",
                                            class: if is_selected {
                                                "p-3 bg-blue-500/20 border-2 border-blue-500 rounded cursor-pointer transition-colors"
                                            } else {
                                                "p-3 bg-white/5 border border-white/10 rounded cursor-pointer hover:bg-white/10 transition-colors"
                                            },
                                            onclick: move |_| toggle_agent(template_id_for_click.clone()),
                                            
                                            div {
                                                class: "flex items-center gap-2",
                                                
                                                // Checkbox indicator
                                                div {
                                                    class: if is_selected {
                                                        "w-5 h-5 bg-blue-500 rounded flex items-center justify-center"
                                                    } else {
                                                        "w-5 h-5 bg-white/5 border border-white/20 rounded"
                                                    },
                                                    if is_selected {
                                                        span { class: "text-white text-sm", "✓" }
                                                    }
                                                }
                                                
                                                // Agent name
                                                span {
                                                    class: "font-semibold text-white",
                                                    "{template.name}"
                                                }
                                            }
                                            
                                            // Agent model info
                                            div {
                                                class: "text-xs text-gray-400 mt-1 ml-7",
                                                "Model: {template.model}"
                                            }
                                        }
                                    }
                                })}
                            }
                        },
                        Some(Err(e)) => rsx! {
                            div { 
                                class: "text-red-400 p-3 bg-red-500/10 border border-red-500/20 rounded", 
                                "Error loading agents: {e}" 
                            }
                        },
                        None => rsx! {
                            div { 
                                class: "text-gray-400 p-3 text-center", 
                                "Loading agents..." 
                            }
                        }
                    }
                }

                // Error display
                if let Some(error) = create_error.read().as_ref() {
                    div {
                        class: "mb-4 p-3 bg-red-500/20 border border-red-500 rounded text-red-300",
                        "{error}"
                    }
                }

                // Action buttons
                div {
                    class: "flex gap-3",
                    button {
                        class: "flex-1 px-4 py-2 bg-white/5 border border-white/10 rounded text-white hover:bg-white/10 transition-colors disabled:opacity-50 disabled:cursor-not-allowed",
                        onclick: move |_| on_close.call(()),
                        disabled: *is_creating.read(),
                        "Cancel"
                    }
                    button {
                        class: "flex-1 px-4 py-2 bg-blue-500 rounded text-white hover:bg-blue-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed",
                        onclick: create_room,
                        disabled: *is_creating.read(),
                        if *is_creating.read() { 
                            "Creating..." 
                        } else { 
                            "Create Room" 
                        }
                    }
                }
            }
        }
    }
}
