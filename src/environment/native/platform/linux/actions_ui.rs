use super::core::Platform;

impl Platform {
    // UI state management
    pub fn should_auto_reload(&self) -> Result<bool, String> {
        log::debug!("Checking auto-reload conditions on Linux");

        // Check network connectivity via NetworkManager DBus
        let network_available = self.check_network_connectivity();

        // Check if any window has focus
        let window_focused = self.check_window_focus();

        let should_reload = network_available && window_focused;

        log::debug!(
            "Auto-reload decision: network={}, focused={}, result={}",
            network_available,
            window_focused,
            should_reload
        );

        Ok(should_reload)
    }

    /// Check network connectivity via NetworkManager DBus interface
    fn check_network_connectivity(&self) -> bool {
        use std::process::Command;

        // Try NetworkManager first (most common)
        if let Ok(output) = Command::new("nmcli")
            .args(&["-t", "-f", "STATE", "general"])
            .output()
        {
            if let Ok(state) = String::from_utf8(output.stdout) {
                let connected = state.trim() == "connected";
                log::debug!("NetworkManager connectivity: {}", connected);
                return connected;
            }
        }

        // Fallback: Check via DBus directly
        if let Ok(output) = Command::new("gdbus")
            .args(&[
                "call",
                "--system",
                "--dest",
                "org.freedesktop.NetworkManager",
                "--object-path",
                "/org/freedesktop/NetworkManager",
                "--method",
                "org.freedesktop.DBus.Properties.Get",
                "org.freedesktop.NetworkManager",
                "Connectivity",
            ])
            .output()
        {
            if let Ok(result) = String::from_utf8(output.stdout) {
                // NetworkManager connectivity values: 1=none, 2=portal, 3=limited, 4=full
                let has_connectivity = result.contains("uint32 4") || result.contains("uint32 3");
                log::debug!("DBus NetworkManager connectivity: {}", has_connectivity);
                return has_connectivity;
            }
        }

        // Final fallback: Multiple ping tests with fallback IPs
        let fallback_ips = ["8.8.8.8", "1.1.1.1", "8.8.4.4"];
        let mut ping_result = false;

        for ip in &fallback_ips {
            if let Ok(output) = Command::new("ping")
                .args(&["-c", "1", "-W", "2", ip])
                .output()
            {
                if output.status.success() {
                    ping_result = true;
                    log::debug!("Ping connectivity test successful with IP: {}", ip);
                    break;
                }
            }
        }

        if !ping_result {
            log::debug!("All ping connectivity tests failed");
        }

        log::debug!("Ping connectivity test: {}", ping_result);
        ping_result
    }

    /// Check if any application window has focus using GTK4
    fn check_window_focus(&self) -> bool {
        use gtk4::prelude::*;

        // Check all toplevel GTK windows for focus
        let toplevels = gtk4::Window::toplevels();
        for i in 0..toplevels.n_items() {
            if let Some(window) = toplevels.item(i) {
                if let Ok(gtk_window) = window.downcast::<gtk4::Window>() {
                    if gtk_window.is_active() {
                        log::debug!("Found focused GTK4 window");
                        return true;
                    }
                }
            }
        }

        // Fallback: Check via X11 if available
        if let Ok(output) = std::process::Command::new("xprop")
            .args(&["-root", "_NET_ACTIVE_WINDOW"])
            .output()
        {
            if let Ok(result) = String::from_utf8(output.stdout) {
                let has_active_window =
                    !result.contains("not found") && result.contains("window id");
                log::debug!("X11 active window check: {}", has_active_window);
                return has_active_window;
            }
        }

        log::debug!("No focused windows detected, defaulting to true");
        true // Default to true if we can't determine focus
    }

