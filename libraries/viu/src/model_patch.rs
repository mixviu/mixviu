// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded scene-data edits. Paths are typed components, never expressions.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

pub const MAX_PATCH_BYTES: usize = 1024 * 1024;
pub const MAX_OPERATIONS: usize = 512;
pub const MAX_REVISION: u64 = (1 << 53) - 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Part { Key(String), Index(usize) }

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Set { path: Vec<Part>, value: Value },
    Remove { path: Vec<Part> },
    Splice { path: Vec<Part>, index: usize, remove: usize, items: Vec<Value> },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Patch {
    pub generation: u64,
    pub base: u64,
    pub revision: u64,
    pub operations: Vec<Operation>,
}

fn bytes(value: &Value) -> usize { serde_json::to_vec(value).expect("JSON value").len() }

impl Operation {
    pub fn path(&self) -> &[Part] {
        match self { Self::Set{path,..}|Self::Remove{path}|Self::Splice{path,..} => path }
    }
}

impl Patch {
    pub fn decode(value: Value) -> Result<Self,String> {
        if bytes(&value)>MAX_PATCH_BYTES { return Err("patch exceeds 1 MiB".into()); }
        let patch:Self=serde_json::from_value(value).map_err(|e|e.to_string())?;
        if patch.base==0 || patch.revision>MAX_REVISION || patch.base.checked_add(1)!=Some(patch.revision) {
            return Err("patch must advance a positive exact revision by one".into());
        }
        if patch.operations.is_empty() || patch.operations.len()>MAX_OPERATIONS {
            return Err("patch requires 1..512 operations".into());
        }
        for operation in &patch.operations {
            let path=operation.path();
            if path.is_empty() || path.len()>32 || !matches!(path[0],Part::Key(_)) || path.iter().any(|p|matches!(p,Part::Key(k) if k.len()>256)) {
                return Err("patch path requires a root key and at most 32 bounded components".into());
            }
        }
        Ok(patch)
    }
    pub fn roots(&self) -> BTreeSet<String> {
        self.operations.iter().map(|op|match &op.path()[0] {Part::Key(k)=>k.clone(),_=>unreachable!()}).collect()
    }
}

fn at<'a>(mut value: &'a mut Value, parts: &[Part]) -> Result<&'a mut Value,String> {
    for part in parts {
        value=match part {
            Part::Key(key)=>value.as_object_mut().and_then(|m|m.get_mut(key)),
            Part::Index(index)=>value.as_array_mut().and_then(|a|a.get_mut(*index)),
        }.ok_or("patch path has a missing parent or wrong component type")?;
    }
    Ok(value)
}

enum Restore {
    Field { path:Vec<Part>, before:Option<Value> },
    Slot { path:Vec<Part>, before:Value },
    Range { path:Vec<Part>, index:usize, inserted:usize, before:Vec<Value> },
}

