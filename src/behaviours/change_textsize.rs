use super::Behaviour;
use crate::environment::types::{MainMenuEvent, UiConfig, UiZoom};
use dioxus::prelude::spawn;

pub struct ChangeTextsizeBehaviour {}

impl Behaviour for ChangeTextsizeBehaviour {
    type InputAction = MainMenuEvent;
    type InputState = UiConfig;
    type Environment = crate::environment::Environment;

    fn setup(environment: &Self::Environment) {
        if let Ok(config) = environment.settings.config() {
            let zoom = config.zoom.css_class();
            log::debug!("startup zoom {zoom}");
            let js = format!(
                r#"
                document.documentElement.classList.add("{zoom}");
                "#
            );

            // Spawn async task for JavaScript execution
            spawn(async move {
                if let Err(e) = crate::environment::platform::execute_js_once_async(&js).await {
                    log::error!("Failed to apply zoom class on startup: {}", e);
                }
            });
        }
    }

    fn handle(
        action: MainMenuEvent,
        state: &mut Self::InputState,
        _environment: &Self::Environment,
    ) {
        match action {
            MainMenuEvent::TextSizeIncrease => {
                let new = state.zoom.increase();
                change_textsize(state, new);
            }
            MainMenuEvent::TextSizeDecrease => {
                let new = state.zoom.decrease();
                change_textsize(state, new);
            }
            MainMenuEvent::TextSizeReset => change_textsize(state, Some(UiZoom::Z100)),
            _ => {}
        }
    }
}

fn change_textsize(state: &mut UiConfig, new: Option<UiZoom>) {
    let current = state.zoom.css_class();
    let Some(new) = new else {
        return;
    };
    log::debug!("change: old zoom {:?}", state.zoom);
    log::debug!("change: new zoom {:?}", new);
    state.zoom = new;
    let new_class = new.css_class();

    let js = format!(
        r#"
        document.documentElement.classList.remove("{current}");
        document.documentElement.classList.add("{new_class}");
    "#
    );

    // Spawn async task for JavaScript execution
    spawn(async move {
        if let Err(e) = crate::environment::platform::execute_js_once_async(&js).await {
            log::error!("Failed to change text size zoom class: {}", e);
        }
    });
}
