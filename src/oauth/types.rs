use crate::database::id::UlidId;

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, utoipa::IntoParams, Clone)]
pub struct AuthorizationRequest {
    pub response_type: ResponseType,
    pub client_id: UlidId,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub scope: Option<String>,
    pub redirect_uri: Option<String>,
    pub code_challenge: String,
    pub code_challenge_method: CodeChallengeMethod,
}

// must be code
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    Code,
    Token,
}

// must be s256
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CodeChallengeMethod {
    Plain,
    S256,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct AuthorizationFinishRequest {
    pub consent: bool,
    pub client_id: UlidId,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct AuthorizationResponse {
    pub code: String,
    pub state: String,
    pub iss: String,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct TokenRequest {
    pub grant_type: GrantType,
    pub code: String,
    pub client_id: UlidId,
    pub code_verifier: String,
    pub redirect_uri: Option<String>,
    pub client_secret: String,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GrantType {
    AuthorizationCode, // there's no way ill support other grant types :)
    RefreshToken,
    ClientCredentials,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: String,
    pub id_token: Option<String>, // oidc
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct OauthMetadata {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub scopes_supported: Vec<String>,
    pub response_types_supported: Vec<ResponseType>,
    pub code_challenge_methods_supported: Vec<CodeChallengeMethod>,
}