    pub fn update_sidebar_visibility(&self, visible: bool) {
        use gtk4::prelude::*;

        log::debug!("Updating sidebar visibility on Linux: {}", visible);

        // Find GTK windows and update sidebar visibility
        let toplevels = gtk4::Window::toplevels();
        for i in 0..toplevels.n_items() {
            if let Some(gtk_window) = toplevels
                .item(i)
                .and_then(|w| w.downcast::<gtk4::Window>().ok())
            {
                // Recursively find sidebar widgets by CSS class
                fn find_sidebar_widgets(widget: &gtk4::Widget) -> Vec<gtk4::Widget> {
                    let mut sidebars = Vec::new();

                    if widget.has_css_class("sidebar")
                        || widget.has_css_class("side-panel")
                        || widget.has_css_class("navigation-sidebar")
                    {
                        sidebars.push(widget.clone());
                    }

                    // Check children recursively
                    let mut child = widget.first_child();
                    while let Some(child_widget) = child {
                        sidebars.extend(find_sidebar_widgets(&child_widget));
                        child = child_widget.next_sibling();
                    }

                    sidebars
                }

                if let Some(root_widget) = gtk_window.child() {
                    let sidebar_widgets = find_sidebar_widgets(&root_widget);

                    for sidebar in sidebar_widgets {
                        sidebar.set_visible(visible);

                        // Also update CSS classes for styling
                        if visible {
                            sidebar.add_css_class("sidebar-visible");
                            sidebar.remove_css_class("sidebar-hidden");
                        } else {
                            sidebar.add_css_class("sidebar-hidden");
                            sidebar.remove_css_class("sidebar-visible");
                        }
                    }

                    log::info!(
                        "Updated {} sidebar widgets visibility to: {}",
                        sidebar_widgets.len(),
                        visible
                    );
                    return;
                }
            }
        }

        log::debug!("No GTK4 windows with sidebar widgets found");
    }

    pub fn set_text_size(&self, size: f32) {
        use gtk4::prelude::*;

        log::debug!("Setting text size on Linux: {}", size);
        self.apply_text_size_css(size)
    }

    pub fn update_text_size(&self, size: f32) {
        use gtk4::prelude::*;

        log::debug!("Updating text size on Linux: {}", size);
        self.apply_text_size_css(size)
    }

    fn apply_text_size_css(&self, size: f32) {
        use gtk4::prelude::*;
        use gtk4::{CssProvider, gdk};

        // Create CSS provider for text size styling
        let css_provider = CssProvider::new();

        // Generate CSS with the specified font size
        let text_size_css = format!(
            "
            * {{
                font-size: {}pt;
            }}
            
            .text-content, .message-text, .chat-text {{
                font-size: {}pt;
            }}
            
            .small-text {{
                font-size: {}pt;
            }}
            
            .large-text {{
                font-size: {}pt;
            }}
            ",
            size,
            size,
            size * 0.8,
            size * 1.2
        );

        css_provider.load_from_string(&text_size_css);

        // Apply CSS to the default display
        if let Some(display) = gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &css_provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 1, // Higher priority for text size
            );

