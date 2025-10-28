use maud::{html, Markup, PreEscaped};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub success: bool,
    pub error_message: Option<String>,
    pub app_name: Option<String>,
    pub redirect_url: Option<String>,
    pub user_email: Option<String>,
    pub custom_vars: HashMap<String, String>,
}

impl Default for TemplateContext {
    fn default() -> Self {
        Self {
            success: true,
            error_message: None,
            app_name: None,
            redirect_url: None,
            user_email: None,
            custom_vars: HashMap::new(),
        }
    }
}

impl TemplateContext {
    pub fn success() -> Self {
        Self::default()
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            error_message: Some(message.into()),
            ..Default::default()
        }
    }

    pub fn with_app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = Some(name.into());
        self
    }

    pub fn with_redirect_url(mut self, url: impl Into<String>) -> Self {
        self.redirect_url = Some(url.into());
        self
    }

    pub fn with_user_email(mut self, email: impl Into<String>) -> Self {
        self.user_email = Some(email.into());
        self
    }

    pub fn with_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_vars.insert(key.into(), value.into());
        self
    }
}

pub fn default_success_template(ctx: &TemplateContext) -> Markup {
    let app_name = ctx.app_name.as_deref().unwrap_or("Application");

    html! {
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (app_name) " - Authentication Successful" }
                style {
                    (PreEscaped(r#"
                        body { 
                            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            min-height: 100vh;
                            margin: 0;
                            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                            color: white;
                        }
                        .container {
                            text-align: center;
                            background: rgba(255, 255, 255, 0.1);
                            padding: 3rem;
                            border-radius: 1rem;
                            backdrop-filter: blur(10px);
                            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
                        }
                        .success { color: #4ade80; }
                        .error { color: #f87171; }
                        h1 { margin: 0 0 1rem 0; }
                        p { margin: 0.5rem 0; opacity: 0.9; }
                        .close-btn {
                            margin-top: 1.5rem;
                            padding: 0.75rem 1.5rem;
                            background: rgba(255, 255, 255, 0.2);
                            border: 1px solid rgba(255, 255, 255, 0.3);
                            border-radius: 0.5rem;
                            color: white;
                            cursor: pointer;
                            font-size: 1rem;
                        }
                        .close-btn:hover {
                            background: rgba(255, 255, 255, 0.3);
                        }
                    "#))
                }
            }
            body {
                div class="container" {
                    @if ctx.success {
                        div class="success" {
                            h1 { "✓ Authentication Successful" }
                            p { "You have successfully authenticated with " (app_name) "." }
                            @if let Some(email) = &ctx.user_email {
                                p { "Logged in as: " strong { (email) } }
                            }
                            p { "You can now close this window and return to the application." }
                            @if let Some(url) = &ctx.redirect_url {
                                p {
                                    a href=(url) { "Continue to application" }
                                }
                            }
                        }
                    } @else {
                        div class="error" {
                            h1 { "✗ Authentication Failed" }
                            @if let Some(error) = &ctx.error_message {
                                p { "Error: " (error) }
                            }
                            p { "Please close this window and try again." }
                        }
                    }
                    button class="close-btn" onclick="window.close()" {
                        "Close Window"
                    }
                }
                script {
                    (PreEscaped(r#"
                        // Auto-redirect if URL provided
                        const redirectUrl = "#)) 
                        @if let Some(url) = &ctx.redirect_url {
                            (PreEscaped(&format!("'{}'", url)))
                        } @else {
                            (PreEscaped("null"))
                        }
                        (PreEscaped(r#";
                        if (redirectUrl && window.opener) {
                            setTimeout(() => {
                                window.opener.location = redirectUrl;
                                window.close();
                            }, 2000);
                        }
                    "#))
                }
            }
        }
    }
}

pub fn minimal_template(ctx: &TemplateContext) -> Markup {
    html! {
        html {
            head {
                meta charset="utf-8";
                title { "Authentication Complete" }
                style {
                    "body { font-family: system-ui; text-align: center; margin: 2rem; }"
                }
            }
            body {
                @if ctx.success {
                    h1 { "Success" }
                    p { "Authentication complete. You can close this window." }
                } @else {
                    h1 { "Error" }
                    @if let Some(error) = &ctx.error_message {
                        p { (error) }
                    }
                }
                script { "setTimeout(() => window.close(), 1000);" }
            }
        }
    }
}
