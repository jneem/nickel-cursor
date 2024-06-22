use std::io;

use tiny_skia::Pixmap;

const HEADER_LEN: u32 = 16;
const TOC_ENTRY_LEN: u32 = 12;
const IMAGE_HEADER_LEN: u32 = 36;
const BYTES_PER_PIXEL: u32 = 4;
const IMAGE_TYPE: u32 = 0xfffd0002;
const IMAGE_VERSION: u32 = 1;

pub struct CursorImage {
    pub image: Pixmap,
    pub xhot: u32,
    pub yhot: u32,
}

trait WriteExt {
    fn write_u32(&mut self, x: u32) -> io::Result<()>;
}

impl<W: io::Write> WriteExt for W {
    fn write_u32(&mut self, x: u32) -> io::Result<()> {
        self.write_all(&x.to_le_bytes())
    }
}

pub fn write<W>(mut out: W, images: &[CursorImage]) -> anyhow::Result<()>
where
    W: std::io::Write,
{
    let n = images.len() as u32;

    write_header(&mut out, n)?;

    // Offset of the first image entry.
    let mut offset = HEADER_LEN + n * TOC_ENTRY_LEN;
    for img in images {
        write_toc_entry(&mut out, img.image.width(), offset)?;
        offset += IMAGE_HEADER_LEN + img.image.width() * img.image.height() * BYTES_PER_PIXEL;
    }
    for img in images {
        write_image(&mut out, img)?;
    }

    Ok(())
}

fn write_header(mut out: &mut dyn io::Write, count: u32) -> io::Result<()> {
    out.write_all(b"Xcur")?;

    // Number of bytes in the header.
    out.write_u32(HEADER_LEN)?;

    // File version number.
    out.write_u32(1u32)?;

    // Number of ToC entries.
    out.write_u32(count)
}

fn write_toc_entry(mut out: &mut dyn io::Write, size: u32, offset: u32) -> io::Result<()> {
    // We only support image entries in the ToC. (The other possible entry type is comments)
    out.write_u32(IMAGE_TYPE)?;
    out.write_u32(size)?;
    out.write_u32(offset)
}

fn write_image(mut out: &mut dyn io::Write, img: &CursorImage) -> io::Result<()> {
    out.write_u32(IMAGE_HEADER_LEN)?;
    out.write_u32(IMAGE_TYPE)?;
    out.write_u32(img.image.width())?; // nominal size
    out.write_u32(IMAGE_VERSION)?;
    out.write_u32(img.image.width())?;
    out.write_u32(img.image.height())?;
    out.write_u32(img.xhot)?;
    out.write_u32(img.yhot)?;
    out.write_u32(1)?; // delay
    for p in img.image.pixels() {
        out.write_all(&[p.blue(), p.green(), p.red(), p.alpha()])?;
    }
    Ok(())
}