            log::info!("Applied text size {}pt CSS to GTK4 display", size);
        } else {
            log::warn!("Could not get default GTK4 display for text size update");
        }
    }

    // Public action handling
    pub async fn handle_public_action(
        &self,
        action: crate::PublicAction,
        instance_url: &str,
    ) -> Result<(), String> {
        log::debug!("Handling public action on Linux: {:?}", action);

        match action {
            crate::PublicAction::OpenLink(url) => {
                self.navigate(&url)?;
            }
            crate::PublicAction::Copy(text) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    use copypasta::{ClipboardContext, ClipboardProvider};
                    if let Ok(mut ctx) = ClipboardContext::new() {
                        let _ = ctx.set_contents(text);
                    }
                }
            }
            crate::PublicAction::OpenProfile(profile) => {
                log::debug!("Opening profile on Linux: {:?}", profile.username);

                // Construct profile URL if available
                if let Some(url) = profile.web_url.as_ref() {
                    match self.navigate(url) {
                        Ok(()) => {
                            log::info!("Successfully opened profile URL: {}", url);
                        }
                        Err(e) => {
                            log::error!("Failed to open profile URL {}: {}", url, e);
                        }
                    }
                } else {
                    log::warn!(
                        "Profile {} has no web URL - cannot open in browser",
                        profile.username
                    );
                }
            }
            crate::PublicAction::OpenTag(tag) => {
                log::debug!("Opening tag on Linux: {}", tag);

                // Construct tag URL using current instance URL
                let tag_url = format!("{}/tags/{}", instance_url, tag);
                match self.navigate(&tag_url) {
                    Ok(()) => {
                        log::info!("Successfully opened tag URL: {}", tag_url);
                    }
                    Err(e) => {
                        log::error!("Failed to open tag URL {}: {}", tag_url, e);
                    }
                }
            }
            _ => {
                log::debug!("Unhandled public action on Linux: {:?}", action);
            }
        }

        Ok(())
    }

    /// Show emoji popup using GTK4 EmojiChooser
    pub fn show_emoji_popup(&self) -> Result<(), String> {
        use gtk4::prelude::*;

        log::debug!("Showing emoji popup using GTK4 EmojiChooser");

        // Find a GTK window to attach the emoji chooser to
        let toplevels = gtk4::Window::toplevels();
        for i in 0..toplevels.n_items() {
            if let Some(gtk_window) = toplevels
                .item(i)
                .and_then(|w| w.downcast::<gtk4::Window>().ok())
            {
                // Create EmojiChooser
                let emoji_chooser = gtk4::EmojiChooser::new();
                emoji_chooser.set_parent(&gtk_window);

                // Connect emoji selection signal
                emoji_chooser.connect_emoji_picked(|_, emoji| {
                    log::info!("Emoji selected: {}", emoji);
                    // Copy emoji to clipboard
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        use copypasta::{ClipboardContext, ClipboardProvider};
                        if let Ok(mut ctx) = ClipboardContext::new() {
                            let _ = ctx.set_contents(emoji.to_string());
                        }
                    }
                });

                // Show the emoji chooser
                emoji_chooser.popup();

                log::info!("GTK4 EmojiChooser displayed");
                return Ok(());
            }
        }

        Err("No GTK4 window found to attach emoji chooser".to_string())
    }

    /// Show a context menu with the given items and coordinates using GTK4 PopoverMenu
    pub fn show_context_menu<T: Clone + std::fmt::Debug + Send + 'static>(
        &self,
        coordinates: (i32, i32),
        title: &str,
        items: Vec<(&str, T)>,
        callback: impl Fn(T) + 'static,
    ) -> Result<(), String> {
        use gtk4::prelude::*;
        use std::cell::RefCell;
        use std::rc::Rc;

        log::debug!(
            "Creating GTK4 context menu at {:?}: {} with {} items",
            coordinates,
            title,
            items.len()
        );

        // Find a GTK window to attach the popover to
        let toplevels = gtk4::Window::toplevels();
        for i in 0..toplevels.n_items() {
            if let Some(gtk_window) = toplevels
                .item(i)
                .and_then(|w| w.downcast::<gtk4::Window>().ok())
            {
                // Create menu model
                let menu_model = gtk4::gio::Menu::new();

                // Add title section if provided
                if !title.is_empty() {
                    let title_section = gtk4::gio::Menu::new();
                    title_section.append(Some(title), None);
                    menu_model.append_section(None, &title_section);
                }

                // Add menu items
                let callback_rc = Rc::new(RefCell::new(Some(callback)));
                for (i, (label, value)) in items.iter().enumerate() {
                    let action_name = format!("context_menu_item_{}", i);
                    menu_model.append(Some(label), Some(&format!("app.{}", action_name)));

                    // Create action for this menu item
                    let action = gtk4::gio::SimpleAction::new(&action_name, None);
                    let value_clone = value.clone();
                    let callback_clone = callback_rc.clone();

                    action.connect_activate(move |_, _| {
                        if let Some(cb) = callback_clone.borrow_mut().take() {
                            cb(value_clone.clone());
                        }
                    });

                    // Add action to application
                    if let Some(app) = gtk4::gio::Application::default() {
                        app.add_action(&action);
                    }
                }

                // Create PopoverMenu from model
                let popover_menu = gtk4::PopoverMenu::from_model(Some(&menu_model));
                popover_menu.set_parent(&gtk_window);

                // Set position based on coordinates
                let rect = gtk4::gdk::Rectangle::new(coordinates.0, coordinates.1, 1, 1);
                popover_menu.set_pointing_to(Some(&rect));

                // Show the context menu
                popover_menu.popup();

                log::info!(
                    "GTK4 PopoverMenu context menu displayed with {} items",
                    items.len()
                );
                return Ok(());
            }
        }

        Err("No GTK4 window found to attach context menu".to_string())
    }
}
