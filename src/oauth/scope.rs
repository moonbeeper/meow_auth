use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::Display,
};

use crate::{
    auth::flags::UserFlags,
    database::models::oauth_application::{ClientType, OauthApplicationId},
    doc_node_ignore,
    settings::Settings,
};

pub const BUILTIN_SCOPE_OPENID_IDENTIFIER: &str = "openid";
pub const BUILTIN_SCOPE_PROFILE_IDENTIFIER: &str = "profile";
pub const BUILTIN_SCOPE_EMAIL_IDENTIFIER: &str = "email";

pub const BUILTIN_SCOPE_OPENID: SettingsScopeDef = SettingsScopeDef {
    identifier: Cow::Borrowed(BUILTIN_SCOPE_OPENID_IDENTIFIER),
    description: Some(Cow::Borrowed("Lets you get a OpenID Connect ID token")),
    allowed_client_ids: None,
    allowed_client_type: ClientType::ThirdParty,
    required_user_flags: None,
};

pub const BUILTIN_SCOPE_PROFILE: SettingsScopeDef = SettingsScopeDef {
    identifier: Cow::Borrowed(BUILTIN_SCOPE_PROFILE_IDENTIFIER),
    description: Some(Cow::Borrowed("Your display name and avatar")),
    allowed_client_ids: None,
    allowed_client_type: ClientType::ThirdParty,
    required_user_flags: None,
};

pub const BUILTIN_SCOPE_EMAIL: SettingsScopeDef = SettingsScopeDef {
    identifier: Cow::Borrowed(BUILTIN_SCOPE_EMAIL_IDENTIFIER),
    description: Some(Cow::Borrowed("Your email address")),
    allowed_client_ids: None,
    allowed_client_type: ClientType::ThirdParty,
    required_user_flags: None,
};

pub const BUILTIN_SCOPES: &[SettingsScopeDef] = &[
    BUILTIN_SCOPE_OPENID,
    BUILTIN_SCOPE_PROFILE,
    BUILTIN_SCOPE_EMAIL,
];

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct SettingsScopeDef {
    pub identifier: Cow<'static, str>,
    pub description: Option<Cow<'static, str>>,
    pub allowed_client_ids: Option<Vec<OauthApplicationId>>,
    pub allowed_client_type: ClientType,
    pub required_user_flags: Option<u64>, // why are the flags an i64??
}
doc_node_ignore!(SettingsScopeDef);

impl From<SettingsScopeDef> for ScopeDef {
    fn from(value: SettingsScopeDef) -> Self {
        Self {
            identifier: clean_identifier(&value.identifier),
            description: value.description.as_deref().map(|v| v.to_string()),
            allowed_client_ids: value.allowed_client_ids.clone(),
            allowed_client_type: value.allowed_client_type,
            required_user_flags: value.required_user_flags.unwrap_or_default().cast_signed(),
        }
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct ScopeDef {
    pub identifier: String,
    pub description: Option<String>,
    pub allowed_client_ids: Option<Vec<OauthApplicationId>>,
    pub allowed_client_type: ClientType,
    pub required_user_flags: i64, // why are the flags an i64??
}

impl ScopeDef {
    pub fn allowed_type(&self, client_type: ClientType) -> bool {
        // if its first party, its always allowed
        if client_type == ClientType::FirstParty {
            return true;
        }

        // only allowed if the scope allows third party clients (i ignore mr param)
        self.allowed_client_type == ClientType::ThirdParty
    }

    pub fn allowed_client(&self, client_id: Option<OauthApplicationId>) -> bool {
        match (client_id, &self.allowed_client_ids) {
            (_, None) => true,
            (None, Some(_)) => false,
            (Some(id), Some(allowed)) => allowed.contains(&id),
        }
        // self.allowed_client_ids
        //     .as_ref()
        //     .map_or_else(|| false, |v| v.contains(&client_id))
    }

    pub fn allowed_userflags(&self, user_flags: UserFlags) -> bool {
        user_flags.contains(UserFlags::from_bits(self.required_user_flags))
    }

    pub fn allowed() {}
}

#[derive(Debug)]
pub struct ScopeRegistry {
    pub scopes: HashMap<String, ScopeDef>,
}

impl ScopeRegistry {
    pub fn new(settings: &Settings) -> anyhow::Result<Self> {
        let mut scopes = HashMap::new();
        tracing::info!("Registring oauth scopes");

        if let Some(scopes_defs) = &settings.oauth.scopes {
            for scope in scopes_defs {
                let scope = ScopeDef::from(scope.clone());
                if scopes.contains_key(&scope.identifier) {
                    anyhow::bail!("duplicated scope identifer");
                }

                let user_flags = UserFlags::from_bits(scope.required_user_flags);
                scopes.insert(
                    scope.identifier.to_lowercase(),
                    ScopeDef {
                        required_user_flags: user_flags.bits(),
                        ..scope
                    },
                );
            }
        }

        for scope in BUILTIN_SCOPES {
            let scope = ScopeDef::from(scope.clone());
            scopes.entry(scope.identifier.clone()).or_insert(scope);
        }

        tracing::info!(
            "Registered {} custom scopes; {} Built in scopes",
            scopes.len() - BUILTIN_SCOPES.len(),
            BUILTIN_SCOPES.len()
        );

        Ok(Self { scopes })
    }

    pub fn get(&self, identifier: &str) -> Option<ScopeDef> {
        self.scopes.get(identifier).cloned()
    }

    pub fn available_scopes(
        &self,
        client_id: Option<OauthApplicationId>,
        client_type: ClientType,
        user_flags: UserFlags,
    ) -> Vec<ScopeDef> {
        self.scopes
            .values()
            .filter(|v| {
                v.allowed_userflags(user_flags)
                    && v.allowed_type(client_type)
                    && v.allowed_client(client_id)
            })
            .cloned()
            .collect::<Vec<_>>()
    }

    pub fn all_registered(&self) -> Vec<ScopeDef> {
        self.scopes.values().cloned().collect::<Vec<_>>()
    }
}

impl IntoIterator for ScopeRegistry {
    type Item = ScopeDef;
    type IntoIter = std::collections::hash_map::IntoValues<String, Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.scopes.into_values()
    }
}

impl<'a> IntoIterator for &'a ScopeRegistry {
    type Item = &'a ScopeDef;
    type IntoIter = std::collections::hash_map::Values<'a, String, ScopeDef>;

