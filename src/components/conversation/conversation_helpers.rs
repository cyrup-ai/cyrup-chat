use std::collections::HashMap;

use crate::environment::model::Model;
use crate::view_model::StatusId;
use crate::view_model::StatusViewModel;
use id_tree::InsertBehavior::*;
use id_tree::*;
use surrealdb_types::ToSql;

#[derive(Debug, Clone)]
pub struct Conversation {
    original_status: String,
    tree: Tree<StatusViewModel>,
}

impl PartialEq for Conversation {
    fn eq(&self, other: &Self) -> bool {
        self.original_status == other.original_status && self.tree == other.tree
    }
}

impl Eq for Conversation {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConversationItem<'a> {
    node: &'a Node<StatusViewModel>,
    id: &'a NodeId,
}

impl<'a> ConversationItem<'a> {
    pub fn cloned_status(&self) -> StatusViewModel {
        self.node.data().clone()
    }
}

impl<'a> std::ops::Deref for ConversationItem<'a> {
    type Target = StatusViewModel;

    fn deref(&self) -> &Self::Target {
        self.node.data()
    }
}

impl Conversation {
    /// Create a conversation from a megalodon Context
    pub fn from_context(context: megalodon::entities::Context) -> Self {
        use crate::view_model::StatusViewModel;

        let mut tree = Tree::new();
        let mut original_status = String::new();

        // First, create nodes for all statuses
        let mut status_nodes = HashMap::new();

        // Process ancestors first
        for status in &context.ancestors {
            let status_vm = StatusViewModel::from(status.clone());
            if original_status.is_empty() {
                original_status = status_vm.id.0.clone();
            }
            match tree.insert(Node::new(status_vm), AsRoot) {
                Ok(node_id) => {
                    status_nodes.insert(status.id.clone(), node_id);
                }
                Err(e) => {
                    log::error!("Failed to insert ancestor status into conversation tree: {e:?}");
                    log::warn!("Skipping ancestor status: {}", status.id);
                    // Continue processing other statuses instead of crashing
                }
            }
        }

        // Process descendants
        for status in &context.descendants {
            let status_vm = StatusViewModel::from(status.clone());
            if original_status.is_empty() {
                original_status = status_vm.id.0.clone();
            }
            match tree.insert(Node::new(status_vm), AsRoot) {
                Ok(node_id) => {
                    status_nodes.insert(status.id.clone(), node_id);
                }
                Err(e) => {
                    log::error!("Failed to insert descendant status into conversation tree: {e:?}");
                    log::warn!("Skipping descendant status: {}", status.id);
                    // Continue processing other statuses instead of crashing
                }
            }
        }

        Self {
            original_status,
            tree,
        }
    }

    pub fn status(&self) -> StatusId {
        StatusId(self.original_status.clone())
    }

    pub fn root(&self) -> Option<ConversationItem<'_>> {
        let id = self.tree.root_node_id()?;
        let node = self.tree.get(id).ok()?;
        Some(ConversationItem { node, id })
    }

    pub fn children<'a>(&'a self, of: &ConversationItem) -> Option<Vec<ConversationItem<'a>>> {
        let children_ids = self.tree.children_ids(of.id).ok()?;
        Some(
            children_ids
                .filter_map(|e| self.tree.get(e).ok().map(|i| (e, i)))
                .map(|(id, node)| ConversationItem { node, id })
                .collect(),
        )
    }

    // insert as a child if the parent `id` exists and if `child.id`
    // doesn't exist yet as a child
    pub fn insert_child_if(&mut self, id: &StatusId, child: StatusViewModel) -> Option<bool> {
        use id_tree::InsertBehavior::*;
        let root_id = self.tree.root_node_id()?;
        let mut found_id = None;
        for node_id in self.tree.traverse_pre_order_ids(root_id).ok()? {
            let Ok(item) = self.tree.get(&node_id) else {
                continue;
            };
            if &item.data().id == id {
                // check if this node already has the reply
                for child_id in item.children() {
                    let Ok(child_item) = self.tree.get(child_id) else {
                        continue;
                    };
                    if child_item.data().id == child.id {
                        // we stop
                        return Some(false);
                    }
                }
                // otherwise, we found it and can insert
                found_id = Some(node_id);
                break;
            }
        }

        let found_id = found_id?;

        self.tree
            .insert(Node::new(child), UnderNode(&found_id))
            .ok()?;

        Some(true)
    }

    pub fn mutate_post<'a, C: FnMut(&'a mut StatusViewModel)>(
        &'a mut self,
        id: &StatusId,
        action: &'a mut C,
    ) -> bool {
        let Some(root_id) = self.tree.root_node_id() else {
            return false;
        };
        let mut found = None;
        let Some(iter) = self.tree.traverse_pre_order_ids(root_id).ok() else {
            return false;
        };
        for node_id in iter {
            if let Ok(item) = self.tree.get(&node_id)
                && &item.data().id == id
            {
                found = Some(node_id);
                break;
            }
        }

        if let Some(node_id) = found
            && let Ok(item) = self.tree.get_mut(&node_id)
        {
            action(item.data_mut());
            return true;
        }
        false
    }
}

