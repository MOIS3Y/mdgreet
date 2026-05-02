use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "mdgreet", about = "Material Design greeter")]
pub struct Args {
    /// Path to configuration file
    #[arg(short, long)]
    pub config: Option<String>,
}

impl Args {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
