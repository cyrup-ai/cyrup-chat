//! Chat input with @mention autocomplete for multi-agent rooms
//!
//! Features:
//! - Detects @ trigger in real-time
//! - Filters agents by name (fuzzy match)
//! - Dropdown UI with agent selection
//! - Inserts @agent-name on selection
//! - Only shows agents in current room

use crate::environment::Environment;
use crate::view_model::agent::AgentTemplate;
use dioxus::prelude::*;
use surrealdb_types::ToSql;

#[component]
pub fn MentionInput(
    value: Signal<String>,
    on_submit: EventHandler<String>,
    disabled: bool,
    room_agents: Vec<String>,  // Agent template IDs in this room
) -> Element {
    let environment = use_context::<Environment>();
    let mut show_autocomplete = use_signal(|| false);
    let mut autocomplete_options = use_signal(Vec::<AgentTemplate>::new);
    let mut autocomplete_filter = use_signal(String::new);

    // Load all agent templates once (filtered by room_agents)
    let templates = use_resource(move || {
        let database = environment.database.clone();
        async move {
            log::debug!("[MentionInput] Loading agent templates");
            database.list_agent_templates().await
        }
    });

    let handle_input = move |e: Event<FormData>| {
        let text = e.value();
        value.set(text.clone());

        // Detect @ mention trigger
        // Find last word boundary (space or @)
        if let Some(last_word_start) = text.rfind(|c: char| c.is_whitespace() || c == '@') {
            let last_word = &text[last_word_start..];
            
            if let Some(filter) = last_word.strip_prefix('@') {
                // Extract filter text after @
                autocomplete_filter.set(filter.to_string());

                log::debug!("[MentionInput] Autocomplete filter: '{}'", filter);

                // Filter agents in this room
                if let Some(Ok(all_templates)) = templates.read().as_ref() {
                    let filtered: Vec<AgentTemplate> = all_templates
                        .iter()
                        .filter(|t| room_agents.contains(&t.id.0.to_sql()))  // Only room participants
                        .filter(|t| {
                            // Fuzzy match: name or ID contains filter
                            let filter_lower = filter.to_lowercase();
                            t.name.to_lowercase().contains(&filter_lower)
                                || t.id.0.to_sql().to_lowercase().contains(&filter_lower)
                        })
                        .cloned()
                        .collect();

                    log::debug!("[MentionInput] Found {} matching agents", filtered.len());
                    autocomplete_options.set(filtered.clone());
                    show_autocomplete.set(!filtered.is_empty());
                } else {
                    show_autocomplete.set(false);
                }
            } else {
                show_autocomplete.set(false);
            }
        } else {
            show_autocomplete.set(false);
        }
    };

    let mut select_agent = move |agent_name: String| {
        let mut text = value.read().clone();
        
        // Find last @ and replace everything after it
        if let Some(mention_start) = text.rfind('@') {
            text.truncate(mention_start + 1);  // Keep @ symbol
            text.push_str(&agent_name);
            text.push(' ');  // Add trailing space
            value.set(text);
            
            log::info!("[MentionInput] Inserted @mention: {}", agent_name);
        }
        
        show_autocomplete.set(false);
    };

    let submit = move |_| {
        let text = value.read().clone();
        if !text.trim().is_empty() {
            log::info!("[MentionInput] Submitting message with mentions");
            on_submit.call(text);
            value.set(String::new());
            show_autocomplete.set(false);
        }
    };

    let options = autocomplete_options.read().clone();
    
    rsx! {
        div {
            class: "relative",

            // Autocomplete dropdown (appears above input)
            if *show_autocomplete.read() {
                div {
                    class: "absolute bottom-full mb-2 w-full bg-[#1a1a2e] border border-white/10 rounded-lg shadow-lg max-h-48 overflow-y-auto",
                    {options.iter().map(|agent| {
                        let agent_name = agent.name.clone();
                        let agent_id = agent.id.0.to_sql();

                        rsx! {
                            div {
                                key: "{agent_id}",
                                class: "px-4 py-2 hover:bg-white/10 cursor-pointer transition-colors",
                                onclick: move |_| select_agent(agent_name.clone()),

                                div {
                                    class: "font-semibold text-white",
                                    "@{agent_name}"
                                }
                                div {
                                    class: "text-xs text-gray-400",
                                    "{agent_id}"
                                }
                            }
                        }
                    })}
                }
            }

            // Input form
            form {
                class: "flex gap-2",
                onsubmit: submit,
                
                input {
                    class: "flex-1 px-4 py-3 bg-white/5 border border-white/10 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-blue-500",
                    r#type: "text",
                    placeholder: "Type @ to mention agents...",
                    value: "{value.read()}",
                    oninput: handle_input,
                    disabled: disabled,
                }
                
                button {
                    class: "px-6 py-3 bg-[#00a8ff] text-white rounded-lg hover:bg-[#0098e6] transition-colors disabled:opacity-50 disabled:cursor-not-allowed",
                    r#type: "submit",
                    disabled: disabled,
                    "Send"
                }
            }
        }
    }
}
