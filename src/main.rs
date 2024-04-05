mod item;

use crate::item::remove_doc_attrs;
use anyhow::{bail, Context, Result};
use clap::Parser;
use quote::ToTokens;
use std::{
    fmt::Write as _,
    fs::{self, File},
    io::{stdout, Write},
    path::{Path, PathBuf},
    process::Command,
};
use syn::{parse_file, Attribute, Ident, Item, Visibility};
use tempfile::NamedTempFile;
use toml::{from_str, Table};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to library crate to expand
    crate_path: String,

    /// Format expanded code. rustfmt must be executable
    #[arg(short, long)]
    format: bool,

    /// Remove all modules and functions which #[cfg(test)] is attached
    #[arg(long)]
    remove_test: bool,

    /// Remove doc comments
    #[arg(long)]
    remove_doc_comment: bool,

    /// Path to file to paste expanded code
    #[arg(short, long, value_name = "INPUT")]
    input: Option<PathBuf>,

    /// Path to file to paste generated code
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,
}

fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path
        .as_ref()
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", path.as_ref().display()))?;
    fs::read_to_string(&path)
        .with_context(|| format!("failed to read content of {}", path.display()))
}

// TODO: Convert #[doc = "..."] to /// ...
//
/// Expand modules in `path`/`name.rs` and return it as string.
fn expand(
    name: &str,
    path: &mut PathBuf,
    expanding_lib: bool,
    remove_test: bool,
    remove_doc_comment: bool,
) -> Result<String> {
    let content = {
        path.push(name);
        path.set_extension("rs");
        let content = read_to_string(&path)
            .with_context(|| format!("failed to read content of {}", path.display()))?;
        path.pop();
        content
    };
    let ast = parse_file(&content)
        .with_context(|| format!("failed to parse content of {}", path.display()))?;
    let mut res = String::new();
    if let Some(shebang) = ast.shebang {
        writeln!(res, "{shebang}")?;
    }
    for attr in ast.attrs {
        if attr.path().is_ident("doc") && remove_doc_comment {
            continue;
        }
        let attr = attr_to_string(attr)
            .with_context(|| format!("failed to convert attribute of module {}", name))?;
        writeln!(res, "{}", attr)?;
    }
    for item in ast.items {
        let string = item_to_string(
            name,
            path,
            expanding_lib,
            remove_test,
            remove_doc_comment,
            item,
        )
        .with_context(|| format!("failed to convert item of module {}", name))?;
        writeln!(res, "{}", string)?;
    }
    Ok(res)
}

/// Convert given attribute into string.
fn attr_to_string(attr: Attribute) -> Result<String> {
    Ok(attr.to_token_stream().to_string())
}

/// Convert given item into string.
fn item_to_string(
    name: &str,
    path: &mut PathBuf,
    expanding_lib: bool,
    remove_test: bool,
    remove_doc_comment: bool,
    item: Item,
) -> Result<String> {
    let item = if remove_doc_comment {
        remove_doc_attrs(item)
    } else {
        item
    };
    let Item::Mod(module) = item else {
        return Ok(item.to_token_stream().to_string());
    };

    // If one of attributes is #[cfg(test)], ignore the module and just return empty vector.
    // Then, if the module is mod name { .. }, just write the content and return it.
    let mut res = String::new();
    for attr in module.attrs {
        let string = attr_to_string(attr.clone())
            .with_context(|| format!("failed to convert attribute of module {}", name))?;
        if !remove_test {
            writeln!(res, "{string}")?;
            continue;
        }
        if attr.path().is_ident("cfg") && remove_test {
            let Ok(args) = attr.parse_args::<Ident>() else {
                writeln!(res, "{string}")?;
                continue;
            };
            if args.to_string() == "test" {
                return Ok(String::new());
            }
        }
        writeln!(res, "{string}")?;
    }
    if let Some(mod_content) = module.content {
        for item in mod_content.1 {
            let string = item_to_string(name, path, false, remove_test, remove_doc_comment, item)
                .with_context(|| format!("failed to convert item of module {}", name))?;
            writeln!(res, "{}", string)?;
        }
        return Ok(res);
    }

    // TODO: We don't expect mod_name/mod.rs exist instead of mod_name.rs exist.
    let mod_name = module.ident.to_string();
    if !expanding_lib {
        path.push(name);
    }
    let mod_content = expand(&mod_name, path, false, remove_test, remove_doc_comment)
        .with_context(|| format!("failed to expand child module {} of {}", mod_name, name))?;
    if !expanding_lib {
        path.pop();
    }

    // Write expanded child modules.
    match module.vis {
        Visibility::Inherited => write!(res, "mod {mod_name} {{")?,
        Visibility::Public(_) => write!(res, "pub mod {mod_name} {{")?,
        _ => bail!("pub(..) is not supported"),
    }
    writeln!(res, "{mod_content} }}")?;
    Ok(res)
}

/// Format given rust source code using rustfmt.
fn format_code(code: String) -> Result<String> {
    let mut file = NamedTempFile::new().context("failed to create temporary file")?;
    file.write_all(code.as_bytes())
        .context("failed to write unformatted code to temporary file")?;
    Command::new("rustfmt")
        .arg(file.path())
        .output()
        .context("failed to format code")?;
    Ok(read_to_string(file.path()).context("failed to read formatted code from temporary file")?)
}

/// Convert crate name that can be used in source code.
fn normalize_crate_name(name: &str) -> String {
    name.chars()
        .map(|c| if c == '-' { '_' } else { c })
        .collect()
}

/// Get crate name from Cargo.toml, or from given path if failed.
fn fetch_crate_name(crate_path: &mut PathBuf) -> Result<String> {
    crate_path.push("Cargo.toml");
    if let Some(crate_name) = try_read_crate_name_from_carto_toml(&crate_path) {
        Ok(crate_name)
    } else {
        crate_path.pop();
        crate_path
            .file_name()
            .context("failed to extract crate name from path")?
            .to_str()
            .map(|name| name.to_string())
            .context("invalid utf-8 encoded crate name")
    }
}

/// Attempt to read package.name from Cargo.toml.
fn try_read_crate_name_from_carto_toml(cargo_toml_path: &PathBuf) -> Option<String> {
    let content = read_to_string(cargo_toml_path).ok()?;
    let toml = from_str::<Table>(&content).ok()?;
    toml.get("package")?
        .as_table()?
        .get("name")?
        .as_str()
        .map(|name| name.to_string())
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut src_path = PathBuf::from(&args.crate_path);
    src_path.push("src");
    let expanded = expand(
        "lib",
        &mut src_path,
        true,
        args.remove_test,
        args.remove_doc_comment,
    )
    .context("failed to expand lib.rs")?;

    let code = {
        let mut code = String::new();
        if let Some(path) = args.input {
            writeln!(
                code,
                "{}",
                read_to_string(path).context("failed to read input file")?
            )?;
        }
        let mut path = PathBuf::from(args.crate_path);
        let crate_name = fetch_crate_name(&mut path)?;
        writeln!(
            code,
            "pub mod {} {{ {expanded} }}",
            normalize_crate_name(&crate_name)
        )?;
        if args.format {
            format_code(code).context("failed to format generated code")?
        } else {
            code
        }
    };

    if let Some(path) = args.output {
        File::create(path)
            .context("failed to create output file")?
            .write_all(code.as_bytes())
            .context("failed to write generated code to output file")?;
    } else {
        stdout()
            .lock()
            .write_all(code.as_bytes())
            .context("failed to write generated code to stdout")?;
    }
    Ok(())
}
