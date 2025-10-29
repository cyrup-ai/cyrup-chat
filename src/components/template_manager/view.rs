//! Template manager view component
//!
//! Provides CRUD interface for agent templates with:
//! - Template list view
//! - Create new template form
//! - Edit existing template form
//! - Delete with confirmation
//!
//! Design Decisions:
//! - Q19: User chooses agent model (Sonnet/Haiku/Opus)
//! - Q20: System prompt is core configuration
//! - Q46: Templates have icon and color customization

use crate::app::context::use_environment;
use crate::view_model::agent::{AgentModel, AgentTemplate, AgentTemplateId};
use chrono::Utc;
use dioxus::prelude::*;
use surrealdb_types::ToSql;

/// Main template manager component
///
/// Shows list of templates with add/edit/delete actions.
/// Uses local signals for UI state and spawns async tasks for database operations.
///
/// # Component Structure
/// - Template list (from database)
/// - "New Template" button
/// - Conditional editor form (when creating or editing)
/// - Delete confirmation (future enhancement)
///
/// # State Management
/// - templates: Vec<AgentTemplate> - loaded from database
/// - editing_id: Option<String> - ID of template being edited (None = creating new)
/// - form_*: Individual form field signals
///
/// # Database Access
/// Uses env.read().model.database() pattern:
/// - Loads templates on mount with use_effect
/// - Creates/updates with spawn(async move {...})
/// - Deletes with spawn(async move {...})
///
/// # References
/// - Database API: [src/database/templates.rs](../../database/templates.rs)
/// - Data types: [src/view_model/agent.rs](../../view_model/agent.rs)
/// - Form pattern: [src/components/post/view/main.rs](../post/view/main.rs)
#[component]
pub fn TemplateManagerComponent() -> Element {
    // Load templates from database
    let mut templates = use_signal(Vec::<AgentTemplate>::new);

    // Editor state: None = not editing, Some(id) = editing template with id, Some("") = creating new
    let mut editing_id = use_signal(|| Option::<String>::None);

    // Form fields
    let mut form_name = use_signal(String::new);
    let mut form_system_prompt = use_signal(String::new);
    let mut form_model = use_signal(|| AgentModel::Sonnet);
    let mut form_max_turns = use_signal(|| 50u32);
    let mut form_icon = use_signal(String::new);
    let mut form_color = use_signal(String::new);

    // Load templates on mount
    use_effect(move || {
        let env = use_environment();
        let db = env.read().model.database().clone();

        spawn(async move {
            match db.list_templates().await {
                Ok(loaded) => {
                    log::info!("Loaded {} templates", loaded.len());
                    templates.set(loaded);
                }
                Err(e) => {
                    log::error!("Failed to load templates: {}", e);
                }
            }
        });
    });

    // Save handler (create or update)
    let handle_save = move |_| {
        let env = use_environment();
        let db = env.read().model.database().clone();
        let current_editing_id = editing_id.read().clone();

        // Build template from form
        let template = AgentTemplate {
            id: AgentTemplateId(current_editing_id.clone().unwrap_or_default()),
            name: form_name.read().clone(),
            system_prompt: form_system_prompt.read().clone(),
            model: form_model.read().clone(),
            max_turns: *form_max_turns.read(),
            icon: if form_icon.read().is_empty() {
                None
            } else {
                Some(form_icon.read().clone())
            },
            color: if form_color.read().is_empty() {
                None
            } else {
                Some(form_color.read().clone())
            },
            created_at: Utc::now(),
        };

        spawn(async move {
            let result = if current_editing_id
                .as_ref()
                .map(|s| s.is_empty())
                .unwrap_or(true)
            {
                // Creating new template
                db.create_template(&template)
                    .await
                    .map(|id| log::info!("Created template: {}", id))
            } else {
                // Updating existing template
                db.update_template(&template)
                    .await
                    .map(|_| log::info!("Updated template: {}", template.id.0.to_sql()))
            };

            if let Err(e) = result {
                log::error!("Failed to save template: {}", e);
            }

            // Reload templates
            if let Ok(loaded) = db.list_templates().await {
                templates.set(loaded);
            }
        });

        // Close editor
        editing_id.set(None);
    };

    // Cancel handler
    let handle_cancel = move |_| {
        editing_id.set(None);
    };

    // Edit handler
    let handle_edit = move |template: AgentTemplate| {
        // Load template into form
        form_name.set(template.name.clone());
        form_system_prompt.set(template.system_prompt.clone());
        form_model.set(template.model.clone());
        form_max_turns.set(template.max_turns);
        form_icon.set(template.icon.clone().unwrap_or_default());
        form_color.set(template.color.clone().unwrap_or_default());

        editing_id.set(Some(template.id.0.to_sql()));
    };

    // Delete handler
    let handle_delete = move |id: String| {
        let env = use_environment();
        let db = env.read().model.database().clone();

        spawn(async move {
            match db.delete_template(&id).await {
                Ok(_) => {
                    log::info!("Deleted template: {}", id);
                    // Reload templates
                    if let Ok(loaded) = db.list_templates().await {
                        templates.set(loaded);
                    }
                }
                Err(e) => {
                    log::error!("Failed to delete template: {}", e);
                }
            }
        });
    };

    // New template button handler
    let handle_new = move |_| {
        // Clear form
        form_name.set(String::new());
        form_system_prompt.set(String::new());
        form_model.set(AgentModel::Sonnet);
        form_max_turns.set(50);
        form_icon.set(String::new());
        form_color.set(String::new());

        editing_id.set(Some(String::new())); // Empty string = creating new
    };

    rsx! {
        div {
            class: "template-manager p-4",

            h2 { "Agent Templates" }

            // Show editor or template list
            if editing_id.read().is_some() {
                // Editor form
                TemplateEditor {
                    name: form_name,
                    system_prompt: form_system_prompt,
                    model: form_model,
                    max_turns: form_max_turns,
                    icon: form_icon,
                    color: form_color,
                    on_save: handle_save,
                    on_cancel: handle_cancel,
                }
            } else {
                // Template list
                div {
                    button {
                        class: "btn btn-primary mb-3",
                        onclick: handle_new,
                        "+ New Template"
                    }

                    div {
                        class: "template-list",
                        for template in templates.read().iter() {
                            TemplateCard {
                                key: "{template.id.0}",
                                template: template.clone(),
                                on_edit: handle_edit,
                                on_delete: handle_delete,
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Template card component for list view
///
/// Shows template summary with edit/delete buttons.
#[component]
fn TemplateCard(
    template: AgentTemplate,
    on_edit: EventHandler<AgentTemplate>,
    on_delete: EventHandler<String>,
) -> Element {
    // Clone template for use in both closures
    let template_for_edit = template.clone();
    let template_id = template.id.0.clone();

    rsx! {
        div {
            class: "template-card border rounded p-3 mb-2",

            div {
                class: "d-flex justify-between items-start",

                div {
                    h4 {
                        class: "mb-1",
                        if let Some(icon) = &template.icon {
                            span { class: "mr-2", "{icon}" }
                        }
                        "{template.name}"
                    }
                    p {
                        class: "text-muted small mb-1",
                        "Model: {template.model}"
                    }
                    p {
                        class: "text-truncate mb-0",
                        style: "max-width: 400px;",
                        "{template.system_prompt}"
                    }
                }

                div {
                    class: "btn-group",
                    button {
                        class: "btn btn-sm btn-outline-primary",
                        onclick: move |_| on_edit.call(template_for_edit.clone()),
                        "Edit"
                    }
                    button {
                        class: "btn btn-sm btn-outline-danger",
                        onclick: move |_| on_delete.call(template_id.clone()),
                        "Delete"
                    }
                }
            }
        }
    }
}

/// Template editor form component
///
/// Form for creating or editing templates.
/// All fields are controlled by parent signals.
#[component]
fn TemplateEditor(
    name: Signal<String>,
    system_prompt: Signal<String>,
    model: Signal<AgentModel>,
    max_turns: Signal<u32>,
    icon: Signal<String>,
    color: Signal<String>,
    on_save: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "template-editor border rounded p-4",

            h3 { class: "mb-4", "Template Editor" }

            div {
                class: "mb-3",
                label {
                    class: "form-label",
                    "Name"
                }
                input {
                    class: "form-control",
                    r#type: "text",
                    value: "{name.read()}",
                    oninput: move |evt| name.set(evt.value().clone()),
                    placeholder: "Enter template name..."
                }
            }

            div {
                class: "mb-3",
                label {
                    class: "form-label",
                    "System Prompt"
                }
                textarea {
                    class: "form-control",
                    value: "{system_prompt.read()}",
                    oninput: move |evt| system_prompt.set(evt.value().clone()),
                    rows: "10",
                    placeholder: "Enter system prompt for the agent..."
                }
            }

            div {
                class: "mb-3",
                label {
                    class: "form-label",
                    "Model"
                }
                select {
                    class: "form-select",
                    value: match *model.read() {
                        AgentModel::Sonnet => "sonnet",
                        AgentModel::Haiku => "haiku",
                        AgentModel::Opus => "opus",
                    },
                    onchange: move |evt| {
                        let selected = match evt.value().as_str() {
                            "haiku" => AgentModel::Haiku,
                            "opus" => AgentModel::Opus,
                            _ => AgentModel::Sonnet,
                        };
                        model.set(selected);
                    },
                    option { value: "sonnet", "Claude 3.5 Sonnet (Balanced)" }
                    option { value: "haiku", "Claude 3 Haiku (Fast)" }
                    option { value: "opus", "Claude 3 Opus (Capable)" }
                }
            }

            div {
                class: "mb-3",
                label {
                    class: "form-label",
                    "Max Turns"
                }
                input {
                    class: "form-control",
                    r#type: "number",
                    value: "{max_turns.read()}",
                    oninput: move |evt| {
                        if let Ok(val) = evt.value().parse::<u32>() {
                            max_turns.set(val);
                        }
                    },
                    min: "1",
                    max: "200"
                }
            }

            div {
                class: "row mb-3",
                div {
                    class: "col-md-6",
                    label {
                        class: "form-label",
                        "Icon (emoji)"
                    }
                    input {
                        class: "form-control",
                        r#type: "text",
                        value: "{icon.read()}",
                        oninput: move |evt| icon.set(evt.value().clone()),
                        placeholder: "🤖"
                    }
                }
                div {
                    class: "col-md-6",
                    label {
                        class: "form-label",
                        "Color (hex)"
                    }
                    input {
                        class: "form-control",
                        r#type: "text",
                        value: "{color.read()}",
                        oninput: move |evt| color.set(evt.value().clone()),
                        placeholder: "#3b82f6"
                    }
                }
            }

            div {
                class: "d-flex gap-2",
                button {
                    class: "btn btn-primary",
                    onclick: move |_evt| on_save.call(()),
                    "Save Template"
                }
                button {
                    class: "btn btn-secondary",
                    onclick: move |_evt| on_cancel.call(()),
                    "Cancel"
                }
            }
        }
    }
}
