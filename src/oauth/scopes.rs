use std::fmt::Display;

use flagset::{FlagSet, flags};

pub const ALL_SCOPES: [Scope; 3] = [Scope::OpenId, Scope::Profile, Scope::Email];

flags! {
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

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "openid" => Some(Scope::OpenId),
            "profile" => Some(Scope::Profile),
            "email" => Some(Scope::Email),
            _ => None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ScopeParseErrors {
    #[error("invalid scope: {0}")]
    InvalidScope(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Scopes(pub FlagSet<Scope>);

impl Scopes {
    pub fn from_str(s: &str, ignore_bad: bool) -> Result<Self, ScopeParseErrors> {
        let mut this = FlagSet::<Scope>::default();

        for scope in s.split_whitespace() {
            let Some(scope) = Scope::from_str(scope) else {
                if ignore_bad {
                    continue;
                } else {
                    return Err(ScopeParseErrors::InvalidScope(scope.to_string()));
                }
            };
            this |= scope;
        }

        Ok(Self(this))
    }

    pub fn bits(self) -> i64 {
        self.0.bits()
    }

    pub fn from_bits(bits: i64) -> Self {
        Self(FlagSet::new_truncated(bits))
    }

    pub fn contains(self, scopes: Scopes) -> bool {
        self.0.contains(scopes.0)
    }

    pub fn sanitize(self, scopes: Scopes) -> Self {
        Self(self.0 & scopes.0)
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
