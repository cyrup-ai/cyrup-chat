// Utility Modules - Production Implementation

pub mod async_file_dialog;
pub mod async_platform;
pub mod datetime;
pub mod http_parser;
pub mod id_generator;
pub mod platform_api;
pub mod scope_manager;

pub use datetime::*;

pub use id_generator::{
    generate_conversation_id, generate_session_id, generate_subscription_id, generate_timestamp_id,
    generate_uuid,
};

pub use scope_manager::{
    ScopeUpdateError, ScopeUpdateManager, get_global_scope_manager, initialize_scope_handling,
    send_global_scope_event,
};

pub use http_parser::{
    LinkEntry, LinkParseError, extract_max_id_from_link_header, parse_link_header,
};

pub use platform_api::{
    DesktopPlatformAPI, PlatformAPI, PlatformFeature, WebPlatformAPI, create_platform_api,
};

pub use async_platform::{AsyncPlatform, CursorPosition, TextAreaConfig};

pub use async_file_dialog::{AsyncFileDialog, FileDialogConfig, FileDialogResult, FileFilter};
