use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    file: String,

    #[arg(short, long)]
    out_dir: String,

    #[arg(short, long)]
    without_cover: bool,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    ext_args: Vec<String>,
}

fn main() {
    let cli = Cli::parse();

    println!("{:?}", cli);
}
