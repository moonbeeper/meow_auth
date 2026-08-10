use std::fmt::Display;

use flagset::{FlagSet, flags};

// pub const ALL_USERFLAGS: [UserFlags; 3] = [UserFlags::OpenId, UserFlags::Profile, UserFlags::Email];

flags! {
    pub enum UserFlag: i64 {
        SuperAdmin, // for big birds like me - moonbeeper :3
        // Admin, I frankly dont see the point of having a lesser admin role. for now at least.
        CannotManageOauthApplications,
        CannotAuthorizeOauthApplications,
        CannotModifyName,
        CannotModifyEmail,
        // If this is set, the user has completed their setup process by changing their display name.
        // If not, access not granted to anything :(
        HasSetName,
    }
}

impl UserFlag {
    pub fn as_str(self) -> &'static str {
        match self {
            UserFlag::SuperAdmin => "super_admin",
            UserFlag::CannotManageOauthApplications => "cannot_manage_oauth_applications",
            UserFlag::CannotAuthorizeOauthApplications => "cannot_authorize_oauth_applications",
            UserFlag::CannotModifyName => "cannot_modify_name",
            UserFlag::CannotModifyEmail => "cannot_modify_email",
            UserFlag::HasSetName => "has_set_name",
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct UserFlags(pub FlagSet<UserFlag>);

impl UserFlags {
    pub fn bits(self) -> i64 {
        self.0.bits()
    }

    pub fn from_bits(bits: i64) -> Self {
        Self(FlagSet::new_truncated(bits))
    }

    /// Returns true if all scopes in `scopes` are contained in this instance
    pub fn contains(self, scopes: UserFlags) -> bool {
        self.0.contains(scopes.0)
    }

    /// Returns true if a singular scope is contained in this instance
    pub fn has(self, scope: UserFlag) -> bool {
        self.0.contains(FlagSet::from(scope))
    }

    /// Removes any scopes that are not in the allowed scopes. Useful for deleting old scopes that no longer exist
    pub fn sanitize(self, allowed_scopes: UserFlags) -> Self {
        Self(self.0 & allowed_scopes.0)
    }

    /// Returns a Scopes instance with all available scopes
    pub fn all() -> Self {
        Self(FlagSet::full())
    }

    /// Add a new flag to the instance
    #[allow(clippy::should_implement_trait)] // I don't feel like doing that?
    pub fn add(mut self, flag: UserFlag) -> Self {
        self.0 |= flag;
        self
    }

    /// Remove a flag from the instance
    pub fn remove(mut self, flag: UserFlag) -> Self {
        self.0 -= flag;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Display for UserFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        for scope in self.0 {
            s.push_str(scope.as_str());
            s.push(' ');
        }

        f.write_str(s.trim_end())
    }
}
