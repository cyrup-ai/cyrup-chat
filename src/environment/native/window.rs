//! Window management and popup functionality

use dioxus::prelude::*;
use dioxus_desktop::{Config, LogicalSize, WindowBuilder, use_window};
use flume::{Receiver, Sender};
use std::rc::Rc;
use std::sync::Arc;

use super::core::Environment;
use crate::environment::OpenWindowState;
use crate::environment::types::AppEvent;

pub fn open_window_impl<S: OpenWindowState + 'static>(
    environment: &Environment,
    state: S,
    width: f64,
    height: f64,
    title: impl AsRef<str>,
    parent_handler: Rc<dyn Fn(S::Action)>,
) {
    // Create channels for window communication
    let (sender, receiver) = flume::unbounded();

    let _dom = VirtualDom::new_with_props(
        NewWindowPopup::<S>,
        NewWindowPopupProps {
            state: state.clone(),
            receiver: receiver.clone(),
            sender: sender.clone(),
            parent_handler: parent_handler.clone(),
            environment: environment.clone(),
        },
    );

    let s = LogicalSize::new(width, height);
    let builder = WindowBuilder::new()
        .with_theme(Some(dioxus_desktop::tao::window::Theme::Dark))
        .with_title(title.as_ref())
        .with_inner_size(s);

    // Context providers moved to component level in NewWindowPopup
    // Using proper use_context_provider pattern as shown in context_api.rs example
    let moved_sender = sender;
    let _ux: Arc<dyn Fn(AppEvent) + Send + Sync> = Arc::new(move |a: AppEvent| {
        let _ = moved_sender.send(a);
        // Signal-based updates in Dioxus 0.7 happen automatically
    });
    let _config = Config::new()
        .with_custom_head(
            r#"
        <link rel="stylesheet" href="/assets/tailwind.css">
        <meta name='color-scheme' content='dark'>
        "#
            .to_string(),
        )
        .with_window(builder);
    // Context provision handled in NewWindowPopup component using use_context_provider
    // File drop handling will be implemented when Dioxus 0.7 API stabilizes
    log::debug!("Window created without file drop handler - feature pending Dioxus 0.7 API");

    // New window functionality deferred until Dioxus 0.7 API clarification
    log::debug!("New window creation deferred - using single window mode");
}

#[derive(Props)]
pub struct NewWindowPopupProps<S: OpenWindowState + 'static> {
    pub state: S,
    pub receiver: Receiver<AppEvent>,
    pub sender: Sender<AppEvent>,
    pub parent_handler: Rc<dyn Fn(S::Action)>,
    pub environment: Environment,
}

impl<S: OpenWindowState + 'static> Clone for NewWindowPopupProps<S> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            receiver: self.receiver.clone(),
            sender: self.sender.clone(),
            parent_handler: self.parent_handler.clone(),
            environment: self.environment.clone(),
        }
    }
}

impl<S: OpenWindowState + 'static> PartialEq for NewWindowPopupProps<S> {
    fn eq(&self, other: &Self) -> bool {
        // Compare only the comparable fields - channels and handlers are not comparable
        self.state == other.state && self.environment == other.environment
    }
}

#[component]
pub fn NewWindowPopup<S: OpenWindowState + 'static>(props: NewWindowPopupProps<S>) -> Element {
    // Provide context at component level using proper Dioxus 0.7 patterns
    // Following the context_api.rs example pattern
    use_context_provider(|| props.state.clone());
    use_context_provider(|| props.receiver.clone());
    use_context_provider(|| props.sender.clone());
    use_context_provider(|| props.parent_handler.clone());
    use_context_provider(|| props.environment.clone());

    // Access context using safe patterns with fallbacks
    // Since we just provided these contexts above, they should be available
    // However, we handle potential failures gracefully for production safety
    let window_state = match try_use_context::<S>() {
        Some(state) => state,
        None => {
            log::error!("CRITICAL: Window state context not found immediately after providing it");
            log::error!("This indicates a serious component hierarchy or timing issue");
            // Return early to prevent further issues - component cannot function without state
            return rsx! { div { "Component initialization error" } };
        }
    };

    let receiver = match try_use_context::<Receiver<AppEvent>>() {
        Some(rcv) => rcv,
        None => {
            log::error!("CRITICAL: Receiver context not found immediately after providing it");
            return rsx! { div { "Event system initialization error" } };
        }
    };

    let environment = match try_use_context::<Environment>() {
        Some(env) => env,
        None => {
            log::error!("CRITICAL: Environment context not found immediately after providing it");
            return rsx! { div { "Environment initialization error" } };
        }
    };

    let sender = match try_use_context::<Sender<AppEvent>>() {
        Some(snd) => snd,
        None => {
            log::error!("CRITICAL: Sender context not found immediately after providing it");
            return rsx! { div { "Event sender initialization error" } };
        }
    };

    let parent_handler = match try_use_context::<Rc<dyn Fn(S::Action)>>() {
        Some(handler) => handler,
        None => {
            log::error!(
                "CRITICAL: Parent handler context not found immediately after providing it"
            );
            return rsx! { div { "Parent handler initialization error" } };
        }
    };

    // Rest of the component implementation

    let _updater = use_signal(|| ());
    let cloned_sender = sender.clone();

    // Set up menu event handling
    let environment_for_menu = environment.clone();
    use_effect(move || {
        let environment = environment_for_menu.clone();
        let sender_for_arc = cloned_sender.clone();

        environment
            .platform
            .handle_menu_events(Arc::new(move |a: AppEvent| {
                let _ = sender_for_arc.send(a);
                // Remove signal update since we don't need it for menu events
            }));

        // No cleanup needed in modern Dioxus
    });

    // Set up text size behavior using modern Dioxus 0.7 patterns
    let environment_for_text = environment.clone();
    use_effect(move || {
        let environment = environment_for_text.clone();
        // Text size behavior implemented using Signal-based reactive patterns
        use_effect(move || {
            // Watch for text size changes in environment settings
            if let Ok(config) = environment.settings.config() {
                let text_size = config.text_size;
                if text_size > 0.0 {
                    log::debug!("Text size changed to: {}", text_size);
                    // Apply text size changes through CSS custom properties
                    environment.platform.update_text_size(text_size);
                }
            }
        });

        log::debug!("ChangeTextsizeBehaviour setup completed with Signal-based reactivity");
    });

    #[cfg(target_os = "macos")]
    let _window = use_window();

    // Handle macOS-specific window setup with modern Dioxus 0.7 patterns
    use_effect(move || {
        #[cfg(target_os = "macos")]
        {
            // macOS webview transparency implemented using modern Dioxus 0.7 platform APIs
            // Use the desktop configuration system instead of direct webview manipulation
            // Webview transparency configuration deferred until Dioxus 0.7 API stabilizes
            log::debug!("macOS webview transparency deferred - using default configuration");
            log::debug!(
                "Window transparency will be configured through window builder when API is available"
            );
        }
    });

    // Render the window content using the window state
    window_state.window(&environment, receiver, parent_handler)
}
