// SPDX-License-Identifier: MIT OR Apache-2.0
//! Reusable raster documents. Mix owns tools, operations, dialogs and storage policy.
pub mod document;
pub mod paint;
pub mod viewport;

use document::{Document,Shape,Point,Crop};
use serde_json::{Value,json};
use std::{collections::BTreeMap,sync::{Arc,Mutex,atomic::{AtomicU64,Ordering}}};

#[derive(Clone,Default)]
pub struct Documents(Arc<Mutex<BTreeMap<String,Document>>>);
static NEXT_DOCUMENT:AtomicU64=AtomicU64::new(1);
impl Documents {
    pub(crate) fn binding(self)->mix::evaluator::ExtFn {
        mix::sync_ext(move |args| {
            let fail=|msg:String|mix::MixError::RuntimeError{msg,span:None};
            let [mix::value::Value::String(op),body]=args.as_slice() else {return Err(fail("viu_document expects operation and object".into()));};
            let body=crate::runtime::to_json(body)?;
            self.call(op,&body).map_err(fail).and_then(crate::runtime::from_json)
        })
    }
    fn call(&self,op:&str,args:&Value)->Result<Value,String> {
        let mut docs=self.0.lock().map_err(|_|"document owner failed")?;
        if op=="open" {
            if docs.len()>=4 {return Err("document capacity reached".into());}
            let path=args["path"].as_str().ok_or("expected path")?;
            let doc=Document::open(std::path::Path::new(path))?;
            let handle=format!("document-{}",NEXT_DOCUMENT.fetch_add(1,Ordering::Relaxed));
            let info=doc.info();docs.insert(handle.clone(),doc);
            return Ok(json!({"handle":handle,"document":info}));
        }
        let handle=args["handle"].as_str().ok_or("expected document handle")?;
        if op=="close" {return Ok(json!({"closed":docs.remove(handle).is_some()}));}
        let doc=docs.get_mut(handle).ok_or("stale document handle")?;
        let mut result=json!({});
        match op {
            "info"=>{},
            "annotate"=>{let shape:Shape=serde_json::from_value(args["shape"].clone()).map_err(|e|e.to_string())?;result["id"]=json!(doc.add(shape)?);},
            "move"=>doc.move_object(number(args,"id")?,scalar(args,"dx")?,scalar(args,"dy")?)?,
            "delete"=>doc.delete(number(args,"id")?)?,
            "crop"=>{let crop:Option<Crop>=serde_json::from_value(args["crop"].clone()).map_err(|e|e.to_string())?;doc.set_crop(crop)?;},
            "undo"=>{if !doc.undo(){return Err("nothing to undo".into());}},
            "redo"=>{if !doc.redo(){return Err("nothing to redo".into());}},
            "saved"=>doc.mark_saved(),
            "hit"=>{
                let p:Point=serde_json::from_value(args["point"].clone()).map_err(|e|e.to_string())?;
                if !p.valid(){return Err("invalid hit point".into());}
                return Ok(json!({"id":doc.hit(p,scalar(args,"tolerance")?),"next_number":doc.next_number()}));
            },
            "export"=>{
                let snapshot=doc.clone();drop(docs);
                let path=args["path"].as_str().ok_or("expected path")?;
                let face=paint::Face::new(include_bytes!("../../assets/fonts/Inter-Regular.ttf"))?;
                snapshot.export(std::path::Path::new(path),&face)?;
                return Ok(json!({"path":path,"document":snapshot.info()}));
            },
            _=>return Err("unknown document operation".into())
        }
        result["document"]=doc.info();
        Ok(result)
    }
}
fn number(args:&Value,key:&str)->Result<u64,String>{args[key].as_u64().ok_or_else(||format!("expected {key}"))}
fn scalar(args:&Value,key:&str)->Result<f32,String>{args[key].as_f64().filter(|n|n.is_finite()&&n.abs()<=f32::MAX as f64).map(|n|n as f32).ok_or_else(||format!("expected finite {key}"))}
