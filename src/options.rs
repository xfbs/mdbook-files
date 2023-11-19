use clap::Parser;

#[derive(Parser, Debug)]
pub struct Options {
    #[clap(subcommand)]
    pub command: Option<Command>
}

#[derive(Parser, Debug)]
pub enum Command {
    Supports(SupportsCommand),
    Process,
}

#[derive(Parser, Debug)]
pub struct SupportsCommand {
    pub renderer: String,
}