#[allow(dead_code)] // Used for conversation view - implementation pending
pub async fn build_conversation(model: &Model, status_id: String) -> Result<Conversation, String> {
    // Get all messages in the conversation thread
    let messages = model.status_context(status_id.clone()).await?;

    if messages.is_empty() {
        return Err("No messages found in conversation".to_string());
    }

    use id_tree::InsertBehavior::*;

    let mut tree: Tree<StatusViewModel> = TreeBuilder::new().with_node_capacity(32).build();

    // Find root message (one with no in_reply_to)
    let root_message = messages
        .iter()
        .find(|m| m.in_reply_to.is_none())
        .or_else(|| messages.first())
        .ok_or_else(|| "No root message found".to_string())?;

    // Convert Message to StatusViewModel
    let root_view_model = message_to_status_view_model(root_message);

    let root_id: NodeId = tree
        .insert(Node::new(root_view_model), AsRoot)
        .map_err(convert)?;

    // Map message IDs to tree node IDs for threading
    let mut ids = HashMap::new();
    ids.insert(&root_message.id, root_id);

    // Build threaded tree from remaining messages
    for message in messages.iter() {
        // Skip root message (already inserted)
        if message.id == root_message.id {
            continue;
        }

        // Find parent node ID from in_reply_to relationship
        let Some(reply_to) = message.in_reply_to.as_ref() else {
            // No reply_to means this is orphaned - skip
            log::warn!("Orphaned message {:?} with no in_reply_to", message.id);
            continue;
        };

        let Some(parent_node_id) = ids.get(reply_to) else {
            log::error!("Could not resolve reply-to for message {:?}", message.id);
            continue;
        };

        let view_model = message_to_status_view_model(message);
        let Ok(child_id) = tree.insert(Node::new(view_model), UnderNode(parent_node_id)) else {
            log::error!("Could not insert message into tree {:?}", message.id);
            continue;
        };

        ids.insert(&message.id, child_id.clone());
    }

    let conv = Conversation {
        original_status: status_id,
        tree,
    };

    Ok(conv)
}

/// Transform Message to StatusViewModel for conversation threading
pub fn message_to_status_view_model(msg: &crate::view_model::message::Message) -> StatusViewModel {
    use crate::helper::clean_html;
    use crate::view_model::*;

    // Create synthetic account for message author
    let account = AccountViewModel {
        id: AccountId(msg.conversation_id.to_sql()),
        image: String::new(),
        image_header: String::new(),
        username: msg.author.clone(),
        display_name: msg.author.clone(),
        display_name_html: msg.author.clone(),
        acct: msg.author.clone(),
        note_plain: String::new(),
        note_html: Vec::new(),
        joined_human: String::new(),
        joined_full: String::new(),
        joined: *msg.timestamp,
        url: String::new(),
        followers: 0,
        followers_str: String::from("0"),
        following: 0,
        following_str: String::from("0"),
        statuses: 0,
        statuses_str: String::from("0"),
        header: String::new(),
        fields: Vec::new(),
        locked: false,
        bot: matches!(msg.author_type, message::AuthorType::Agent),
    };

    // Parse message content to HTML items
    let (_, content_html) = clean_html(&msg.content);

    StatusViewModel {
        id: StatusId(msg.id.to_sql()),
        uri: String::new(),
        account,
        status_images: Vec::new(),
        created: *msg.timestamp,
        created_human: String::new(),
        created_full: String::new(),
        reblog_status: None,
        content: content_html,
        card: None,
        replies: String::new(),
        replies_title: String::new(),
        replies_count: 0,
        is_reply: msg.in_reply_to.is_some(),
        in_reply_to_id: msg.in_reply_to.as_ref().map(|id| id.to_sql()),
        has_reblogged: false,
        is_reblog: false,
        reblog_count: 0,
        reblog: String::new(),
        reblog_title: String::new(),
        is_favourited: false,
        favourited: String::new(),
        favourited_count: 0,
        favourited_title: String::new(),
        bookmarked_title: if msg.pinned {
            String::from("Pinned")
        } else {
            String::new()
        },
        is_bookmarked: false,
        share_title: String::new(),
        mentions: Vec::new(),
        has_conversation: None,
        text: msg.content.clone(),
        media: Vec::new(),
        visibility: types::Visibility::Public,
        is_pinned: msg.pinned,
    }
}

#[allow(dead_code)] // Error conversion helper for conversation functionality
fn convert(value: NodeIdError) -> String {
    format!("{value:?}")
}
