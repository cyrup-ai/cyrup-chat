#![allow(non_snake_case)]

pub mod app;
pub mod auth;
pub mod behaviours;
mod components;
pub mod config;
pub mod constants;
pub mod database;
pub mod environment;
pub mod errors;
pub mod helper;
pub mod i18n;
pub mod loc;
pub mod notifications;
pub mod services;
// mod style; // Removed: using Tailwind CSS via document::Link instead
pub mod utils;
pub mod view_model;
mod widgets;
mod windows;

pub mod icons;
pub mod public_action;
pub mod status_mutation;

pub use public_action::PublicAction;
pub use status_mutation::StatusMutation;