    fn into_iter(self) -> Self::IntoIter {
        self.scopes.values()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Scopes(HashSet<String>);

impl Scopes {
    /// Creates a new Scopes instance from a string of space separated scope identifiers.
    ///
    /// Scopes that aren't registered in the registry will be ignored.
    pub fn from_str(s: &str, registry: &ScopeRegistry) -> Self {
        let mut scopes = HashSet::new();

        for identifier in s.split_whitespace() {
            let identifier = clean_identifier(identifier);
            if registry.get(&identifier).is_some() {
                scopes.insert(identifier);
            }
        }

        Self(scopes)
    }

    /// Returns true if all scopes in `scopes` are contained in this instance
    pub fn contains(&self, scopes: Self) -> bool {
        self.0.is_superset(&scopes.0)
    }

    /// Returns true if a singular scope is contained in this instance
    pub fn has(&self, scope: &str) -> bool {
        self.0.contains(&clean_identifier(scope))
    }

    /// Removes any scopes that are not in the allowed scopes. Useful for deleting old scopes that no longer exist
    pub fn sanitize(self, allowed_scopes: Self) -> Self {
        Self(self.0.intersection(&allowed_scopes.0).cloned().collect())
    }

    /// Returns a Scopes instance with all available scopes
    pub fn all(registry: &ScopeRegistry) -> Self {
        Self(registry.into_iter().map(|v| v.identifier.clone()).collect())
    }

    /// Returns true if there's no scopes set
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::ops::Add<&str> for Scopes {
    type Output = Self;

    fn add(mut self, rhs: &str) -> Self::Output {
        self.0.insert(clean_identifier(rhs));
        self
    }
}
impl std::ops::Sub<&str> for Scopes {
    type Output = Self;

    fn sub(mut self, rhs: &str) -> Self::Output {
        self.0.remove(&clean_identifier(rhs));
        self
    }
}

impl Display for Scopes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        for scope in self.0.clone() {
            s.push_str(scope.as_str());
            s.push(' ');
        }

        f.write_str(s.trim_end())
    }
}

/// Cleans an identifier by removing any non ascii characters and replacing spaces by underscores
pub fn clean_identifier(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.trim().chars().flat_map(|v| v.to_lowercase()) {
        if c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.') {
            out.push(c);
        }

        if c.is_whitespace() && !out.is_empty() {
            out.push('_');
        }
    }
    out.trim_matches('_').to_string()
}