/// Owns only displaced values. Unchanged model data is neither cloned nor
/// serialised; callers commit after their owning validation succeeds.
pub struct Edit { undo:Vec<Restore>, pub bytes:usize }
impl Edit {
    pub fn apply(model:&mut Value, current_bytes:usize, patch:&Patch) -> Result<Self,String> {
        let mut edit=Self{undo:Vec::new(),bytes:current_bytes};
        for operation in &patch.operations {
            if let Err(error)=edit.one(model,operation) { edit.rollback(model);return Err(error); }
            if edit.bytes>crate::view::MAX_MODEL_BYTES {
                edit.rollback(model);return Err("scene model exceeds 8 MiB data budget".into());
            }
        }
        Ok(edit)
    }
    fn one(&mut self,model:&mut Value,operation:&Operation)->Result<(),String> {
        match operation {
            Operation::Set{path,value}=>{
                let (last,parent)=path.split_last().expect("checked path");
                let target=at(model,parent)?;
                match last {
                    Part::Key(key)=>{
                        let map=target.as_object_mut().ok_or("set key requires an object")?;
                        let before=map.insert(key.clone(),value.clone());
                        if let Some(old)=&before {self.bytes=self.bytes-bytes(old)+bytes(value);}
                        else {self.bytes+=bytes(&Value::String(key.clone()))+1+bytes(value)+usize::from(map.len()>1);}
                        self.undo.push(Restore::Field{path:path.clone(),before});
                    }
                    Part::Index(index)=>{
                        let slot=target.as_array_mut().and_then(|a|a.get_mut(*index)).ok_or("set index requires an existing list element")?;
                        let before=std::mem::replace(slot,value.clone());
                        self.bytes=self.bytes-bytes(&before)+bytes(value);
                        self.undo.push(Restore::Slot{path:path.clone(),before});
                    }
                }
            }
            Operation::Remove{path}=>{
                let (last,parent)=path.split_last().expect("checked path");
                let Part::Key(key)=last else {return Err("remove requires an object key; use splice for lists".into());};
                let map=at(model,parent)?.as_object_mut().ok_or("remove requires an object")?;
                let before=map.remove(key).ok_or("remove requires an existing key")?;
                self.bytes-=bytes(&Value::String(key.clone()))+1+bytes(&before)+usize::from(!map.is_empty());
                self.undo.push(Restore::Field{path:path.clone(),before:Some(before)});
            }
            Operation::Splice{path,index,remove,items}=>{
                let array=at(model,path)?.as_array_mut().ok_or("splice requires a list")?;
                if *index>array.len() || *remove>array.len()-index {return Err("splice range is outside the list".into());}
                let old_commas=array.len().saturating_sub(1);
                let before:Vec<_>=array.splice(*index..index+remove,items.iter().cloned()).collect();
                self.bytes=self.bytes-before.iter().map(bytes).sum::<usize>()-old_commas
                    +items.iter().map(bytes).sum::<usize>()+array.len().saturating_sub(1);
                self.undo.push(Restore::Range{path:path.clone(),index:*index,inserted:items.len(),before});
            }
        }
        Ok(())
    }
    pub fn rollback(mut self,model:&mut Value) {
        for undo in self.undo.drain(..).rev() {
            match undo {
                Restore::Field{path,before}=>{
                    let (last,parent)=path.split_last().unwrap();let Part::Key(key)=last else {unreachable!()};
                    let map=at(model,parent).expect("undo parent").as_object_mut().unwrap();
                    if let Some(value)=before {map.insert(key.clone(),value);} else {map.remove(key);}
                }
                Restore::Slot{path,before}=>{*at(model,&path).expect("undo slot")=before;}
                Restore::Range{path,index,inserted,before}=>{
                    at(model,&path).expect("undo list").as_array_mut().unwrap().splice(index..index+inserted,before);
                }
            }
        }
    }
}

pub fn model_bytes(model:&Value)->usize {bytes(model)}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn patch(operations:Value)->Patch {Patch::decode(json!({"generation":1,"base":1,"revision":2,"operations":operations})).unwrap()}
    #[test]
    fn edits_account_exact_bytes_and_rollback_in_reverse_order() {
        let initial=json!({"a/b":{"~":[{"id":"old"},2,3]},"empty":{},"unicode":"é"});
        let mut model=initial.clone();
        let patch=patch(json!([
            {"op":"set","path":["a/b","~",0,"id"],"value":"new"},
            {"op":"splice","path":["a/b","~"],"index":1,"remove":2,"items":["四",{},null]},
            {"op":"set","path":["empty","quote\""],"value":[1]},
            {"op":"remove","path":["unicode"]},
            {"op":"set","path":["last"],"value":false}
        ]));
        let size=bytes(&model);let edit=Edit::apply(&mut model,size,&patch).unwrap();
        assert_eq!(edit.bytes,bytes(&model));
        edit.rollback(&mut model);assert_eq!(model,initial);
    }
    #[test]
    fn later_invalid_operation_restores_earlier_mutations() {
        for bad in [json!({"op":"remove","path":["a",0]}),json!({"op":"set","path":["a","0"],"value":3}),json!({"op":"splice","path":["a"],"index":2,"remove":0,"items":[]})] {
            let initial=json!({"a":[1]});let mut model=initial.clone();let size=bytes(&model);
            let patch=patch(json!([{"op":"set","path":["new"],"value":42},bad]));
            assert!(Edit::apply(&mut model,size,&patch).is_err());assert_eq!(model,initial);
        }
    }
    #[test]
    fn refuses_unbounded_and_ambiguous_operations_before_editing() {
        for operations in [json!([]),json!([{"op":"set","path":[],"value":3}]),json!([{"op":"set","path":["a"],"value":3,"extra":true}]),json!([{"op":"splice","path":["a"],"index":-1,"remove":0,"items":[]}])] {
            assert!(Patch::decode(json!({"generation":1,"base":1,"revision":2,"operations":operations})).is_err());
        }
    }
}
