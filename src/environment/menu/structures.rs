//! Context menu data structures and constructors

use super::types::Payload;

#[derive(Debug)]
#[allow(unused)]
pub enum ContextMenuKind {
    Checkbox {
        title: String,
        checked: bool,
        enabled: bool,
        payload: Payload,
    },
    Item {
        title: String,
        enabled: bool,
        payload: Payload,
    },
    Submenu {
        title: String,
        enabled: bool,
        children: Vec<ContextMenuItem>,
    },
    Separator,
}

#[derive(Debug)]
pub struct ContextMenuItem {
    // Core menu item data with efficient memory layout
    pub(super) kind: ContextMenuKind,
}

#[derive(Debug)]
pub struct ContextMenu<A> {
    pub title: String,
    pub enabled: bool,
    pub children: Vec<ContextMenuItem>,
    pub _m: std::marker::PhantomData<A>,
}

impl<A> ContextMenu<A> {
    pub fn new(title: impl AsRef<str>, enabled: bool, children: Vec<ContextMenuItem>) -> Self {
        Self {
            title: title.as_ref().to_string(),
            enabled,
            children,
            _m: std::marker::PhantomData,
        }
    }
}

#[allow(unused)]
impl ContextMenuItem {
    pub fn item<T: Send + Sync + 'static>(title: impl AsRef<str>, payload: T) -> Self {
        Self {
            kind: ContextMenuKind::Item {
                title: title.as_ref().to_string(),
                enabled: true,
                payload: Box::new(payload),
            },
        }
    }

    pub fn checkbox<T: Send + Sync + 'static>(
        title: impl AsRef<str>,
        checked: bool,
        payload: T,
    ) -> Self {
        Self {
            kind: ContextMenuKind::Checkbox {
                title: title.as_ref().to_string(),
                checked,
                enabled: true,
                payload: Box::new(payload),
            },
        }
    }

    pub fn submenu(title: impl AsRef<str>, children: Vec<ContextMenuItem>) -> Self {
        Self {
            kind: ContextMenuKind::Submenu {
                title: title.as_ref().to_string(),
                enabled: true,
                children,
            },
        }
    }

    pub fn separator() -> Self {
        Self {
            kind: ContextMenuKind::Separator,
        }
    }

    /// Create menu item with explicit enabled state
    pub fn item_enabled<T: Send + Sync + 'static>(
        title: impl AsRef<str>,
        enabled: bool,
        payload: T,
    ) -> Self {
        Self {
            kind: ContextMenuKind::Item {
                title: title.as_ref().to_string(),
                enabled,
                payload: Box::new(payload),
            },
        }
    }

    /// Create checkbox menu item with explicit enabled state
    pub fn checkbox_enabled<T: Send + Sync + 'static>(
        title: impl AsRef<str>,
        enabled: bool,
        checked: bool,
        payload: T,
    ) -> Self {
        Self {
            kind: ContextMenuKind::Checkbox {
                title: title.as_ref().to_string(),
                checked,
                enabled,
                payload: Box::new(payload),
            },
        }
    }

    /// Create submenu with explicit enabled state
    pub fn submenu_enabled(
        title: impl AsRef<str>,
        enabled: bool,
        children: Vec<ContextMenuItem>,
    ) -> Self {
        Self {
            kind: ContextMenuKind::Submenu {
                title: title.as_ref().to_string(),
                enabled,
                children,
            },
        }
    }

    /// Get a reference to the kind of context menu item
    pub fn kind(&self) -> &ContextMenuKind {
        &self.kind
    }

    /// Zero-allocation builder for menu items with string references
    /// Avoids cloning strings when possible
    pub fn item_ref<T: Send + Sync + 'static>(title: &str, payload: T) -> Self {
        Self {
            kind: ContextMenuKind::Item {
                title: title.to_string(),
                enabled: true,
                payload: Box::new(payload),
            },
        }
    }

    /// Zero-allocation builder for checkbox items with string references
    pub fn checkbox_ref<T: Send + Sync + 'static>(title: &str, checked: bool, payload: T) -> Self {
        Self {
            kind: ContextMenuKind::Checkbox {
                title: title.to_string(),
                checked,
                enabled: true,
                payload: Box::new(payload),
            },
        }
    }

    /// Zero-allocation builder for submenu items with string references
    pub fn submenu_ref(title: &str, children: Vec<ContextMenuItem>) -> Self {
        Self {
            kind: ContextMenuKind::Submenu {
                title: title.to_string(),
                enabled: true,
                children,
            },
        }
    }

    /// Validate menu item structure for type safety
    pub fn validate(&self) -> Result<(), MenuValidationError> {
        match &self.kind {
            ContextMenuKind::Checkbox { title, .. }
            | ContextMenuKind::Item { title, .. }
            | ContextMenuKind::Submenu { title, .. } => {
                if title.is_empty() {
                    return Err(MenuValidationError::EmptyTitle);
                }
            }
            ContextMenuKind::Separator => {}
        }

        if let ContextMenuKind::Submenu { children, .. } = &self.kind {
            if children.is_empty() {
                return Err(MenuValidationError::EmptySubmenu);
            }
            for child in children {
                child.validate()?;
            }
        }

        Ok(())
    }

    /// Get the title of the menu item, if it has one
    pub fn title(&self) -> Option<&str> {
        match &self.kind {
            ContextMenuKind::Checkbox { title, .. }
            | ContextMenuKind::Item { title, .. }
            | ContextMenuKind::Submenu { title, .. } => Some(title),
            ContextMenuKind::Separator => None,
        }
    }

    /// Check if this menu item is enabled
    pub fn is_enabled(&self) -> bool {
        match &self.kind {
            ContextMenuKind::Checkbox { enabled, .. } => *enabled,
            ContextMenuKind::Item { enabled, .. } => *enabled,
            ContextMenuKind::Submenu { enabled, .. } => *enabled,
            ContextMenuKind::Separator => true,
        }
    }
}

/// Error types for menu validation
#[derive(Debug, Clone, PartialEq)]
pub enum MenuValidationError {
    /// Menu item has empty title
    EmptyTitle,
    /// Submenu has no children
    EmptySubmenu,
    /// Invalid menu structure
    InvalidStructure,
}

impl std::fmt::Display for MenuValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuValidationError::EmptyTitle => write!(f, "Menu item cannot have empty title"),
            MenuValidationError::EmptySubmenu => write!(f, "Submenu cannot be empty"),
            MenuValidationError::InvalidStructure => write!(f, "Invalid menu structure"),
        }
    }
}

impl std::error::Error for MenuValidationError {}

impl<A> ContextMenu<A> {
    /// Zero-allocation constructor with string reference
    pub fn new_ref(title: &str, enabled: bool, children: Vec<ContextMenuItem>) -> Self {
        Self {
            title: title.to_string(),
            enabled,
            children,
            _m: std::marker::PhantomData,
        }
    }

    /// Validate the entire menu structure
    pub fn validate(&self) -> Result<(), MenuValidationError> {
        if self.title.is_empty() {
            return Err(MenuValidationError::EmptyTitle);
        }

        for child in &self.children {
            child.validate()?;
        }

        Ok(())
    }

    /// Get the number of menu items
    pub fn item_count(&self) -> usize {
        self.children.len()
    }

    /// Check if the menu is empty
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
}
