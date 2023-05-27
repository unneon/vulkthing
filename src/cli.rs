use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[arg(long)]
    pub demo: bool,
}
