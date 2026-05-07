use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "mdgreet",
    version,
    about = "A clean Material Design 3 greeter for greetd in Rust and Slint"
)]
pub struct Args {
    /// Path to configuration file
    #[arg(short, long)]
    pub config: Option<String>,

    /// Run in demo mode (no greetd connection)
    #[arg(short, long, default_value_t = false)]
    pub demo: bool,
}

impl Args {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
