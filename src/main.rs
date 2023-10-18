#[allow(dead_code, unused_imports)]
mod filetree;
pub mod processing;
pub mod ui;

use clap::Parser;
use std::env;

#[derive(Parser, Debug)]
pub struct Args {
    /// Print all files
    #[arg(long, short)]
    all: bool,
    /// Print recursively all directories
    #[arg(long, short = 'R')]
    recursive: bool,
    /// Use a list format
    #[arg(long, short, default_value_t = false)]
    long: bool,
    #[arg(long, short)]
    size: bool,
    /// Pretty print file sizes
    #[arg(long = "human-readable", short = 'H')]
    // may need to change that latter to conform to ls
    human_readable: bool,
    /// list one file per line
    #[arg(short = '1')]
    one_file_per_line: bool,
}

fn main() -> anyhow::Result<()> {
    let current_dir = env::current_dir()?;
    let args = Args::parse();
    let mut tree = filetree::walker::get_tree(current_dir, &args)?;

    if let Err(err) = tree.sort_by(&args) {
        println!("{}", err);
    }

    crate::filetree::FileTree::ls_print(&tree, &args);

    Ok(())
}
