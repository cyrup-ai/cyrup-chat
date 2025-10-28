//! Room chat view for multi-agent conversations
//!
//! Similar to ChatComponent but:
//! - Uses room_id instead of conversation_id
//! - Uses MentionInput instead of regular input
//! - Parses @mentions before sending
//! - Calls send_room_message() instead of send_chat_message()

use crate::components::chat::mention_input::MentionInput;
use crate::environment::Environment;
use crate::services::{agent_chat, mention_parser};
use dioxus::prelude::*;

#[component]
pub fn RoomChatView(room_id: String) -> Element {
    let environment = use_context::<Environment>();
    let database_for_room = environment.database.clone();
    let database_for_send = environment.database.clone();
    let room_id_for_room = room_id.clone();
    let room_id_for_send = room_id.clone();
    let input_value = use_signal(String::new);
    let mut is_sending = use_signal(|| false);

    // Load room details (title, participants)
    let room = use_resource(move || {
        let database = database_for_room.clone();
        let room_id = room_id_for_room.clone();
        async move {
            log::debug!("[RoomChat] Loading room: {}", room_id);
            database.get_room(&room_id).await
        }
    });

    let send_message = move |message: String| {
        is_sending.set(true);

        // Parse @mentions from message
        let mentions = mention_parser::parse_mentions(&message);
        log::info!("[RoomChat] Parsed {} mentions from message", mentions.len());
        
        spawn({
            let database = database_for_send.clone();
            let room_id = room_id_for_send.clone();
            let mut is_sending = is_sending;

            async move {
                match agent_chat::send_room_message(
                    database,
                    room_id,
                    message,
                    mentions,
                )
                .await
                {
                    Ok(_) => {
                        log::info!("[RoomChat] Message sent successfully");
                    }
                    Err(e) => {
                        log::error!("[RoomChat] Failed to send message: {}", e);
                    }
                }
                is_sending.set(false);
            }
        });
    };

    rsx! {
        div {
            class: "flex flex-col h-screen bg-transparent",

            // Room header (title + participant count)
            match &*room.read() {
                Some(Ok(room_data)) => rsx! {
                    div {
                        class: "p-4 border-b border-white/10 bg-[#1a1a2e]/90 backdrop-blur-sm",
                        div {
                            class: "text-xl font-bold text-white",
                            "{room_data.title}"
                        }
                        div {
                            class: "text-sm text-gray-400 mt-1",
                            "Participants: {room_data.participants.len()} agents"
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    div {
                        class: "p-4 border-b border-white/10 bg-red-500/10",
                        "Error loading room: {e}"
                    }
                },
                None => rsx! {
                    div {
                        class: "p-4 border-b border-white/10 bg-[#1a1a2e]/90",
                        "Loading room..."
                    }
                }
            }

            // Message timeline
            // TODO: Implement LIVE QUERY message loading (same pattern as ChatComponent)
            div {
                class: "flex-1 overflow-y-auto px-6 py-4",
                div {
                    class: "text-gray-400 text-center mt-4",
                    "Messages will appear here"
                }
            }

            // Mention input (with @autocomplete)
            match &*room.read() {
                Some(Ok(room_data)) => rsx! {
                    div {
                        class: "p-4 bg-[#1a1a2e]/90 border-t border-white/10 backdrop-blur-sm",
                        MentionInput {
                            value: input_value,
                            on_submit: send_message,
                            disabled: *is_sending.read(),
                            room_agents: room_data.participants.iter().map(|p| p.0.clone()).collect(),
                        }
                    }
                },
                _ => rsx! {}
            }
        }
    }
}
