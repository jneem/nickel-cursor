use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use clap::Parser;
use nickel_cursor::{load_theme, render_cursor};

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
    let theme = load_theme(args.input)?;
    let theme_dir = args.out.join(&theme.name);
    let cursor_dir = theme_dir.join("cursors");
    std::fs::create_dir_all(&cursor_dir)?;
    for (name, cursor) in theme.cursors {
        let image = render_cursor(&cursor, &theme.style)?;
        let out_path = cursor_dir.join(name);
        let out_file = File::create(out_path)?;
        nickel_cursor::xcursor::write(out_file, &[image])?;
    }

    write_theme_file(theme_dir.join("cursor.theme"), &theme.name)?;
    write_theme_file(theme_dir.join("index.theme"), &theme.name)?;
    Ok(())
}
