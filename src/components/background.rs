use dioxus::prelude::*;

#[allow(dead_code)] // Keep for potential fallback use
static OCEANIC_BG: Asset = asset!("/assets/img/cyrup_oceanic.png");

#[component]
pub fn BackgroundImage() -> Element {
    rsx! {
        style {
            r#"
            .background-container {{
                position: fixed;
                top: 0;
                left: 0;
                width: 100vw;
                height: 100vh;
                background-image: url("{OCEANIC_BG}");
                background-size: cover;
                background-position: center;
                background-repeat: no-repeat;
                z-index: -1;
            }}
            .background-container::before {{
                content: "";
                position: absolute;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                background-color: rgba(0, 0, 0, 0.4);
                z-index: 1;
            }}
            "#
        }
        div { class: "background-container" }
    }
}
