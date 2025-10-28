use super::core::{AppWindow, Platform};
use std::sync::Arc;

use crate::environment::{
    storage::UiTab,
    types::{ActionFromEvent, AppEvent, MainMenuConfig},
};

impl Platform {
    /// Setup Linux-specific toolbar with GTK integration
    pub fn setup_toolbar(&self, window: &AppWindow) {
        use gtk4::prelude::*;

        log::debug!("Setting up Linux toolbar for window: {}", window.window_id);

        // Find the GTK window and set up HeaderBar
        let toplevels = gtk4::Window::toplevels();
        for i in 0..toplevels.n_items() {
            if let Some(gtk_window) = toplevels
                .item(i)
                .and_then(|w| w.downcast::<gtk4::Window>().ok())
            {
                // Create HeaderBar for modern GNOME integration
                let header_bar = gtk4::HeaderBar::new();
                header_bar.set_show_title_buttons(true);

                // Set window title
                let title_label = gtk4::Label::new(Some(&window.title));
                title_label.add_css_class("title");
                header_bar.set_title_widget(Some(&title_label));

                // Set HeaderBar as titlebar
                gtk_window.set_titlebar(Some(&header_bar));

                log::info!("GTK4 HeaderBar configured for window: {}", window.title);
                return;
            }
        }

        log::warn!("No GTK4 window found for toolbar setup");
    }

    /// Update menu configuration with Linux-specific menu structure
    pub fn update_menu<'a>(&self, window: &AppWindow, mutator: impl Fn(&mut MainMenuConfig)) {
        use gtk4::prelude::*;

        log::debug!("Updating Linux menu for window: {}", window.window_id);

        // Apply the menu configuration changes
        let mut config = MainMenuConfig::default();
        mutator(&mut config);

        // Find the GTK window and update its menu
        let toplevels = gtk4::Window::toplevels();
        for i in 0..toplevels.n_items() {
            if let Some(gtk_window) = toplevels
                .item(i)
                .and_then(|w| w.downcast::<gtk4::Window>().ok())
            {
                // Create GMenuModel for GNOME integration
                let menu_model = gtk4::gio::Menu::new();

                // Add File menu section
                let file_section = gtk4::gio::Menu::new();
                file_section.append(Some("New"), Some("app.new"));
                file_section.append(Some("Open"), Some("app.open"));
                file_section.append(Some("Save"), Some("app.save"));
                menu_model.append_section(Some("File"), &file_section);

                // Add Edit menu section
                let edit_section = gtk4::gio::Menu::new();
                edit_section.append(Some("Cut"), Some("app.cut"));
                edit_section.append(Some("Copy"), Some("app.copy"));
                edit_section.append(Some("Paste"), Some("app.paste"));
                menu_model.append_section(Some("Edit"), &edit_section);

                // Set application menu
                if let Some(app) = gtk4::gio::Application::default() {
                    app.set_menubar(Some(&menu_model));
                }

                log::info!("GTK4 GMenuModel updated for window: {}", window.title);
                return;
            }
        }

