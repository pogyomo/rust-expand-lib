use anyhow::{bail, Context, Result};
use clap::Parser;
use std::{
    ffi::OsString,
    fmt::Write as _,
    fs::{canonicalize, File},
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Component, PathBuf},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    crate_path: String,

    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,
}

// TODO: This function is too long and must be separated into some functions.
fn expand(root: &mut PathBuf, reading_mod_name: OsString, reading_lib: bool) -> Result<String> {
    if !root.is_file() {
        bail!("{} is a directory", root.display());
    }
    let file = File::open(&root).with_context(|| format!("failed to open {}", root.display()))?;
    let lines = BufReader::new(file).lines();
    let mut res = String::new();
    for line in lines {
        let line = line.with_context(|| format!("failed to create line from file"))?;
        let mut tokens = line.split_whitespace();
        let is_pub = match tokens.clone().next() {
            Some("pub") => {
                tokens.next();
                true
            }
            _ => false,
        };
        let head = tokens.next();
        if matches!(head, Some("//") | Some("///")) {
            continue;
        } else if head != Some("mod") {
            writeln!(res, "{line}").with_context(|| format!("failed to write line"))?;
            continue;
        }
        let Some(mod_name) = tokens.next() else {
            bail!("No module name exist after `mod`");
        };
        if mod_name.chars().last().unwrap() == ';' {
            // extract module name
            let mod_name = &mod_name[0..mod_name.len() - 1];

            let (is_mod_file, is_lib_file) = if reading_lib {
                root.pop(); // just remove lib.rs
                (false, true)
            } else {
                // we are reading .../mod_name.rs or .../mod_name/mod.rs
                // in both cases, next root is .../mod_name, so
                // * root == .../mod_name.rs => replace mod_name.rs to mod_name
                // * root == .../mod_name/mod.rs => just remove mod.rs
                let last = root.components().last().unwrap();
                match last {
                    Component::Normal(last) => {
                        if last == "mod.rs" {
                            root.pop();
                            (true, false)
                        } else {
                            root.pop();
                            root.push(&reading_mod_name);
                            (false, false)
                        }
                    }
                    _ => bail!(
                        "Unexpected component was taken from last of {}",
                        root.display()
                    ),
                }
            };

            // try open root/mod_name.rs and expand it
            root.push(format!("{mod_name}.rs"));
            let childs = if root.exists() {
                expand(root, OsString::from(mod_name), false)?
            } else {
                // try open root/mod_name/mod.rs and expand it
                root.pop();
                root.push(mod_name);
                if root.exists() {
                    expand(root, OsString::from(mod_name), false)?
                } else {
                    let a = root.clone();
                    root.pop();
                    root.push(format!("{mod_name}.rs"));
                    let b = root;
                    bail!(
                        "Failed to open module {mod_name}. Neither of {} nor {} is exist",
                        a.display(),
                        b.display()
                    );
                }
            };
            if is_pub {
                writeln!(res, "pub mod {mod_name} {{")?;
            } else {
                writeln!(res, "mod {mod_name} {{")?;
            }
            for line in childs.lines() {
                writeln!(res, "    {line}")?;
            }
            writeln!(res, "}}")?;
            root.pop();

            if is_mod_file {
                root.push("mod.rs");
            } else if is_lib_file {
                root.push("lib.rs");
            } else {
                root.pop();
                root.push(format!("{}.rs", reading_mod_name.to_str().unwrap()));
            }
        } else {
            writeln!(res, "{line}")?;
            continue;
        }
    }
    Ok(res)
}

fn main() -> Result<()> {
    let args = Args::parse();
    let (mut path_to_src_root, crate_name) = match canonicalize(&args.crate_path) {
        Ok(p) => {
            let maybe_crate_name = p
                .components()
                .last()
                .with_context(|| format!("No component found from given path"))?;
            match maybe_crate_name {
                Component::Normal(crate_name) => {
                    let crate_name = crate_name.to_owned();
                    (p, crate_name)
                }
                _ => bail!("todo"),
            }
        }
        Err(e) => {
            bail!("Failed to get absolute path of {}: {e}", args.crate_path);
        }
    };
    path_to_src_root.push("src");
    path_to_src_root.push("lib.rs");
    let expanded = expand(&mut path_to_src_root, OsString::from(""), true)?;
    let mut output: BufWriter<Box<dyn Write>> = match args.output {
        Some(output) => BufWriter::new(Box::new(File::create(output)?)),
        None => BufWriter::new(Box::new(std::io::stdout().lock())),
    };
    writeln!(
        output,
        "pub mod {} {{",
        crate_name
            .to_str()
            .unwrap()
            .chars()
            .map(|c| if c == '-' { '_' } else { c })
            .collect::<String>()
    )?;
    for line in expanded.lines() {
        writeln!(output, "    {}", line)?;
    }
    writeln!(output, "}}")?;
    Ok(())
}
