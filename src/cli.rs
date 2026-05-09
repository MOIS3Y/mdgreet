use clap::Parser;

/// Command-line arguments for the application.
#[derive(Parser, Debug)]
#[command(
    name = "mdgreet",
    version,
    about = "A clean Material Design 3 greeter for greetd in Rust and Slint"
)]
pub struct Args {
    /// Path to configuration file (TOML)
    #[arg(short, long)]
    pub config: Option<String>,

    /// Run in demo mode (no greetd connection)
    #[arg(short, long, default_value_t = false)]
    pub demo: bool,
}

impl Args {
    /// Parses the command-line arguments and returns an instance of `Args`.
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
