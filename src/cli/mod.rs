use clap::Parser;
use clap::builder::{IntoResettable, StyledStr};
use console::Term;
use dialoguer::Confirm;
use tokio::task;

mod database;
mod settings;

pub trait Run {
    #[allow(async_fn_in_trait)]
    async fn run(&self) -> anyhow::Result<()>;
}

struct HelpTemplate;

impl IntoResettable<StyledStr> for HelpTemplate {
    fn into_resettable(self) -> clap::builder::Resettable<StyledStr> {
        color_print::cstr!(
            r#"<bold><underline>{name} {version}</underline></bold>
<dim>{tab}{author}</dim>

cool cli for managing stuff for this beautiful backend. its a toolbelt!! <italic>(wow)</italic>

{usage-heading}
{tab}{usage}

{all-args}{after-help}
"#
        )
        .into_resettable()
    }
}

#[derive(Parser)]
#[clap(version, about, author, propagate_version = true, help_template = HelpTemplate)]
pub enum Commands {
    #[clap(alias = "s")]
    Settings(settings::Settings),
    #[clap(alias = "db")]
    Database(database::Database),
}

impl Run for Commands {
    async fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Settings(settings) => settings.run().await,
            Self::Database(database) => database.run().await,
        }
    }
}

struct PromptGuard(bool);

impl Drop for PromptGuard {
    fn drop(&mut self) {
        if !self.0 {
            Term::stderr().show_cursor().unwrap();
        }
    }
}

async fn ask_prompt(prompt: String) -> bool {
    let mut guard = PromptGuard(false);
    let decision = task::spawn_blocking(move || {
        Confirm::new()
            .with_prompt(prompt)
            .wait_for_newline(true)
            .default(true)
            .show_default(true)
            .interact()
    })
    .await
    .expect("Decision task panicked");
    match decision {
        Ok(result) => {
            guard.0 = true;
            result
        }
        _ => {
            drop(guard);
            false
        }
    }
}
