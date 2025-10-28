use super::{ChatMessage, MessageSender, ReactionSummary};
use crate::constants::ui_text;
use crate::environment::Environment;
use crate::services::agent_chat;
use crate::view_model::agent::{AgentModel, AgentTemplate, AgentTemplateId};
use crate::view_model::conversation::{Conversation, ConversationId};

use crate::widgets::ErrorBox;
use dioxus::prelude::*;
use futures_util::StreamExt;
use std::collections::HashSet;
use surrealdb::Notification;
use surrealdb_types::Action;

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

    let messages = use_signal(Vec::<ChatMessage>::new);
    let mut input_value = use_signal(String::new);
    let mut is_sending = use_signal(|| false);
    let mut send_error = use_signal(|| Option::<String>::None);
    let mut active_tool = use_signal(|| Option::<String>::None);
    
    // Reply state tracking
    let mut replying_to = use_signal(|| Option::<(String, String)>::None);
    
    // Bookmark state tracking
    let bookmarked_msg_ids = use_signal(|| HashSet::<String>::new());

    // Bootstrap conversation on mount - ensure selected conversation exists
    use_effect({
        let database = environment.database.clone();
        let conversation_id = conversation_id;
        move || {
            let database = database.clone();
            let current_id = conversation_id.read().clone();
            spawn(async move {
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

                        // Create conversation
                        let now = chrono::Utc::now();
                        let conversation = Conversation {
                            id: ConversationId(current_id.clone()),
                            title: "Chat with CYRUP".to_string(),
                            template_id: AgentTemplateId(template_id),
                            summary: "General conversation with AI assistant".to_string(),
                            agent_session_id: None, // Lazy spawn on first message
                            last_summarized_message_id: None,
                            last_message_at: now,
                            created_at: now,
                        };

                        match database.create_conversation(&conversation).await {
                            Ok(id) => {
                                log::info!("[Chat] Created conversation: {}", id);
                            }
                            Err(e) => {
                                log::error!("[Chat] Failed to create conversation: {}", e);
                            }
                        }
                    }
                }
            });
        }
    });

    // Subscribe to LIVE QUERY for real-time message updates
    // React to conversation_id changes - reload messages when conversation changes
    use_effect({
        let database = environment.database.clone();
        let mut messages = messages;
        let conversation_id = conversation_id;
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
                        let mut chat_messages: Vec<ChatMessage> = db_messages
                            .into_iter()
                            .map(ChatMessage::from_db_message)
                            .collect();
                        
                        // Load reactions for each message
                        for msg in chat_messages.iter_mut() {
                            match database.get_reaction_counts(&msg.id).await {
                                Ok(counts) => {
                                    let full_reactions = database.get_message_reactions(&msg.id).await
                                        .unwrap_or_default();
                                    
                                    let user_id = "hardcoded-david-maple";
                                    
                                    msg.reactions = counts.into_iter()
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
                                }
                                Err(e) => {
                                    log::error!("[Chat] Failed to load reactions for message {}: {}", msg.id, e);
                                }
                            }
                        }
                        
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
                                    log::debug!("[Chat] LIVE QUERY: Message deleted");
                                    // For MVP, we don't implement delete UI
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
        let conversation_id = conversation_id;
        move || {
            let database = database.clone();
            let current_id = conversation_id.read().clone();
            spawn(async move {
                log::info!("[Chat] Starting reaction LIVE QUERY subscription");
                
                // Start LIVE QUERY for reactions
                let stream_result = database
                    .client()
                    .query("LIVE SELECT * FROM reaction WHERE message_id IN (SELECT id FROM message WHERE conversation_id = $conv_id)")
                    .bind(("conv_id", current_id.clone()))
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
                
                log::info!("[Chat] Reaction LIVE QUERY subscription active");
                
                // Consume notifications
                while let Some(notification) = stream.next().await {
                    match notification {
                        Ok(notif) => {
                            let reaction = notif.data;
                            let message_id = reaction.message_id.clone();
                            
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
                            .map(|msg| msg.id.0)
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
                    match agent_chat::send_chat_message(
                        database,
                        current_conversation_id,
                        content,
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
                        bookmarked_msg_ids: bookmarked_msg_ids
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
            
            div {
                class: "p-4 bg-gradient-to-r from-[#1a1a2e]/80 to-[#16213e]/80 glass border-t border-white/10",
                form {
                    onsubmit: move |_| {
                        send_message(());
                    },
                    input {
                        class: "flex-1 px-4 py-3 bg-white/5 border border-white/10 rounded-lg text-white text-base transition-colors duration-200 focus:outline-none focus:border-[#00a8ff] placeholder:text-white/40",
                        r#type: "text",
                        placeholder: ui_text::chat_input_placeholder(),
                        value: "{input_value.read()}",
                        oninput: move |e| input_value.set(e.value()),
                        disabled: *is_sending.read(),
                    }
                    button {
                        class: "px-6 py-3 bg-[var(--g-accentColor)] text-white rounded-lg font-semibold cursor-pointer transition-all duration-200 hover:bg-[var(--g-accentColorHighlight)] hover:-translate-y-px active:translate-y-0",
                        r#type: "submit",
                        disabled: *is_sending.read(),
                        "Send"
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
        }
    }
}

#[component]
fn ChatMessageView(
    message: ChatMessage, 
    on_reply: EventHandler<(String, String)>,
    bookmarked_msg_ids: Signal<HashSet<String>>
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

    rsx! {
        div {
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
        log::debug!("[Chat] Using existing template: {}", template.id.0);
        return Ok(template.id.0.clone());
    }

    // No templates exist - create default
    log::info!("[Chat] Creating default template");

    let default_template = AgentTemplate {
        id: AgentTemplateId("template:default".to_string()),
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
    Ok(default_template.id.0)
}
