//! Sidebar more component
//!
//! This module provides the "More" section of the sidebar with additional
//! timeline and account options.

use super::super::reducer::{MoreSelection, SidebarAction, SidebarSignal};
use crate::{loc, widgets::*};
use dioxus::prelude::*;

#[allow(dead_code)] // Sidebar more menu entries - pending integration
enum SidebarMoreEntry {
    More(MoreSelection),
    Title(String),
}

#[component]
pub fn SidebarMoreComponent(store: SidebarSignal) -> Element {
    let selection = store.read().more_selection;
    let structure = {
        use MoreSelection::*;
        use SidebarMoreEntry::*;
        vec![
            Title(loc!("Timelines").to_string()),
            More(Classic),
            More(Local),
            More(Federated),
            //Title(loc!("Explore")),
            //More(Posts),
            //More(Hashtags),
            Title(loc!("Account").to_string()),
            More(Yours),
            More(Followers),
            More(Following),
            More(Bookmarks),
            More(Favorites),
        ]
    };

    rsx! {
        div { class: "scroll",
            div {
                class: "scroll-margin-fix",
                VStack {
                    class: "p-3 gap-2",

                    for entry in structure {
                        match entry {
                            SidebarMoreEntry::Title(t) => rsx!(SidebarTextHeadline {
                                text: t,
                            }),
                            SidebarMoreEntry::More(m) => rsx!(SidebarTextEntry {
                                icon: m.content(),
                                text: m.title(),
                                selected: selection == m,
                                onclick: move |_| {
                                    use crate::components::sidebar::handle_action;
                                    use crate::app::use_environment;
                                    let environment = use_environment();
                                    handle_action(store, SidebarAction::More(m), &environment.read());
                                }
                            })
                        }
                    }
                }
            }
        }
    }
}

impl MoreSelection {
    #[allow(dead_code)] // More selection content mapping - pending UI integration
    pub fn content(&self) -> &str {
        match self {
            Self::Classic => "􀭞",   // square.fill.text.grid.1x2
            Self::Yours => "􀈎",     // square.and.pencil
            Self::Local => "􀝋",     // person.3.fill
            Self::Federated => "􀆪", // globe
            Self::Posts => "􀌪",     // bubble.left
            Self::Hashtags => "􀋡",  // tag
            Self::Followers => "􀉬", // person.2.fill
            Self::Following => "􀉫", // person.2
            Self::Bookmarks => "􀼺", // bookmark.square.fill
            Self::Favorites => "􀠨", // star.square.fill
        }
    }

    #[allow(dead_code)] // More selection title mapping - pending UI integration
    pub fn title(&self) -> String {
        match self {
            MoreSelection::Classic => loc!("Classic Timeline"),
            MoreSelection::Yours => loc!("Your Posts"),
            MoreSelection::Local => loc!("Local"),
            MoreSelection::Federated => loc!("Federated"),
            MoreSelection::Posts => loc!("Posts"),
            MoreSelection::Hashtags => loc!("Hashtags"),
            MoreSelection::Followers => loc!("Followers"),
            MoreSelection::Following => loc!("Following"),
            MoreSelection::Bookmarks => loc!("Bookmarks"),
            MoreSelection::Favorites => loc!("Favorites"),
        }
    }
}
