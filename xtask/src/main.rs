#![deny(warnings)]

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use sauron::Render;
use sauron_html_parser::parse_html;
use xshell::Shell;

const RESERVED_KEYWORDS: &[&str] = &["box", "macro", "move", "type"];

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

fn prettify_html(content: &str) -> Result<String> {
    if let Some(node) = parse_html::<()>(&content)? {
        let mut buffer = String::new();
        node.render_with_indent(&mut buffer, 2, false)?;
        Ok(buffer)
    } else {
        Err(anyhow!("Error parsing html.."))
    }
}

fn main() -> Result<()> {
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
                    let content = sh.read_file(&file)?;
                    let filename = file.file_name().expect("must have a filename");
                    let filename_str = filename.to_str().expect("must be a valid filename");
                    if filename_str.contains("-") {
                        let new_filename = filename_str.replace("-", "_");
                        path_buf.set_file_name(&new_filename);
                        println!("Rename {} -> {}", filename_str, new_filename);
                        std::fs::rename(file, &path_buf)?;
                    }
                    let pretty_html = prettify_html(&content)?;
                    println!("Prettifying {}", path_buf.display());
                    sh.write_file(path_buf, pretty_html)?;
                }
            }
        }
        Commands::GenerateAssets => {
            let svg_files = sh.read_dir(base_dir)?;
            let mut buffer = String::new();
            buffer += "//! Warning: DO NOT EDIT THIS FILE";
            buffer += "\n//! This file is generated using `cargo xtask generate-assets`";
            buffer += "\n";
            buffer += "\nuse sauron::{Node,node};";
            buffer += "\n";
            for file in svg_files {
                let filestem = file.file_stem().expect("must have a file stem");
                println!("Processing {:?}...", file.file_name());
                let fn_name = filestem.to_str().expect("must be a str");
                let starts_digit = if let Some(bytes) = fn_name.as_bytes().get(0) {
                    bytes.is_ascii_digit()
                } else {
                    false
                };
                let fn_name = if RESERVED_KEYWORDS.contains(&fn_name) {
                    format!("r#{fn_name}")
                } else if starts_digit {
                    format!("_{fn_name}")
                } else {
                    fn_name.to_string()
                };
                let content = sh.read_file(&file)?;
                let pretty = prettify_html(&content)?;
                let code = &format!(
                    "\npub fn {fn_name}<MSG>() -> Node<MSG>{{\
                     \n   node!{{\
                     \n       {pretty}\
                     \n   }}\
                     \n}}\
                     \n"
                );
                buffer += code;
            }
            let assets_file = "rings/src/assets.rs";
            println!("Writing to {}", assets_file);
            println!("{}", buffer);
            sh.write_file(assets_file, buffer)?;
        }
    }
    Ok(())
}
