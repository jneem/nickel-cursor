use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context};
use clap::Parser;
use nickel_cursor::{load_theme, render_cursor, LoadError};

#[derive(Parser)]
struct Args {
    input: PathBuf,

    #[arg(short, long)]
    out: PathBuf,
}

fn write_theme_file(path: impl AsRef<Path>, name: &str) -> anyhow::Result<()> {
    let mut file = File::create(path.as_ref())?;
    writeln!(&mut file, "[Icon Theme]")?;
    writeln!(&mut file, "Name={name}")?;
    writeln!(&mut file, "Inherits=\"hicolor\"")?;
    Ok(())
}

pub fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let theme = match load_theme(args.input) {
        Ok(t) => t,
        Err(LoadError::Io { path, err }) => {
            eprintln!("Failed to read input file {}: {err}", path.display());
            bail!("Failed :(");
        }
        Err(LoadError::Nickel { mut prog, err }) => {
            prog.report(err, nickel_lang_core::error::report::ErrorFormat::Text);
            bail!("Failed :(");
        }
        Err(LoadError::Bug { err }) => {
            eprintln!(
                "Unexpected error while loading theme (probably a bug in nickel-cursor): {err}"
            );
            bail!("Failed :(");
        }
    };

    let theme_dir = args.out.join(&theme.name);
    let cursor_dir = theme_dir.join("cursors");
    std::fs::create_dir_all(&cursor_dir)
        .with_context(|| format!("when creating output directory {}", cursor_dir.display()))?;
    for (name, cursor) in theme.cursors {
        let images = render_cursor(&cursor, &theme.style)?;
        let out_path = cursor_dir.join(name);
        let out_file = File::create(&out_path)
            .with_context(|| format!("when opening file {}", out_path.display()))?;
        nickel_cursor::xcursor::write(out_file, &images)
            .with_context(|| format!("while rendering output file {}", out_path.display()))?;
    }

    for (name, target) in theme.links {
        let source_path = cursor_dir.join(name);
        // Make room for the symlink, if there's a file there already.
        let _ = std::fs::remove_file(&source_path);

        std::os::unix::fs::symlink(&target, &source_path).with_context(|| {
            format!(
                "while trying to link {} -> {}",
                source_path.display(),
                target
            )
        })?;
    }

    write_theme_file(theme_dir.join("cursor.theme"), &theme.name)?;
    write_theme_file(theme_dir.join("index.theme"), &theme.name)?;
    Ok(())
}