        log::warn!("No GTK4 window found for menu update");
    }

    /// Update toolbar state based on current account and tab
    pub fn update_toolbar(&self, account: &str, tab: &UiTab, has_notifications: bool) {
        use gtk4::prelude::*;

        log::debug!(
            "Updating Linux toolbar - Account: {}, Tab: {:?}, Notifications: {}",
            account,
            tab,
            has_notifications
        );

        // Find GTK window and update HeaderBar
        let toplevels = gtk4::Window::toplevels();
        for i in 0..toplevels.n_items() {
            if let Some(gtk_window) = toplevels
                .item(i)
                .and_then(|w| w.downcast::<gtk4::Window>().ok())
            {
                if let Some(titlebar) = gtk_window
                    .titlebar()
                    .and_then(|t| t.downcast::<gtk4::HeaderBar>().ok())
                {
                    // Update title with account info
                    let title_text = format!("{} - {}", account, tab.to_string());
                    if let Some(title_widget) = titlebar
                        .title_widget()
                        .and_then(|w| w.downcast::<gtk4::Label>().ok())
                    {
                        title_widget.set_text(&title_text);
                    }

                    // Update notification badge
                    if has_notifications {
                        titlebar.add_css_class("has-notifications");

                        // Create notification badge button
                        let badge_button = gtk4::Button::new();
                        badge_button.set_icon_name("notification-symbolic");
                        badge_button.add_css_class("notification-badge");
                        titlebar.pack_end(&badge_button);
                    } else {
                        titlebar.remove_css_class("has-notifications");
                    }

                    log::info!("HeaderBar updated for account: {}, tab: {:?}", account, tab);
                    return;
                }
            }
        }

        log::warn!("No GTK4 HeaderBar found for toolbar update");
    }

    /// Handle menu events with Linux-specific event processing
    pub fn handle_menu_events<A: ActionFromEvent + 'static>(
        &self,
        updater: Arc<dyn Fn(A) + Send + Sync>,
    ) {
        use gtk4::prelude::*;

        log::debug!("Setting up Linux menu event handling");

        // Get the default GApplication and set up action handlers
        if let Some(app) = gtk4::gio::Application::default() {
            // Create GSimpleAction instances for menu items
            let new_action = gtk4::gio::SimpleAction::new("new", None);
            let open_action = gtk4::gio::SimpleAction::new("open", None);
            let save_action = gtk4::gio::SimpleAction::new("save", None);
            let cut_action = gtk4::gio::SimpleAction::new("cut", None);
            let copy_action = gtk4::gio::SimpleAction::new("copy", None);
            let paste_action = gtk4::gio::SimpleAction::new("paste", None);

            // Connect action signals to updater
            let updater_clone = updater.clone();
            new_action.connect_activate(move |_, _| {
                if let Some(action) = A::from_event(&AppEvent::MenuAction("new".to_string())) {
                    updater_clone(action);
                }
            });

            let updater_clone = updater.clone();
            open_action.connect_activate(move |_, _| {
                if let Some(action) = A::from_event(&AppEvent::MenuAction("open".to_string())) {
                    updater_clone(action);
                }
            });

            // Add actions to application
            app.add_action(&new_action);
            app.add_action(&open_action);
            app.add_action(&save_action);
            app.add_action(&cut_action);
            app.add_action(&copy_action);
            app.add_action(&paste_action);

            log::info!("GTK4 GAction handlers configured");
        } else {
            log::warn!("No GTK4 Application found for menu event handling");
        }
    }

    /// Set toolbar event handler for Linux platform
    pub fn set_toolbar_handler(&self, handler: Arc<dyn Fn(AppEvent) + Send + Sync>) {
        let mut handlers = self.menu_handlers.lock().unwrap_or_else(|e| {
            log::error!("Failed to lock menu handlers: {e}");
            panic!("Menu handler lock poisoned");
        });
        handlers.push(handler);
        log::debug!("Linux toolbar handler registered");
    }

    /// Configure toolbar for logged out state
    pub fn loggedout_toolbar(&self, window: &AppWindow) {
        use gtk4::prelude::*;

        log::debug!(
            "Configuring logged out toolbar for window: {}",
            window.window_id
        );

        // Find GTK window and configure HeaderBar for logged out state
        let toplevels = gtk4::Window::toplevels();
        for i in 0..toplevels.n_items() {
            if let Some(gtk_window) = toplevels
                .item(i)
                .and_then(|w| w.downcast::<gtk4::Window>().ok())
            {
                if let Some(titlebar) = gtk_window
                    .titlebar()
                    .and_then(|t| t.downcast::<gtk4::HeaderBar>().ok())
                {
                    // Update title for logged out state
                    let title_text = "Cyrup Chat - Please Log In";
                    if let Some(title_widget) = titlebar
                        .title_widget()
                        .and_then(|w| w.downcast::<gtk4::Label>().ok())
                    {
                        title_widget.set_text(title_text);
                    }

                    // Remove notification indicators
                    titlebar.remove_css_class("has-notifications");

                    // Clear any notification badges
                    let mut child = titlebar.first_child();
                    while let Some(widget) = child {
                        let next = widget.next_sibling();
                        if widget.has_css_class("notification-badge") {
                            titlebar.remove(&widget);
                        }
                        child = next;
                    }

                    // Add login button
                    let login_button = gtk4::Button::with_label("Login");
                    login_button.add_css_class("suggested-action");
                    titlebar.pack_end(&login_button);

                    log::info!("HeaderBar configured for logged out state");
                    return;
                }
            }
        }

        log::warn!("No GTK4 HeaderBar found for logged out toolbar configuration");
    }
}
