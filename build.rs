use std::{env, fs::File, io::Write};

use shadow_rs::{COMMIT_HASH, SHORT_COMMIT, SdResult, ShadowBuilder};

fn main() {
    println!("cargo:rerun-if-env-changed=GIT_COMMIT");
    println!("cargo:rerun-if-env-changed=GIT_SHORT_COMMIT");

    let mut builder = ShadowBuilder::builder();

    let has_env_commit = env::var("GIT_COMMIT").is_ok_and(|v| !v.is_empty());
    if has_env_commit {
        let mut deny = shadow_rs::default_deny();
        deny.insert(COMMIT_HASH);
        deny.insert(SHORT_COMMIT);
        builder = builder.deny_const(deny).hook(hook);
    }

    builder.build().expect("failed building shadow_rs meta");
}

fn hook(mut file: &File) -> SdResult<()> {
    let commit = env::var("GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string());
    let short = env::var("GIT_SHORT_COMMIT")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| commit.chars().take(8).collect());

    writeln!(
        file,
        r#"#[allow(clippy::all, clippy::pedantic, clippy::restriction, clippy::nursery)]
        pub const COMMIT_HASH: &str = "{commit}";
        pub const SHORT_COMMIT: &str = "{short}";
        "#
    )?;
    Ok(())
}
