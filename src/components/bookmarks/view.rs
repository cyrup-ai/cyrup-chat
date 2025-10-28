use crate::components::status_timeline::{
    AnyTimelineProvider, BookmarkTimelineProvider, 
    State as TimelineState, TimelineComponent,
    handle_action, Action
};
use crate::environment::Environment;
use dioxus::prelude::*;

#[component]
pub fn BookmarksView() -> Element {
    let environment = use_context::<Environment>();
    
    let timeline_signal = use_signal(|| {
        let provider = BookmarkTimelineProvider::new(environment.clone());
        let any_provider = AnyTimelineProvider::new(provider, &"bookmarks");
        
        let ui_settings = match environment.settings.config() {
            Ok(settings) => settings,
            Err(e) => {
                log::error!("[BookmarksView] Failed to load UI settings: {}", e);
                Default::default()
            }
        };
        
        TimelineState::new(any_provider, ui_settings, None)
    });
    
    // Initialize timeline - triggers Action::LoadData to fetch bookmarks
    use_effect({
        let store = timeline_signal;
        let env = environment.clone();
        move || {
            let mut env_clone = env.clone();
            spawn(async move {
                handle_action(store, Action::Initial, &mut env_clone);
            });
        }
    });
    
    rsx! {
        div {
            class: "flex-1 flex flex-col bg-transparent",
            div {
                class: "p-6 border-b border-white/5",
                h2 { 
                    class: "text-2xl font-bold text-[var(--g-labelColor)]", 
                    "Bookmarked Messages" 
                }
            }
            
            // Show empty state when not loading and no posts
            if !timeline_signal.read().is_loading 
                && !timeline_signal.read().is_loading_more 
                && timeline_signal.read().posts.is_empty() {
                div {
                    class: "empty-state flex flex-col items-center justify-center p-8 text-gray-500",
                    div {
                        class: "text-4xl mb-4",
                        dangerous_inner_html: crate::icons::ICON_BOOKMARK1
                    }
                    div {
                        class: "text-lg font-semibold mb-2",
                        "No bookmarks yet"
                    }
                    div {
                        class: "text-sm",
                        "Bookmark messages to save them for later"
                    }
                }
            }
            
            TimelineComponent { store: timeline_signal }
        }
    }
}
