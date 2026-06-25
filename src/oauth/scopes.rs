use std::fmt::Display;

use flagset::{FlagSet, flags};

pub const ALL_SCOPES: [Scope; 3] = [Scope::OpenId, Scope::Profile, Scope::Email];

flags! {
    // #[derive(strum::EnumString)] peace of crap. i hate this, anyways for what ethe heck i use sturm anyways in the whole codebase
    // #[strum(serialize_all="snake_case")]
    pub enum Scope: i64 {
        OpenId,
        Profile,
        Email,
    }
}

impl Scope {
    pub fn as_str(self) -> &'static str {
        match self {
            Scope::OpenId => "openid",
            Scope::Profile => "profile",
            Scope::Email => "email",
        }
    }

    #[allow(clippy::should_implement_trait)] // shut
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "openid" => Some(Scope::OpenId),
            "profile" => Some(Scope::Profile),
            "email" => Some(Scope::Email),
            _ => None,
        }
    }
}

// #[derive(Debug, thiserror::Error)]
// pub enum ScopeParseErrors {
//     #[error("invalid scope: {0}")]
//     InvalidScope(String),
// }

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Scopes(pub FlagSet<Scope>);

impl Scopes {
    #[allow(clippy::should_implement_trait)] // shut x2
    pub fn from_str(s: &str) -> Self {
        let mut this = FlagSet::<Scope>::default();

        for scope in s.split_whitespace() {
            let scope = &scope.trim().to_lowercase();
            let Some(scope) = Scope::from_str(scope) else {
                continue;
            };
            this |= scope;
        }

        Self(this)
    }

    pub fn bits(self) -> i64 {
        self.0.bits()
    }

    pub fn from_bits(bits: i64) -> Self {
        Self(FlagSet::new_truncated(bits))
    }

    /// Returns true if all scopes in `scopes` are contained in this instance
    pub fn contains(self, scopes: Scopes) -> bool {
        self.0.contains(scopes.0)
    }

    /// Returns true if a singular scope is contained in this instance
    pub fn has(self, scope: Scope) -> bool {
        self.0.contains(FlagSet::from(scope))
    }

    /// Removes any scopes that are not in the allowed scopes. Useful for deleting old scopes that no longer exist
    pub fn sanitize(self, allowed_scopes: Scopes) -> Self {
        Self(self.0 & allowed_scopes.0)
    }

    /// Returns a Scopes instance with all available scopes.
    pub fn all() -> Self {
        Self(FlagSet::full())
    }
}

impl Display for Scopes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        for scope in self.0 {
            s.push_str(scope.as_str());
            s.push(' ');
        }

        f.write_str(s.trim_end())
    }
}
