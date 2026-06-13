use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub file: String,

    #[arg(short, long)]
    pub out_dir: String,

    #[arg(short, long)]
    pub without_cover: bool,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub ext_args: Vec<String>,
}
