use crate::{
    error::OAuthError,
    future::WrappedFuture,
    pkce::PkceChallenge,
    server,
    types::{AccessType, OAuthResponse},
    Result,
};
use zeroize::Zeroizing;

pub struct AuthFlow {
    client_id: String,
    client_secret: Zeroizing<String>,
    redirect_uri: String,
    scopes: Vec<String>,
    state: Option<String>,
    access_type: AccessType,
    pkce_challenge: PkceChallenge,
}

impl AuthFlow {
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Result<Self> {
        let pkce_challenge = PkceChallenge::new()?;
        Ok(Self {
            client_id: client_id.into(),
            client_secret: Zeroizing::new(client_secret.into()),
            redirect_uri: redirect_uri.into(),
            scopes: vec!["user:email".to_string()],
            state: None,
            access_type: AccessType::Online,
            pkce_challenge,
        })
    }

    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }

    pub fn with_state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    pub fn with_access_type(mut self, access_type: AccessType) -> Self {
        self.access_type = access_type;
        self
    }


    /// Use a custom PKCE challenge/verifier pair
    /// 
    /// This allows you to provide your own PKCE challenge, which can be useful
    /// for testing or when you need to manage the challenge lifecycle yourself.
    pub fn with_pkce_challenge(mut self, challenge: PkceChallenge) -> Self {
        self.pkce_challenge = challenge;
        self
    }

    pub fn auth_url(&self) -> String {
        let scope = self.scopes.join(" ");
        let default_state;
        let state = match self.state.as_deref() {
            Some(s) => s,
            None => {
                default_state = uuid::Uuid::new_v4().to_string();
                &default_state
            }
        };

        let mut auth_url = format!(
            "https://github.com/login/oauth/authorize?scope={}&response_type=code&state={}&redirect_uri={}&client_id={}",
            urlencoding::encode(&scope),
            urlencoding::encode(state),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&self.client_id)
        );

        // Add PKCE parameters (always enabled in 2025)
        auth_url.push_str(&format!(
            "&code_challenge={}&code_challenge_method={}",
            urlencoding::encode(&self.pkce_challenge.code_challenge),
            self.pkce_challenge.challenge_method()
        ));

        auth_url
    }

    pub fn handle_callback(&self, callback_url: &str) -> WrappedFuture<Result<OAuthResponse>> {
        let code = match server::extract_callback_code(callback_url) {
            Ok(code) => code,
            Err(e) => return WrappedFuture::new(async move { Err(e) }),
        };

        let client_id = self.client_id.clone();
        let client_secret = self.client_secret.clone();
        let redirect_uri = self.redirect_uri.clone();
        let pkce_challenge = self.pkce_challenge.clone();

        WrappedFuture::new(async move {
            let mut params = vec![
                ("code", code),
                ("client_id", client_id),
                ("client_secret", client_secret.as_str().to_string()),
                ("redirect_uri", redirect_uri),
                ("grant_type", "authorization_code".to_string()),
            ];

            // Add code_verifier (PKCE always enabled in 2025)
            params.push(("code_verifier", pkce_challenge.code_verifier.as_str().to_string()));

            let client = reqwest::Client::new();
            let response = client
                .post("https://github.com/login/oauth/access_token")
                .header("Accept", "application/json")
                .form(&params)
                .send()
                .await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                return Err(OAuthError::TokenExchange(error_text));
            }

            let oauth_response: OAuthResponse = response.json().await?;
            Ok(oauth_response)
        })
    }
}
