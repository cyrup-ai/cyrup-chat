mod auth_flow;
mod error;
mod future;
mod login;
mod pkce;
mod provider;
mod refresh;
mod server;
mod template;
mod traits;
mod types;
mod user_info;

pub use auth_flow::AuthFlow;
pub use error::OAuthError;
pub use future::WrappedFuture;
pub use pkce::PkceChallenge;
pub use provider::GitHubProvider;
pub use template::{default_success_template, minimal_template, TemplateContext};
pub use traits::{OAuthConfigBuilder, OAuthLogin, OAuthProvider, OAuthRefresh, OAuthUserInfo};
pub use types::{AccessType, CallbackMode, OAuthResponse, TokenResponse, UserInfo};

pub use login::{Login, LoginConfigBuilder, LoginScopesBuilder};
pub use refresh::{Refresh, RefreshExecuteBuilder, RefreshTokenBuilder};
pub use user_info::{UserInfo as UserInfoBuilder, UserInfoExecuteBuilder};

pub type Result<T> = std::result::Result<T, OAuthError>;
