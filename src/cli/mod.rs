use clap::Parser;
use clap::builder::{IntoResettable, StyledStr};

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
}

impl Run for Commands {
    async fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Settings(settings) => settings.run().await,
        }
    }
}
