use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use nickel_lang_core::{eval::cache::CacheImpl, program::Program};
use tiny_skia::{Mask, Paint, PathBuilder, Pixmap, Stroke, Transform};
use xcursor::CursorImage;

pub mod xcursor;

#[derive(serde::Deserialize, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(serde::Deserialize, Debug)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

#[derive(serde::Deserialize, Debug)]
pub struct Cursor {
    paths: Vec<String>,
    hot: Point,
    rotation_degrees: f64,
}

#[derive(serde::Deserialize, Debug)]
pub struct Style {
    pub sizes: Vec<u32>,
    pub fill_color: Color,
    pub stroke_width: f64,
    pub stroke_color: Color,
}

#[derive(serde::Deserialize, Debug)]
pub struct CursorTheme {
    pub cursors: HashMap<String, Cursor>,
    pub style: Style,
    pub name: String,
    pub links: HashMap<String, String>,
}

pub enum LoadError {
    Io {
        path: PathBuf,
        err: std::io::Error,
    },
    Nickel {
        prog: Program<CacheImpl>,
        err: nickel_lang_core::error::Error,
    },
    Bug {
        err: anyhow::Error,
    },
}

impl From<serde_json::Error> for LoadError {
    fn from(err: serde_json::Error) -> Self {
        LoadError::Bug { err: err.into() }
    }
}

pub fn load_theme(path: impl AsRef<Path>) -> Result<CursorTheme, LoadError> {
    let mut prog: Program<CacheImpl> =
        Program::new_from_file(path.as_ref().to_owned(), std::io::stderr()).map_err(|err| {
            LoadError::Io {
                path: path.as_ref().to_owned(),
                err,
            }
        })?;
    if let Ok(nickel_path) = std::env::var("NICKEL_IMPORT_PATH") {
        prog.add_import_paths(nickel_path.split(':'));
    }
    let export = prog
        .eval_full_for_export()
        .map_err(|err| LoadError::Nickel { prog, err })?;
    let json = serde_json::to_string(&export)?;
    Ok(serde_json::from_str(&json)?)
}

fn parse_path(data: &str) -> anyhow::Result<tiny_skia::Path> {
    let mut path = PathBuilder::new();
    for seg in svgtypes::SimplifyingPathParser::from(data) {
        match seg? {
            svgtypes::SimplePathSegment::MoveTo { x, y } => path.move_to(x as f32, y as f32),
            svgtypes::SimplePathSegment::LineTo { x, y } => path.line_to(x as f32, y as f32),
            svgtypes::SimplePathSegment::Quadratic { x1, y1, x, y } => {
                path.quad_to(x1 as f32, y1 as f32, x as f32, y as f32)
            }
            svgtypes::SimplePathSegment::CurveTo {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => path.cubic_to(
                x1 as f32, y1 as f32, x2 as f32, y2 as f32, x as f32, y as f32,
            ),
            svgtypes::SimplePathSegment::ClosePath => path.close(),
        }
    }
    Ok(path.finish().unwrap())
}

fn transform(deg: f64, nominal_scale: f64) -> Transform {
    Transform::identity()
        .post_rotate(deg as f32)
        .post_scale(nominal_scale as f32 / 256.0, nominal_scale as f32 / 256.0)
}

pub fn render_cursor(cursor: &Cursor, style: &Style) -> anyhow::Result<Vec<CursorImage>> {
    style
        .sizes
        .iter()
        .map(|s| render_cursor_one_size(cursor, style, *s))
        .collect()
}

fn render_cursor_one_size(
    cursor: &Cursor,
    style: &Style,
    size: u32,
) -> anyhow::Result<CursorImage> {
    let mut pixmap = Pixmap::new(size, size).unwrap();

    let mut paint = Paint::default();
    let Color { r, g, b, a } = style.fill_color;
    paint.set_color_rgba8(r as u8, g as u8, b as u8, a as u8);

    let mut mask = Mask::new(size, size).unwrap();
    let transform = transform(cursor.rotation_degrees, size as f64);

    for path in &cursor.paths {
        let path = parse_path(path)?;
        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, transform, None);
        mask.fill_path(&path, tiny_skia::FillRule::Winding, true, transform);
    }
    mask.invert();

    let Color { r, g, b, a } = style.stroke_color;
    paint.set_color_rgba8(r as u8, g as u8, b as u8, a as u8);
    let stroke = Stroke {
        width: style.stroke_width as f32,
        miter_limit: 0.0,
        line_cap: tiny_skia::LineCap::Round,
        line_join: tiny_skia::LineJoin::Round,
        dash: None,
    };
    for path in &cursor.paths {
        let path = parse_path(path)?;
        pixmap.stroke_path(&path, &paint, &stroke, transform, Some(&mask));
    }

    let mut hot_p = tiny_skia::Point {
        x: cursor.hot.x as f32 - 128.0,
        y: cursor.hot.y as f32 - 128.0,
    };
    transform.map_point(&mut hot_p);

    Ok(CursorImage {
        image: pixmap,
        xhot: (hot_p.x.round() + size as f32 / 2.0) as u32,
        yhot: (hot_p.y.round() + size as f32 / 2.0) as u32,
    })
}
