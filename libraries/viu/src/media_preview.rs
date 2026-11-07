// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded still-image decoding. Callers own fetch admission and lifetime.
use std::path::Path;
pub fn decode(path: &Path, mime: &str) -> Result<egui::ColorImage, String> {
    let kind = match mime.trim().to_ascii_lowercase().as_str() {
        "image/png" => image::ImageFormat::Png,
        "image/jpeg" => image::ImageFormat::Jpeg,
        "image/gif" => image::ImageFormat::Gif,
        "image/webp" => image::ImageFormat::WebP,
        _ => return Err("unsupported image format".into()),
    };
    let bytes=thumbs::cache::read_regular(path,16*1024*1024).map_err(|e|e.to_string())?;
    let mut reader=image::ImageReader::with_format(std::io::Cursor::new(bytes),kind);
    let mut limits=image::Limits::default();
    limits.max_image_width=Some(8192);limits.max_image_height=Some(8192);limits.max_alloc=Some(256*1024*1024);
    reader.limits(limits);
    let decoded=reader.decode().map_err(|e|e.to_string())?;
    let display=if decoded.width().max(decoded.height())>2048 {decoded.thumbnail(2048,2048)} else {decoded};
    let rgba=display.into_rgba8();
    Ok(egui::ColorImage::from_rgba_unmultiplied([rgba.width() as usize,rgba.height() as usize],rgba.as_raw()))
}
pub(crate) fn binding() -> mix::evaluator::ExtFn {
    mix::sync_ext(|args| {
        let fail=|msg:String|mix::MixError::RuntimeError{msg,span:None};
        let [mix::value::Value::String(path),mix::value::Value::String(mime),mix::value::Value::String(destination)]=args.as_slice() else {return Err(fail("viu_preview expects source, MIME and destination".into()));};
        if !Path::new(path).is_absolute() || !Path::new(destination).is_absolute() {return Err(fail("preview paths must be absolute".into()));}
        let image=decode(Path::new(path),mime).map_err(fail)?;
        crate::capture::write_png(Path::new(destination),&image).map_err(fail)?;
        crate::runtime::from_json(serde_json::json!({"width":image.size[0],"height":image.size[1]}))
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn still_image_limits_and_mime_selection_precede_display_allocation() {
        let dir=tempfile::tempdir().unwrap();let path=dir.path().join("image.png");
        image::RgbImage::from_pixel(4096,1024,image::Rgb([240,0,0])).save(&path).unwrap();
        assert_eq!(decode(&path,"image/png").unwrap().size,[2048,512]);
        assert!(decode(&path,"image/jpeg").is_err());assert!(decode(&path,"image/svg+xml").is_err());
        let link=dir.path().join("link.png");std::os::unix::fs::symlink(&path,&link).unwrap();assert!(decode(&link,"image/png").is_err());
    }
}
