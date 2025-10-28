use super::types::Data;
use crate::components::conversation::Conversation;
use crate::environment::model::Status;
use crate::view_model::*;

impl Data {
    /// Will return the currently open conversation (for the current tab)
    /// if there is one
    pub fn conversation(&self, id: &StatusId) -> Option<&Conversation> {
        // let current = self.conversation_id()?;
        self.conversations.get(id)
    }

    pub fn possibly_update_conversation_with_reply(&mut self, reply: &Status) {
        // If this was a reply to an update in the currently loaded conversation,
        // then inject it there
        let Some(reply_id) = &reply.in_reply_to_id.as_ref().map(|e| StatusId(e.clone())) else {
            return;
        };

        for (_, conversation) in self.conversations.iter_mut() {
            conversation
                .insert_child_if(reply_id, StatusViewModel::new(reply))
                .unwrap_or_default();
        }
    }
}
