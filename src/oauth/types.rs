use compact_jwt::{Jwk, JwsEs256Signer};
use data_encoding::BASE64URL_NOPAD;

use crate::database::id::UlidId;

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, utoipa::IntoParams, Clone)]
pub struct AuthorizationRequest {
    pub response_type: ResponseType,
    pub client_id: UlidId,
    pub state: Option<String>,
    pub scope: Option<String>,
    pub redirect_uri: Option<String>,
    pub code_challenge: String,
    pub code_challenge_method: CodeChallengeMethod,
    // OIDC
    pub nonce: Option<String>,
    pub prompt: Option<PromptType>,
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
pub enum CodeChallengeMethod {
    #[serde(rename = "plain")]
    Plain,
    S256,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PromptType {
    None,
    Login, // Don't care.
    Consent,
    SelectAccount,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct AuthorizationDecisionRequest {
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
    pub client_secret: String,
    pub code_verifier: String,
    pub redirect_uri: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GrantType {
    AuthorizationCode, // there's no way ill support other grant types :)
    RefreshToken,
    ClientCredentials,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Bearer,
    Mac,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: TokenType,
    pub expires_in: i64,
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>, // oidc
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct OauthMetadata {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub jwks_uri: String,
    pub scopes_supported: Vec<String>,
    pub response_types_supported: Vec<ResponseType>,
    pub response_modes_supported: Vec<ResponseModes>,
    pub grant_types_supported: Vec<GrantType>,
    pub code_challenge_methods_supported: Vec<CodeChallengeMethod>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseModes {
    Query,
    Fragment,
    FormPost,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct JwkKeySet {
    pub keys: Vec<JwkKey>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema, PartialEq, Eq, smart_default::SmartDefault)]
#[serde(rename_all = "snake_case")]
pub struct JwkKey {
    #[default("EC".to_string())]
    pub kty: String,
    #[default("P-256".to_string())]
    pub crv: String,
    #[default("ec x".to_string())]
    pub x: String,
    #[default("ec y".to_string())]
    pub y: String,
    #[default("ES256".to_string())]
    pub alg: String,
    #[serde(rename = "use")]
    #[default("sig".to_string())]
    pub use_: String,
    #[default("kid".to_string())]
    pub kid: String,
}

impl JwkKey {
    pub fn from_signer(signer: &JwsEs256Signer, kid: String) -> Self {
        let value = signer
            .public_key_as_jwk()
            .expect("jwk key should be valid to reach here");
        match value {
            Jwk::EC { x, y, .. } => Self {
                x: BASE64URL_NOPAD.encode(&x).to_string(),
                y: BASE64URL_NOPAD.encode(&y).to_string(),
                kid,
                ..Default::default()
            },
            Jwk::RSA { .. } => unimplemented!("not using rsa?"),
        }
    }
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct OpenIdMetadata {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwks_uri: String,
    pub subject_types_supported: Vec<SubjectTypes>,
    pub id_token_signing_alg_values_supported: Vec<IdTokenSigningAlg>,
    pub token_endpoint_auth_methods_supported: Vec<TokenAuthMethod>,
    pub scopes_supported: Vec<String>,
    pub response_types_supported: Vec<ResponseType>,
    pub response_modes_supported: Vec<ResponseModes>,
    pub grant_types_supported: Vec<GrantType>,
    pub code_challenge_methods_supported: Vec<CodeChallengeMethod>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubjectTypes {
    Pairwise,
    Public,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema, PartialEq, Eq)]
pub enum IdTokenSigningAlg {
    ES256,
    RS256,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TokenAuthMethod {
    ClientSecretPost,
    ClientSecretBasic,
    ClientSecretJwt,
    PrivateKeyJwt,
}
