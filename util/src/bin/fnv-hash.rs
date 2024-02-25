use clap::Parser;
use wwise_analysis::fnv;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[arg(short, long)]
    /// The input string that needs to be hashed
    input: String,
}

fn main() {
    let args = Arguments::parse();

    println!("{}", fnv::create_hash(args.input.as_str()));
}
