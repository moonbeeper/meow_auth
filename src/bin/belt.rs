use clap::Parser;
use meow_auth::cli::Run;

#[tokio::main]
async fn main() {
    let _ = nu_ansi_term::enable_ansi_support();
    // https://github.com/crate-ci/typos/blob/master/crates/typos-cli/src/bin/typos-cli/main.rs#L14C33-L15C5
    human_panic::setup_panic!();

    let cmd = match meow_auth::cli::Commands::try_parse() {
        Ok(cmd) => cmd,
        Err(e) => {
            e.print().expect("where's the clap error?");
            std::process::exit(e.exit_code());
        }
    };

    match cmd.run().await {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("Whoops: {}", e);
            std::process::exit(1);
        }
    }
}
