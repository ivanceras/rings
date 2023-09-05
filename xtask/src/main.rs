use clap::{Parser, Subcommand};
use xshell::{cmd, Shell};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// generate asset.rs file
    GenerateAssets,
    /// fix the files replacing kebab-case to snake_case
    FixFilenames,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let sh = Shell::new()?;
    let base_dir = "icons";
    match args.command {
        Commands::FixFilenames => {
            let svg_files = sh.read_dir(base_dir)?;
            for file in svg_files {
                let ext = file.extension().expect("must have extension");
                let mut path_buf = file.to_path_buf();
                if ext == "svg" {
                    let filename = file.file_name().expect("must have a filename");
                    let filename = filename.to_str().expect("must be a valid filename");
                    let new_filename = filename.replace("-", "_");
                    path_buf.set_file_name(&new_filename);
                    println!("file: {}", file.display());
                    println!("-->>> {}", path_buf.display());
                    std::fs::rename(file, path_buf)?;
                }
            }
        }
        Commands::GenerateAssets => {
            let svg_files = sh.read_dir(base_dir)?;
            for file in svg_files {
                println!("{:?} -> {:?}", file.file_name(), file.file_stem());
            }
        }
    }
    Ok(())
}
