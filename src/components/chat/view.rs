use super::{ChatMessage, MessageSender, ReactionSummary};
use crate::components::chat::mention_input::MentionInput;
use crate::constants::ui_text;
use crate::environment::Environment;
use crate::services::{agent_chat, mention_parser};
use crate::view_model::agent::{AgentModel, AgentTemplate, AgentTemplateId};
use crate::view_model::conversation::Conversation;

use crate::widgets::ErrorBox;
use dioxus::prelude::*;
use futures_util::StreamExt;
use std::collections::HashSet;
use surrealdb::Notification;
use surrealdb_types::{Action, ToSql};

#[component]
fn PinnedBanner(conversation_id: String) -> Element {
    let environment = use_context::<Environment>();
    let database = environment.database.clone();
    
    // Fetch pinned messages
    let pinned_messages = use_resource(move || {
        let database = database.clone();
        let conv_id = conversation_id.clone();
        async move {
            let db_messages = database.get_pinned_messages(&conv_id).await.ok()?;
            Some(db_messages.into_iter().map(ChatMessage::from_db_message).collect::<Vec<_>>())
        }
    });

    rsx! {
        match &*pinned_messages.read() {
            Some(Some(messages)) if !messages.is_empty() => rsx! {
                div {
                    class: "sticky top-0 z-10 bg-gradient-to-r from-[#1a1a2e]/95 to-[#16213e]/95 backdrop-blur-md border-b border-white/10 p-4 shadow-lg",
                    div {
                        class: "flex items-center gap-2 mb-3",
                        span { class: "text-xl", "📌" }
                        span { class: "text-sm font-semibold text-[var(--g-labelColor)]", "Pinned Messages" }
                        span { class: "text-xs text-[var(--g-secondaryLabelColor)]", "({messages.len()}/5)" }
                    }
                    div {
                        class: "space-y-2",
                        {messages.iter().map(|message| {
                            let msg_id = message.id.clone();
                            let timestamp_str = message.timestamp.format("%H:%M").to_string();
                            let content_str = message.content.clone();
                            
                            rsx! {
                                div {
                                    key: "{msg_id}",
                                    class: "p-3 bg-white/5 border border-white/10 rounded-lg cursor-pointer hover:bg-white/8 transition-all duration-200",
                                    onclick: move |_| {
                                        log::info!("Clicked pinned message: {}", msg_id);
                                    },
                                    div {
                                        class: "text-xs text-[var(--g-secondaryLabelColor)] mb-1",
                                        "{timestamp_str}"
                                    }
                                    div {
                                        class: "text-sm text-[var(--g-labelColor)] line-clamp-2",
                                        "{content_str}"
                                    }
                                }
                            }
                        })}
                    }
                }
            },
            _ => rsx! {}
        }
    }
}

