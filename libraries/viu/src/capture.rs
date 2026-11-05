// SPDX-License-Identifier: MIT OR Apache-2.0
//! One bounded native application readback, with fresh-file output and a
//! deadline. This endpoint proves GPU readback and PNG completion, never scanout.
use egui::{ColorImage,Context,ScreenshotCallback,ViewportCommand};
use serde_json::{Value,json};
use std::{path::{Path,PathBuf},sync::{Arc,Mutex},time::{Duration,Instant}};
#[derive(Default)]
struct Shared{live:Option<u64>,image:Option<Arc<ColorImage>>}
struct Pending{token:u64,path:PathBuf,deadline:Instant}
pub struct Capture{root:PathBuf,shared:Arc<Mutex<Shared>>,pending:Option<Pending>,next:u64,wake:crate::runtime::Wake}
impl Capture{
    pub fn new(root:PathBuf,wake:crate::runtime::Wake)->Self{Self{root,shared:Arc::new(Mutex::new(Shared::default())),pending:None,next:0,wake}}
    pub fn request(&mut self,ctx:&Context,name:Option<&str>)->Result<u64,String>{
        if self.pending.is_some(){return Err("BUSY: screenshot already pending".into());}
        self.next=self.next.checked_add(1).ok_or("capture token exhausted")?;
        let generated=format!("window-{}-{}.png",std::process::id(),self.next);
        let name=name.unwrap_or(&generated);
        if name.len()>128||name.len()<=4||!name.ends_with(".png")||name.chars().any(|c|c.is_control()||c=='/'||c=='\\'){return Err("Capture name must be a plain .png file name".into());}
        if !self.root.is_absolute(){return Err("Capture runtime directory is unavailable".into());}
        let scale=ctx.pixels_per_point();
        let pixels=ctx.input(|i|i.viewport().inner_rect.map(|r|r.area()*scale.powi(2)));
        if pixels.is_some_and(|n|!n.is_finite()||n>16_777_216.0){return Err("Capture exceeds sixteen megapixels".into());}
        std::fs::create_dir_all(&self.root).map_err(|e|e.to_string())?;
        let path=self.root.join(name);
        if std::fs::symlink_metadata(&path).is_ok(){return Err("Capture destination already exists".into());}
        let token=self.next;let shared=self.shared.clone();let wake=self.wake.clone();
        {let mut state=shared.lock().unwrap_or_else(|e|e.into_inner());state.live=Some(token);state.image=None;}
        self.pending=Some(Pending{token,path,deadline:Instant::now()+Duration::from_secs(4)});
        ctx.send_viewport_cmd(ViewportCommand::Screenshot(ScreenshotCallback::new(move|image|{
            let mut state=shared.lock().unwrap_or_else(|e|e.into_inner());
            if state.live==Some(token){state.image=Some(image);drop(state);wake();}
        })));
        ctx.request_repaint();Ok(token)
    }
    pub fn poll(&mut self,ctx:&Context)->Option<(u64,Result<Value,String>)>{
        let pending=self.pending.as_ref()?;
        if Instant::now()>=pending.deadline{
            let token=pending.token;self.pending=None;let mut state=self.shared.lock().unwrap_or_else(|e|e.into_inner());state.live=None;state.image=None;
            return Some((token,Err("CAPTURE_TIMEOUT: no native readback before deadline".into())));
        }
        ctx.request_repaint_after(pending.deadline.saturating_duration_since(Instant::now()));
        let image=self.shared.lock().unwrap_or_else(|e|e.into_inner()).image.take()?;
        let pending=self.pending.take().unwrap();self.shared.lock().unwrap_or_else(|e|e.into_inner()).live=None;
        let result=write_png(&pending.path,&image).map(|_|json!({"path":pending.path,"width":image.size[0],"height":image.size[1],"source":"application-gpu-readback","endpoint":"PNG written; not physical presentation"}));
        Some((pending.token,result))
    }
}
impl Drop for Capture{fn drop(&mut self){let mut state=self.shared.lock().unwrap_or_else(|e|e.into_inner());state.live=None;state.image=None;}}
// The encoder and exclusive-file contract follow the retained toolkit drive
// primitive; its origin and the bounded adaptation are in provenance.
fn write_png(path:&Path,image:&ColorImage)->Result<(),String>{
    use std::os::unix::fs::OpenOptionsExt;
    if image.size[0].checked_mul(image.size[1]).is_none_or(|n|n>16_777_216){return Err("Capture exceeds sixteen megapixels".into());}
    let file=std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path).map_err(|e|e.to_string())?;
    let encode=||->Result<(),String>{
        let mut encoder=png::Encoder::new(std::io::BufWriter::new(file),image.size[0] as u32,image.size[1] as u32);encoder.set_color(png::ColorType::Rgba);encoder.set_depth(png::BitDepth::Eight);encoder.set_compression(png::Compression::Fast);
        let mut writer=encoder.write_header().map_err(|e|e.to_string())?;
        let data:Vec<u8>=image.pixels.iter().flat_map(|p|p.to_srgba_unmultiplied()).collect();
        writer.write_image_data(&data).map_err(|e|e.to_string())?;writer.finish().map_err(|e|e.to_string())
    };
    let result=encode();if result.is_err(){let _=std::fs::remove_file(path);}result
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn capture_owns_one_readback_refuses_overwrite_and_writes_a_complete_png(){
        let dir=tempfile::tempdir().unwrap();let ctx=Context::default();let mut capture=Capture::new(dir.path().into(),Arc::new(||{}));
        assert!(capture.request(&ctx,Some("../escape.png")).is_err());
        let token=capture.request(&ctx,Some("native.png")).unwrap();assert!(capture.request(&ctx,None).is_err());
        capture.shared.lock().unwrap().image=Some(Arc::new(ColorImage::filled([2,3],egui::Color32::GREEN)));
        let (finished,result)=capture.poll(&ctx).unwrap();assert_eq!(finished,token);assert_eq!(result.unwrap()["height"],3);
        assert_eq!(thumbs::png::size(&std::fs::read(dir.path().join("native.png")).unwrap()),Some((2,3)));
        assert!(capture.request(&ctx,Some("native.png")).is_err());
    }
    #[test]
    fn expired_or_dropped_capture_never_writes_a_late_callback(){
        let dir=tempfile::tempdir().unwrap();let ctx=Context::default();let mut capture=Capture::new(dir.path().into(),Arc::new(||{}));
        capture.request(&ctx,Some("late.png")).unwrap();capture.pending.as_mut().unwrap().deadline=Instant::now();
        capture.shared.lock().unwrap().image=Some(Arc::new(ColorImage::filled([1,1],egui::Color32::RED)));
        assert!(capture.poll(&ctx).unwrap().1.is_err());assert!(!dir.path().join("late.png").exists());
        capture.request(&ctx,Some("dropped.png")).unwrap();let shared=capture.shared.clone();drop(capture);assert!(shared.lock().unwrap().live.is_none());assert!(!dir.path().join("dropped.png").exists());
    }
}
