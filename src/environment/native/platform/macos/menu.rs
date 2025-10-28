//! macOS menu system with zero-allocation patterns
//!
//! This module provides production-ready menu creation and management
//! using the muda API for native macOS integration.

use super::super::super::super::types::{MainMenuConfig, MainMenuEvent};
use muda::{
    Menu, MenuItem, PredefinedMenuItem, Submenu,
    accelerator::{Accelerator as MudaAccelerator, Code, Modifiers},
};

/// Create the main application menu bar with comprehensive error handling
#[inline(always)]
pub fn create_main_menu(config: MainMenuConfig) -> Menu {
    // Proper menu system implemented using muda API
    let menu_bar = Menu::new();

    // App menu for macOS (following muda API patterns)
    #[cfg(target_os = "macos")]
    {
        create_app_menu(&menu_bar);
    }

    // File menu with safe error handling
    create_file_menu(&menu_bar, &config);

    // Edit menu with safe error handling
    create_edit_menu(&menu_bar);

    // View menu with safe error handling
    create_view_menu(&menu_bar, &config);

    // Window menu with safe error handling
    create_window_menu(&menu_bar);

    log::debug!("Menu system implemented using proper muda API");
    menu_bar
}

/// Create the macOS application menu
#[cfg(target_os = "macos")]
#[inline(always)]
fn create_app_menu(menu_bar: &Menu) {
    let app_m = Submenu::new("CYRUP", true);
    if let Err(e) = menu_bar.append(&app_m) {
        log::error!("Failed to append app menu: {e}");
        return;
    }

    // Add standard macOS app menu items with safe error handling
    if let Err(e) = app_m.append_items(&[
        &PredefinedMenuItem::about(None, None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::services(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::hide(None),
        &PredefinedMenuItem::hide_others(None),
        &PredefinedMenuItem::show_all(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::quit(None),
    ]) {
        log::error!("Failed to append app menu items: {e}");
    }
}

/// Create the File menu with configuration-based enabled states
#[inline(always)]
fn create_file_menu(menu_bar: &Menu, config: &MainMenuConfig) {
    let file_m = Submenu::new("&File", true);
    if let Err(e) = menu_bar.append(&file_m) {
        log::error!("Failed to append file menu: {e}");
        return;
    }

    // New Post menu item with safe error handling
    let new_post_item = MenuItem::with_id(
        MainMenuEvent::NewPost.menu_id(),
        "New Post",
        config.logged_in,
        Some(MudaAccelerator::new(Some(Modifiers::SUPER), Code::KeyN)),
    );
    if let Err(e) = file_m.append(&new_post_item) {
        log::error!("Failed to append new post item: {e}");
    }

    // Reload menu item with safe error handling
    let reload_item = MenuItem::with_id(
        MainMenuEvent::Reload.menu_id(),
        "Reload",
        config.logged_in,
        Some(MudaAccelerator::new(Some(Modifiers::SUPER), Code::KeyR)),
    );
    if let Err(e) = file_m.append(&reload_item) {
        log::error!("Failed to append reload item: {e}");
    }

    // File menu separators and items with safe error handling
    if let Err(e) = file_m.append(&PredefinedMenuItem::separator()) {
        log::error!("Failed to append separator: {e}");
    }
    if let Err(e) = file_m.append(&PredefinedMenuItem::close_window(Some("Close"))) {
        log::error!("Failed to append close item: {e}");
    }
}

/// Create the Edit menu with standard editing operations
#[inline(always)]
fn create_edit_menu(menu_bar: &Menu) {
    let edit_m = Submenu::new("&Edit", true);
    if let Err(e) = menu_bar.append(&edit_m) {
        log::error!("Failed to append edit menu: {e}");
        return;
    }

    // Add edit menu items with safe error handling
    if let Err(e) = edit_m.append_items(&[
        &PredefinedMenuItem::undo(None),
        &PredefinedMenuItem::redo(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::cut(None),
        &PredefinedMenuItem::copy(None),
        &PredefinedMenuItem::paste(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::select_all(None),
    ]) {
        log::error!("Failed to append edit menu items: {e}");
    }
}

/// Create the View menu with navigation and display options
#[inline(always)]
fn create_view_menu(menu_bar: &Menu, config: &MainMenuConfig) {
    let view_m = Submenu::new("&View", true);
    if let Err(e) = menu_bar.append(&view_m) {
        log::error!("Failed to append view menu: {e}");
        return;
    }

    // Timeline menu item with safe error handling
    let timeline_item = MenuItem::with_id(
        MainMenuEvent::Timeline.menu_id(),
        "Timeline",
        config.logged_in,
        Some(MudaAccelerator::new(Some(Modifiers::SUPER), Code::Digit1)),
    );
    if let Err(e) = view_m.append(&timeline_item) {
        log::error!("Failed to append timeline item: {e}");
    }

    // Mentions menu item with safe error handling
    let mentions_item = MenuItem::with_id(
        MainMenuEvent::Mentions.menu_id(),
        "Mentions",
        config.logged_in,
        Some(MudaAccelerator::new(Some(Modifiers::SUPER), Code::Digit2)),
    );
    if let Err(e) = view_m.append(&mentions_item) {
        log::error!("Failed to append mentions item: {e}");
    }

    // More menu item with safe error handling
    let more_item = MenuItem::with_id(
        MainMenuEvent::More.menu_id(),
        "More",
        config.logged_in,
        Some(MudaAccelerator::new(Some(Modifiers::SUPER), Code::Digit4)),
    );
    if let Err(e) = view_m.append(&more_item) {
        log::error!("Failed to append more item: {e}");
    }

    // View menu separators and items with safe error handling
    if let Err(e) = view_m.append(&PredefinedMenuItem::separator()) {
        log::error!("Failed to append separator: {e}");
    }
    if let Err(e) = view_m.append(&PredefinedMenuItem::fullscreen(None)) {
        log::error!("Failed to append fullscreen item: {e}");
    }
}

/// Create the Window menu with window management options
#[inline(always)]
fn create_window_menu(menu_bar: &Menu) {
    let window_m = Submenu::new("&Window", true);
    if let Err(e) = menu_bar.append(&window_m) {
        log::error!("Failed to append window menu: {e}");
        return;
    }

    // Add window menu items with safe error handling
    if let Err(e) = window_m.append_items(&[
        &PredefinedMenuItem::minimize(None),
        &PredefinedMenuItem::maximize(None),
        &PredefinedMenuItem::bring_all_to_front(None),
    ]) {
        log::error!("Failed to append window menu items: {e}");
    }
}