#[component]
pub fn ChatComponent() -> Element {
    // Get environment from context (provided by App component)
    let environment = use_context::<Environment>();
    
    // Get selected conversation ID from context (provided by LoggedInApp)
    let conversation_id = match try_use_context::<Signal<String>>() {
        Some(id) => id,
        None => {
            // Fallback to default if context not available
            use_signal(|| "conversation:default_chat".to_string())
        }
    };

    let mut messages = use_signal(Vec::<ChatMessage>::new);
    let mut input_value = use_signal(String::new);
    let mut is_sending = use_signal(|| false);
    let mut send_error = use_signal(|| Option::<String>::None);
    let mut active_tool = use_signal(|| Option::<String>::None);
    
    // Reply state tracking
    let mut replying_to = use_signal(|| Option::<(String, String)>::None);
    
    // Bookmark state tracking
    let bookmarked_msg_ids = use_signal(HashSet::<String>::new);
    
    // Delete confirmation state tracking
    let mut show_delete_confirmation = use_signal(|| Option::<String>::None);
    
    // Track whether we've scrolled to first unread (one-time per conversation open)
    let mut has_scrolled_to_unread = use_signal(|| false);

    // Load conversation to check participant count for conditional input rendering
    let conversation_for_input = {
        let database = environment.database.clone();
        use_resource(move || {
            let database = database.clone();
            let conv_id = conversation_id.read().clone();
            async move {
                database.get_conversation(&conv_id).await.ok()
            }
        })
    };

    // Bootstrap conversation on mount - ensure selected conversation exists
    use_future({
        let database = environment.database.clone();
        let conversation_id = conversation_id;
        move || {
            let database = database.clone();
            let mut conversation_id = conversation_id;
            async move {
                let current_id = conversation_id.read().clone();
                // Check if selected conversation exists
                match database.get_conversation(&current_id).await {
                    Ok(_) => {
                        log::debug!("[Chat] Conversation exists: {}", current_id);
                        
                        // Mark all messages in this conversation as read
                        if let Err(e) = database.mark_messages_read(&current_id).await {
                            log::error!("[Chat] Failed to mark messages read: {}", e);
                        } else {
                            log::debug!("[Chat] Marked messages read for: {}", current_id);
                        }
                    }
                    Err(_) => {
                        // Conversation doesn't exist - create it
                        log::info!("[Chat] Creating conversation: {}", current_id);

                        // Get or create default template
                        let template_id = match ensure_default_template(&database).await {
                            Ok(id) => id,
                            Err(e) => {
                                log::error!("[Chat] Failed to ensure default template: {}", e);
                                return;
                            }
                        };

                        // Generate unique ID if using default placeholder
                        let new_conversation_id = if current_id == "conversation:default_chat" {
                            format!("conversation:{}", uuid::Uuid::new_v4().to_string().replace("-", ""))
                        } else {
                            current_id.clone()
                        };

                        // Create conversation
                        let now = chrono::Utc::now();
                        let conversation = Conversation {
                            id: new_conversation_id.clone().into(),
                            title: "Chat with CYRUP".to_string(),
                            participants: vec![template_id.into()],
                            summary: "General conversation with AI assistant".to_string(),
                            agent_sessions: std::collections::HashMap::new(), // Lazy spawn on first message
                            last_summarized_message_id: None,
                            last_message_at: now.into(),
                            created_at: now.into(),
                        };

                        match database.create_conversation(&conversation).await {
                            Ok(id) => {
                                log::info!("[Chat] Created conversation: {}", id);
                                // Update the signal to match the actual created ID
                                conversation_id.set(id);
                            }
                            Err(e) => {
                                log::error!("[Chat] Failed to create conversation: {}", e);
                            }
                        }
                    }
                }
            }
        }
    });

    // Subscribe to LIVE QUERY for real-time message updates
    // React to conversation_id changes - reload messages when conversation changes
    use_effect({
        let database = environment.database.clone();
        let mut messages = messages;
        move || {
            let database = database.clone();
            let current_id = conversation_id.read().clone();
            spawn(async move {
                // Mark messages as read when conversation is opened
                if let Err(e) = database.mark_messages_as_read(current_id.clone()).await {
                    log::error!("[Chat] Failed to mark messages as read: {}", e);
                }

                // STEP 1: Load existing messages FIRST
                log::info!("[Chat] Loading existing messages for conversation: {}", current_id);
                match database.get_all_messages(&current_id).await {
                    Ok(db_messages) => {
                        let chat_messages: Vec<ChatMessage> = db_messages
                            .into_iter()
                            .map(ChatMessage::from_db_message)
                            .collect();

                        // Display messages immediately without reactions
                        // Reactions will be populated by LIVE QUERY subscription (line 321-410)
                        let count = chat_messages.len();
                        messages.set(chat_messages);
                        log::info!("[Chat] Loaded {} existing messages", count);
                    }
                    Err(e) => {
                        log::error!("[Chat] Failed to load existing messages: {}", e);
                    }
                }

                // STEP 2: NOW start LIVE QUERY for future changes
                log::info!("[Chat] Starting LIVE QUERY subscription for: {}", current_id);

                // Start LIVE QUERY
                let stream_result = database
                    .client()
                    .query("LIVE SELECT * FROM message WHERE conversation_id = $id")
                    .bind(("id", current_id.clone()))
                    .await;

                let mut stream = match stream_result {
                    Ok(mut response) => {
                        match response
                            .stream::<Notification<crate::view_model::message::Message>>(0)
                        {
                            Ok(s) => s,
                            Err(e) => {
                                log::error!("[Chat] Failed to create stream: {}", e);
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("[Chat] Failed to start LIVE QUERY: {}", e);
                        return;
                    }
                };

                log::info!("[Chat] LIVE QUERY subscription active");

                // Consume notifications
                while let Some(notification) = stream.next().await {
                    match notification {
                        Ok(notif) => {
                            let message_data = notif.data;

                            match notif.action {
                                Action::Create => {
                                    // New message - append to list
                                    log::debug!("[Chat] LIVE QUERY: New message created");
                                    let chat_msg = ChatMessage::from_db_message(message_data);
                                    messages.write().push(chat_msg);
                                }
                                Action::Update => {
                                    // Message updated (streaming!) - find by ID and replace
                                    log::debug!("[Chat] LIVE QUERY: Message updated (streaming)");
                                    let chat_msg = ChatMessage::from_db_message(message_data);
                                    let mut msgs = messages.write();

                                    // Match by database ID
                                    if let Some(pos) = msgs.iter().position(|m| m.id == chat_msg.id)
                                    {
                                        msgs[pos] = chat_msg;
                                    } else {
                                        log::warn!(
                                            "[Chat] Update for unknown message ID: {}",
                                            chat_msg.id
                                        );
                                        // Message not found - may have been deleted or not yet loaded
                                        // Don't crash, just log warning
                                    }
                                }
                                Action::Delete => {
                                    // Message deleted - remove from list
                                    log::debug!("[Chat] LIVE QUERY: Message deleted - {}", message_data.id.0.to_sql());
                                    let mut msgs = messages.write();
                                    msgs.retain(|m| m.id != message_data.id.0.to_sql());
                                }
                                _ => {}
                            }
                        }
                        Err(e) => {
                            log::error!("[Chat] LIVE QUERY notification error: {}", e);
                        }
                    }
                }

                log::warn!("[Chat] LIVE QUERY stream ended");
            });
        }
    });

    // Subscribe to LIVE QUERY for reaction updates
    use_effect({
        let database = environment.database.clone();
        let mut messages = messages;
        let _conversation_id = conversation_id;
        move || {
            let database = database.clone();
            spawn(async move {
                log::info!("[Chat] Starting reaction LIVE QUERY subscription (global, app-filtered)");
                
                // Start LIVE QUERY for reactions
                let stream_result = database
                    .client()
                    .query("LIVE SELECT * FROM reaction")
                    .await;
                
                let mut stream = match stream_result {
                    Ok(mut response) => {
                        match response.stream::<Notification<crate::database::reactions::Reaction>>(0) {
                            Ok(s) => s,
                            Err(e) => {
                                log::error!("[Chat] Failed to create reaction stream: {}", e);
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("[Chat] Failed to start reaction LIVE QUERY: {}", e);
                        return;
                    }
                };
                
                log::info!("[Chat] Reaction LIVE QUERY subscription active (filtering to {} messages)", messages.read().len());
                
                // Consume notifications
                while let Some(notification) = stream.next().await {
                    match notification {
                        Ok(notif) => {
                            let reaction = notif.data;
                            let message_id = reaction.message_id.clone();
                            
                            // ✅ FILTER: Only process reactions for messages in this conversation
                            let message_exists = messages.read().iter().any(|m| m.id == message_id);
                            if !message_exists {
                                log::trace!("[Chat] Ignoring reaction for message not in conversation: {}", message_id);
                                continue; // Skip reactions for other conversations
                            }
                            
                            log::debug!("[Chat] Processing reaction for message: {}", message_id);
                            
                            // Reload reactions for this message
                            match database.get_reaction_counts(&message_id).await {
                                Ok(counts) => {
                                    // Get full reactions to check user participation
                                    let full_reactions = database.get_message_reactions(&message_id).await
                                        .unwrap_or_default();
                                    
                                    let user_id = "hardcoded-david-maple";
                                    
                                    // Convert to ReactionSummary
                                    let reaction_summaries: Vec<ReactionSummary> = counts.into_iter()
                                        .map(|(emoji, count)| {
                                            let user_reacted = full_reactions.iter()
                                                .any(|r| r.emoji == emoji && r.user_id == user_id);
                                            ReactionSummary {
                                                emoji,
                                                count,
                                                user_reacted,
                                            }
                                        })
                                        .collect();
                                    
                                    // Update the message in the messages list
                                    let mut msgs = messages.write();
                                    if let Some(msg) = msgs.iter_mut().find(|m| m.id == message_id) {
                                        msg.reactions = reaction_summaries;
                                    }
                                }
                                Err(e) => {
                                    log::error!("[Chat] Failed to reload reaction counts: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("[Chat] Reaction LIVE QUERY notification error: {}", e);
                        }
                    }
                }
                
                log::warn!("[Chat] Reaction LIVE QUERY stream ended");
            });
        }
    });

    // Load bookmarked message IDs on mount
    use_effect({
        let database = environment.database.clone();
        let mut bookmarked_msg_ids = bookmarked_msg_ids;
        move || {
            let database = database.clone();
            spawn(async move {
                let user_id = "hardcoded-david-maple";
                match database.get_bookmarked_messages(user_id).await {
                    Ok(messages) => {
                        let ids: HashSet<String> = messages
                            .into_iter()
                            .map(|msg| msg.id.0.to_sql())
                            .collect();
                        bookmarked_msg_ids.set(ids);
                        log::debug!("[Chat] Loaded {} bookmarked messages", bookmarked_msg_ids.read().len());
                    }
                    Err(e) => {
                        log::error!("[Chat] Failed to load bookmarks: {}", e);
                    }
                }
            });
        }
    });

    // Watch for conversation ID changes and reset scroll flag
    use_effect(move || {
        let _current_id = conversation_id.read().clone();
        
        // Reset scroll-to-unread flag when conversation changes
        has_scrolled_to_unread.set(false);
        
        // Reset messages (they will be reloaded by LIVE QUERY)
        messages.set(Vec::new());
    });

    // Scroll to first unread message on conversation load
    use_effect(move || {
        let messages_list = messages.read().clone();
        let scrolled = *has_scrolled_to_unread.read();
        
        // Only scroll once per conversation open, and only if there are unreads
        if !scrolled && !messages_list.is_empty()
            && let Some(first_unread) = messages_list.iter().find(|msg| msg.unread) {
            let message_id = first_unread.id.clone();
            has_scrolled_to_unread.set(true);
            
            log::debug!("[Chat] Scrolling to first unread: {}", message_id);
            
            spawn(async move {
                use dioxus::document;
                
                let script = format!(
                    r#"
                    const element = document.getElementById('message-{}');
                    if (element) {{
                        element.scrollIntoView({{ behavior: 'smooth', block: 'center' }});
                        // Highlight briefly
                        element.style.backgroundColor = 'rgba(59, 130, 246, 0.2)';
                        setTimeout(() => element.style.backgroundColor = '', 2000);
                    }}
                    "#,
                    message_id
                );
                
                let eval_result = document::eval(&script);
                if let Err(e) = eval_result.await {
                    log::warn!("[Chat] Failed to scroll to unread: {}", e);
                }
            });
        }
    });

    // Clone database for use in MentionInput on_submit handler (needed before send_message captures environment)
    let database_for_mention_input = environment.database.clone();

    let mut send_message = move |_| {
        let content = input_value.read().trim().to_string();
        if !content.is_empty() && !*is_sending.read() {
            input_value.set(String::new());
            is_sending.set(true);

            // Capture reply target before clearing
            let parent_message_id = replying_to.read().as_ref().map(|(id, _)| id.clone());
            replying_to.set(None); // Clear reply state after capturing

            // Send message via agent_chat service (stateless, database-driven)
            spawn({
                let database = environment.database.clone();
                let content = content.clone();
                let mut is_sending = is_sending;
                let mut send_error = send_error;
                let current_conversation_id = conversation_id.read().clone();

                async move {
                    match agent_chat::send_message(
                        database,
                        current_conversation_id,
                        content,
                        None, // mentioned_agents (None = use all participants)
                        parent_message_id,
                    )
                    .await
                    {
                        Ok(_) => {
                            log::debug!("[Chat] Message sent successfully");
                            // LIVE QUERY handles UI updates automatically!
                        }
                        Err(e) => {
                            log::error!("[Chat] Failed to send message: {}", e);

                            // Show user-friendly error message
                            let friendly_message = if e.contains("create Claude client") {
                                "Unable to connect to AI assistant. Please try again."
                            } else if e.contains("send to agent") {
                                "Unable to send message. Please try again."
                            } else {
                                "Failed to send message. Please try again."
                            };

                            send_error.set(Some(friendly_message.to_string()));

                            // Auto-clear after 5 seconds
                            spawn({
                                let mut send_error = send_error;
                                async move {
                                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                                    send_error.set(None);
                                }
                            });
                        }
                    }
                    is_sending.set(false);
                }
            });
        }
    };

    let start_reply = move |(message_id, author): (String, String)| {
        replying_to.set(Some((message_id, author)));
    };

    let cancel_reply = move |_| {
        replying_to.set(None);
    };

    let confirm_delete = move |message_id: String| {
        show_delete_confirmation.set(None);
        
        spawn({
            let model = environment.model.clone();
            async move {
                match model.delete_status(message_id.clone()).await {
                    Ok(_) => {
                        log::info!("[Chat] Message deleted: {}", message_id);
                        // LIVE QUERY automatically removes message from UI
                    }
                    Err(e) => {
                        log::error!("[Chat] Failed to delete message: {}", e);
                        // TODO: Show error toast to user
                    }
                }
            }
        });
    };

    let cancel_delete = move |_| {
        show_delete_confirmation.set(None);
    };

    // Subscribe to agent tool-use events
    use_effect(move || {
        spawn(async move {
            let (_, receiver) = crate::services::agent_chat::get_tool_event_channel();

            loop {
                match receiver.recv_async().await {
                    Ok(tool_name) => {
                        log::debug!("[Chat] Agent using tool: {}", tool_name);
                        active_tool.set(Some(tool_name));

                        // Auto-clear after 3 seconds
                        let mut active_tool_clear = active_tool;
                        spawn(async move {
                            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                            active_tool_clear.set(None);
                        });
                    }
                    Err(e) => {
                        log::error!("[Chat] Tool event channel closed: {}", e);
                        break;
                    }
                }
            }
        });
    });

    rsx! {
        div {
            class: "flex flex-col h-screen bg-transparent",
            
            PinnedBanner { conversation_id: conversation_id.read().clone() }
            
            div {
                class: "flex-1 overflow-y-auto px-6 py-4 flex flex-col gap-6",
                for message in messages.read().iter() {
                    ChatMessageView { 
                        message: message.clone(),
                        on_reply: start_reply,
                        bookmarked_msg_ids: bookmarked_msg_ids,
                        show_delete_confirmation: show_delete_confirmation
                    }
                }
                if *is_sending.read() {
                    div {
                        class: "self-start px-4 py-3 bg-white/5 border border-white/10 rounded-lg opacity-70 italic animate-[pulse_1.5s_infinite]",
                        if let Some(tool) = active_tool.read().as_ref() {
                            "CYRUP is using {tool}..."
                        } else {
                            "CYRUP is thinking..."
                        }
                    }
                }
            }
            
            // Reply indicator (shows when replying_to is Some)
            if let Some((_, author)) = replying_to.read().as_ref() {
                div {
                    class: "px-4 py-2 bg-blue-500/10 border-l-4 border-blue-500 flex justify-between items-center",
                    div {
                        class: "flex items-center gap-2 text-sm text-blue-300",
                        span { "↩️" }
                        span { "Replying to {author}" }
                    }
                    button {
                        class: "text-white/50 hover:text-white transition-colors",
                        onclick: cancel_reply,
                        "✕"
                    }
                }
            }
            
            // Load conversation to check participant count
            {
                let is_multi_agent = conversation_for_input.read().as_ref()
                    .and_then(|opt| opt.as_ref())
                    .map(|conv| conv.participants.len() > 1)
                    .unwrap_or(false);
                
                let room_agents = conversation_for_input.read().as_ref()
                    .and_then(|opt| opt.as_ref())
                    .map(|conv| conv.participants.iter().map(|p| p.0.to_sql()).collect::<Vec<String>>())
                    .unwrap_or_default();
                
                if is_multi_agent {
                    let database_for_submit = database_for_mention_input.clone();
                    let room_agents_for_submit = room_agents.clone();
                    
                    rsx! {
                        // Multi-agent: Use MentionInput with @mention autocomplete
                        div {
                            class: "p-4 bg-gradient-to-r from-[#1a1a2e]/80 to-[#16213e]/80 glass border-t border-white/10",
                            MentionInput {
                                value: input_value,
                                on_submit: move |msg: String| {
                                    // Parse @mentions from message
                                    let mentioned = mention_parser::parse_mentions(&msg);
                                    
                                    // Filter to only participants in this conversation
                                    let valid_mentions: Vec<String> = mentioned
                                        .into_iter()
                                        .filter(|agent_id| room_agents_for_submit.contains(agent_id))
                                        .collect();
                                    
                                    // Send with mentioned_agents parameter
                                    input_value.set(String::new());
                                    is_sending.set(true);
                                    
                                    spawn({
                                        let database = database_for_submit.clone();
                                        let current_conversation_id = conversation_id.read().clone();
                                        let mut is_sending = is_sending;
                                        
                                        async move {
                                            match agent_chat::send_message(
                                                database,
                                                current_conversation_id,
                                                msg,
                                                Some(valid_mentions),
                                                None,
                                            ).await {
                                                Ok(_) => log::debug!("[Chat] Message sent to mentioned agents"),
                                                Err(e) => log::error!("[Chat] Failed: {}", e),
                                            }
                                            is_sending.set(false);
                                        }
                                    });
                                },
                                disabled: *is_sending.read(),
                                room_agents: room_agents,
                            }
                        }
                    }
                } else {
                    rsx! {
                        // Single-agent: Use regular input with professional styling
                        div {
                            class: "p-6 bg-gradient-to-r from-[#1a1a2e]/95 to-[#16213e]/95 backdrop-blur-xl glass border-t border-white/20 shadow-[0_-4px_20px_rgba(0,0,0,0.3)]",
                            form {
                                class: "flex items-center gap-3",
                                onsubmit: move |evt| {
                                    evt.prevent_default();
                                    send_message(());
                                },
                                input {
                                    class: "flex-1 px-5 py-4 bg-white/10 border border-white/20 rounded-xl text-white text-base transition-all duration-200 focus:outline-none focus:border-[#00a8ff] focus:bg-white/15 focus:shadow-[0_0_20px_rgba(0,168,255,0.2)] placeholder:text-white/50 shadow-inner",
                                    r#type: "text",
                                    placeholder: ui_text::chat_input_placeholder(),
                                    value: "{input_value.read()}",
                                    oninput: move |e| input_value.set(e.value()),
                                    disabled: *is_sending.read(),
                                }
                                button {
                                    class: "px-8 py-4 bg-gradient-to-r from-[#0078ff] to-[#00a8ff] text-white rounded-xl font-bold cursor-pointer transition-all duration-200 hover:from-[#0088ff] hover:to-[#00b8ff] hover:shadow-[0_4px_20px_rgba(0,168,255,0.4)] hover:-translate-y-0.5 active:translate-y-0 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:translate-y-0",
                                    r#type: "submit",
                                    disabled: *is_sending.read(),
                                    "Send"
                                }
                            }
                        }
                    }
                }
            }

            // Display error message if present
            if let Some(error_msg) = send_error.read().as_ref() {
                ErrorBox {
                    content: error_msg.clone(),
                    onclick: move |_| {
                        send_error.set(None);
                    }
                }
            }

            // Delete confirmation dialog
            if let Some(message_id) = show_delete_confirmation.read().as_ref() {
                DeleteConfirmationDialog {
                    message_id: message_id.clone(),
                    on_confirm: confirm_delete,
                    on_cancel: cancel_delete,
                }
            }
        }
    }
}

#[component]
fn DeleteConfirmationDialog(
    message_id: String,
    on_confirm: EventHandler<String>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50",
            onclick: move |_| on_cancel.call(()),

            div {
                class: "bg-gradient-to-br from-[#1a1a2e] to-[#16213e] rounded-xl p-6 w-[400px] border border-white/10 shadow-2xl",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    class: "flex items-center gap-3 mb-4",
                    span {
                        class: "text-2xl",
                        "🗑️"
                    }
                    h3 {
                        class: "text-xl font-bold text-white",
                        "Delete Message?"
                    }
                }

                // Explanation
                div {
                    class: "mb-6 text-sm text-gray-300 space-y-2",
                    p {
                        "This message will be removed from the chat."
                    }
                    p {
                        class: "text-xs text-gray-400 italic",
                        "Note: The message remains in the database for AI context but is hidden from view."
                    }
                }

                // Action buttons
                div {
                    class: "flex gap-3",
                    button {
                        class: "flex-1 px-4 py-2 bg-white/5 border border-white/10 rounded-lg text-white hover:bg-white/10 transition-all duration-200",
                        onclick: move |_| on_cancel.call(()),
                        "Cancel"
                    }
                    button {
                        class: "flex-1 px-4 py-2 bg-red-500 rounded-lg text-white font-semibold hover:bg-red-600 transition-all duration-200",
                        onclick: move |_| on_confirm.call(message_id.clone()),
                        "Delete"
                    }
                }
            }
        }
    }
}

#[component]
fn ChatMessageView(
    message: ChatMessage, 
    on_reply: EventHandler<(String, String)>,
    bookmarked_msg_ids: Signal<HashSet<String>>,
    mut show_delete_confirmation: Signal<Option<String>>
) -> Element {
    let environment = use_context::<Environment>();
    let model = environment.model.clone();
    let database = environment.database.clone();
    
    // State for emoji picker
    let mut show_emoji_picker = use_signal(|| false);
    
    let (sender_classes, sender_name, sender_icon) = match message.sender {
        MessageSender::User => (
            "self-end bg-gradient-to-br from-[#0078ff]/20 to-[#00a8ff]/20 border border-[#00a8ff]/30 [box-shadow:inset_0_0_0_2000px_rgba(0,0,0,0.1)]",
            "You",
            "",
        ),
        MessageSender::Cyrup => (
            "self-start bg-white/5 border border-white/10 [box-shadow:inset_0_0_0_2000px_rgba(0,0,0,0.1)]",
            "CYRUP",
            "",
        ),
        MessageSender::System => (
            "self-center bg-yellow-500/10 border border-yellow-500/30 italic",
            "System",
            "ℹ️ ",
        ),
        MessageSender::Tool => (
            "self-start bg-green-500/10 border border-green-500/30 font-mono text-sm",
            "Tool",
            "🔧 ",
        ),
    };

    let indent_class = if message.in_reply_to.is_some() {
        "ml-8 border-l-2 border-white/20 pl-4"
    } else {
        ""
    };

    // Clone fields before closures to avoid ownership conflict
    let message_id_for_pin = message.id.clone();
    let message_pinned = message.pinned;
    let message_id_for_bookmark = message.id.clone();
    let message_is_bookmarked = bookmarked_msg_ids.read().contains(&message.id);
    let message_id_for_reply = message.id.clone();
    let message_sender = message.sender.clone();
    let message_id_for_emoji_picker = message.id.clone();
    let database_for_emoji_picker = database.clone();
    let message_id_for_reactions = message.id.clone();
    let database_for_reactions = database.clone();
    let database_for_bookmark = database.clone();
    let bookmarked_msg_ids_for_button = bookmarked_msg_ids;
    let message_id_for_delete = message.id.clone();

    rsx! {
        div {
            id: "message-{message.id}",
            class: "group relative flex flex-col px-5 py-4 rounded-xl max-w-[70%] animate-[slideIn_0.3s_ease-out] {sender_classes} {indent_class}",
            
            // Unread indicator dot
            if message.unread {
                div {
                    class: "absolute top-2 left-2 flex items-center gap-2 z-10",
                    div {
                        class: "w-2 h-2 bg-blue-500 rounded-full animate-pulse",
                        title: "Unread message"
                    }
                    span {
                        class: "text-xs text-blue-400 font-semibold",
                        "NEW"
                    }
                }
            }
            
            // Pin and Reaction buttons (visible on hover)
            div {
                class: "absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity duration-200 flex gap-2",
                button {
                    class: if message_pinned {
                        "px-2 py-1 bg-gradient-to-br from-[#0078ff]/30 to-[#00a8ff]/30 border border-[#00a8ff]/50 rounded text-white cursor-pointer transition-all duration-200 hover:from-[#0078ff]/40 hover:to-[#00a8ff]/40"
                    } else {
                        "px-2 py-1 bg-white/5 border border-white/10 rounded text-white/50 cursor-pointer transition-all duration-200 hover:bg-white/10 hover:border-white/20 hover:text-white"
                    },
                    onclick: move |_| {
                        let message_id = message_id_for_pin.clone();
                        let model_clone = model.clone();
                        let is_pinned = message_pinned;
                        spawn(async move {
                            let result = if is_pinned {
                                model_clone.unpin_status(message_id).await
                            } else {
                                model_clone.pin_status(message_id).await
                            };
                            if let Err(e) = result {
                                log::error!("Failed to toggle pin: {}", e);
                            }
                        });
                    },
                    "📌"
                }
                button {
                    class: if message_is_bookmarked {
                        "px-2 py-1 bg-gradient-to-br from-[#0078ff]/30 to-[#00a8ff]/30 border border-[#00a8ff]/50 rounded text-white cursor-pointer transition-all duration-200 hover:from-[#0078ff]/40 hover:to-[#00a8ff]/40"
                    } else {
                        "px-2 py-1 bg-white/5 border border-white/10 rounded text-white/50 cursor-pointer transition-all duration-200 hover:bg-white/10 hover:border-white/20 hover:text-white"
                    },
                    onclick: move |_| {
                        let message_id = message_id_for_bookmark.clone();
                        let database = database_for_bookmark.clone();
                        let is_bookmarked = message_is_bookmarked;
                        let mut bookmarked_ids = bookmarked_msg_ids_for_button;
                        spawn(async move {
                            let user_id = "hardcoded-david-maple";
                            let result = if is_bookmarked {
                                database.unbookmark_message(user_id, &message_id).await
                            } else {
                                database.bookmark_message(user_id, &message_id).await.map(|_| ())
                            };
                            
                            match result {
                                Ok(_) => {
                                    // Update the local state
                                    let mut ids = bookmarked_ids.read().clone();
                                    if is_bookmarked {
                                        ids.remove(&message_id);
                                    } else {
                                        ids.insert(message_id.clone());
                                    }
                                    bookmarked_ids.set(ids);
                                }
                                Err(e) => {
                                    log::error!("Failed to toggle bookmark: {}", e);
                                }
                            }
                        });
                    },
                    dangerous_inner_html: if message_is_bookmarked {
                        crate::icons::ICON_BOOKMARK2
                    } else {
                        crate::icons::ICON_BOOKMARK1
                    }
                }
                button {
                    class: "px-2 py-1 bg-white/5 border border-white/10 rounded text-white/70 cursor-pointer transition-all duration-200 hover:bg-white/10 hover:border-white/20 hover:text-white",
                    onclick: move |_| {
                        let current = *show_emoji_picker.read();
                        show_emoji_picker.set(!current);
                    },
                    "😊"
                }
            }
            
            // Delete button (visible on hover, only for user messages)
            if matches!(message.sender, MessageSender::User) {
                div {
                    class: "absolute top-2 right-12 opacity-0 group-hover:opacity-100 transition-opacity duration-200",
                    button {
                        class: "px-2 py-1 bg-red-500/20 border border-red-500/50 rounded text-white/70 cursor-pointer transition-all duration-200 hover:bg-red-500/30 hover:border-red-500 hover:text-white",
                        onclick: move |_| {
                            let message_id = message_id_for_delete.clone();
                            show_delete_confirmation.set(Some(message_id));
                        },
                        "🗑️"
                    }
                }
            }
            
            div {
                class: "flex justify-between mb-2 text-[0.85em] opacity-70",
                span {
                    class: "font-semibold",
                    "{sender_icon}{sender_name}"
                }
                span {
                    class: "text-[0.9em]",
                    {message.timestamp.format("%H:%M").to_string()}
                }
            }
            
            // Show "Reply to [author]" if this is a reply
            if let Some(reply_author) = message.reply_to_author.as_ref() {
                div {
                    class: "text-xs text-white/50 mb-1 italic",
                    "↩️ Reply to {reply_author}"
                }
            }
            
            div {
                class: "text-[var(--g-labelColor)] leading-relaxed whitespace-pre-wrap",
                "{message.content}"
            }
            
            // Emoji picker (conditional render)
            if *show_emoji_picker.read() {
                div {
                    class: "mt-2 p-3 bg-gradient-to-br from-[#1a1a2e]/95 to-[#16213e]/95 border border-white/20 rounded-lg shadow-xl backdrop-blur-sm",
                    div {
                        class: "grid grid-cols-5 gap-2",
                        for emoji in ["👍", "❤️", "😂", "🎯", "✅", "🔥", "👏", "🙌"] {
                            {
                                let emoji_str = emoji.to_string();
                                let message_id = message_id_for_emoji_picker.clone();
                                let database = database_for_emoji_picker.clone();
                                
                                rsx! {
                                    button {
                                        class: "text-2xl hover:scale-125 transition-transform cursor-pointer p-2 hover:bg-white/10 rounded",
                                        onclick: move |_| {
                                            let emoji_str = emoji_str.clone();
                                            let message_id = message_id.clone();
                                            let database = database.clone();
                                    
                                    spawn(async move {
                                        let user_id = "hardcoded-david-maple";
                                        
                                        // Check if user already reacted with this emoji
                                        let reactions = database.get_message_reactions(&message_id).await
                                            .unwrap_or_default();
                                        
                                        let already_reacted = reactions.iter()
                                            .any(|r| r.user_id == user_id && r.emoji == emoji_str);
                                        
                                        if already_reacted {
                                            // Remove reaction
                                            if let Err(e) = database.remove_reaction(&message_id, user_id, &emoji_str).await {
                                                log::error!("Failed to remove reaction: {}", e);
                                            }
                                        } else {
                                            // Add reaction
                                            if let Err(e) = database.add_reaction(&message_id, user_id, &emoji_str).await {
                                                log::error!("Failed to add reaction: {}", e);
                                            }
                                        }
                                    });
                                            
                                            // Close picker after selection
                                            show_emoji_picker.set(false);
                                        },
                                        "{emoji_str}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Reaction display
            if !message.reactions.is_empty() {
                div {
                    class: "flex flex-wrap gap-2 mt-3",
                    for reaction in message.reactions.iter() {
                        {
                            let emoji_str = reaction.emoji.clone();
                            let count = reaction.count;
                            let user_reacted = reaction.user_reacted;
                            let message_id = message_id_for_reactions.clone();
                            let database = database_for_reactions.clone();
                            
                            rsx! {
                                button {
                                    class: if user_reacted {
                                        "px-3 py-1 bg-gradient-to-br from-[#0078ff]/30 to-[#00a8ff]/30 border border-[#00a8ff]/50 rounded-full text-sm flex items-center gap-1 cursor-pointer transition-all duration-200 hover:from-[#0078ff]/40 hover:to-[#00a8ff]/40"
                                    } else {
                                        "px-3 py-1 bg-white/5 border border-white/10 rounded-full text-sm flex items-center gap-1 cursor-pointer transition-all duration-200 hover:bg-white/10 hover:border-white/20"
                                    },
                                    onclick: move |_| {
                                        let emoji_str = emoji_str.clone();
                                        let message_id = message_id.clone();
                                        let database = database.clone();
                                        
                                        spawn(async move {
                                            let user_id = "hardcoded-david-maple";
                                            
                                            if user_reacted {
                                                // Remove reaction
                                                if let Err(e) = database.remove_reaction(&message_id, user_id, &emoji_str).await {
                                                    log::error!("Failed to remove reaction: {}", e);
                                                }
                                            } else {
                                                // Add reaction
                                                if let Err(e) = database.add_reaction(&message_id, user_id, &emoji_str).await {
                                                    log::error!("Failed to add reaction: {}", e);
                                                }
                                            }
                                        });
                                    },
                                    span { class: "text-base", "{emoji_str}" }
                                    span { class: "text-xs text-white/70", "{count}" }
                                }
                            }
                        }
                    }
                }
            }
            
            // Reply button (only show for User and Cyrup messages)
            if !matches!(message.sender, MessageSender::System | MessageSender::Tool) {
                button {
                    class: "mt-2 text-xs text-white/50 hover:text-white/80 transition-colors self-start",
                    onclick: move |_| {
                        let author = match message_sender {
                            MessageSender::User => "You",
                            MessageSender::Cyrup => "CYRUP",
                            _ => "Unknown"
                        };
                        on_reply.call((message_id_for_reply.clone(), author.to_string()));
                    },
                    "Reply"
                }
            }
        }
    }
}

/// Ensure default template exists, create if needed
///
/// Returns template_id of default template
async fn ensure_default_template(
    database: &std::sync::Arc<crate::database::Database>,
) -> Result<String, String> {
    // Try to get any existing template
    let templates = database.list_agent_templates().await?;

    if let Some(template) = templates.first() {
        log::debug!("[Chat] Using existing template: {}", template.id);
        return Ok(template.id.0.to_sql());
    }

    // No templates exist - create default
    log::info!("[Chat] Creating default template");

    let default_template = AgentTemplate {
        id: AgentTemplateId(surrealdb_types::RecordId::new("agent_template", "default")),
        name: "CYRUP Assistant".to_string(),
        system_prompt:
            "You are CYRUP, a helpful AI assistant. Provide clear, concise, and accurate responses."
                .to_string(),
        model: AgentModel::Sonnet,
        max_turns: 25,
        icon: None,
        color: None,
        created_at: chrono::Utc::now(),
    };

    database.create_template(&default_template).await?;
    Ok(default_template.id.0.to_sql())
}
