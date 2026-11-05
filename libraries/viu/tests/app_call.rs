// SPDX-License-Identifier: MIT OR Apache-2.0
use mix::{BusHandler,MixResult,value::Value as MixValue,evaluator::{BusFuture,IncomingEvent}};
use serde_json::{Value,json};
use std::{collections::VecDeque,rc::Rc,sync::{Arc,Mutex,mpsc},time::Duration,path::PathBuf};
use viu::runtime::{Runtime,Program,from_json,to_json};
struct Scripted {
    replies:Arc<Mutex<VecDeque<(i32,Value)>>>,
    calls:Arc<Mutex<Vec<(String,String,Value)>>>,
}
impl BusHandler for Scripted {
    fn send<'a>(&'a self,target:&'a str,command:&'a str,args:&'a MixValue)->BusFuture<'a,MixResult<(i32,MixValue)>>{
        Box::pin(async move{
            self.calls.lock().unwrap().push((target.into(),command.into(),to_json(args)?));
            let (rc,body)=self.replies.lock().unwrap().pop_front().expect("unexpected extra effect");
            Ok((rc,from_json(body)?))
        })
    }
    fn emit<'a>(&'a self,_:&'a str,_:&'a str,_:&'a MixValue)->BusFuture<'a,MixResult<()>>{Box::pin(async{Ok(())})}
    fn port_exists<'a>(&'a self,_:&'a str)->BusFuture<'a,MixResult<bool>>{Box::pin(async{Ok(false)})}
    fn next_incoming<'a>(&'a self)->BusFuture<'a,Option<IncomingEvent>>{Box::pin(async{None})}
}
fn execute(replies:Vec<(i32,Value)>)->(u8,Value,Vec<(String,String,Value)>){
    let calls=Arc::new(Mutex::new(Vec::new()));
    let replies=Arc::new(Mutex::new(VecDeque::from(replies)));
    let (tx,rx)=mpsc::channel();
    let keep=calls.clone();
    let mut runtime=Runtime::new(4,Arc::new(move||{let _=tx.send(());}),Some(Box::new(move||Rc::new(Scripted{replies,calls}))));
    let path=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/dopus/effect-job.mix");
    runtime.load(Program{source:std::fs::read_to_string(&path).unwrap(),path,scene:"effect-test".into(),citizen:"test".into()}).unwrap();
    rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(runtime.try_completion().unwrap().error.is_none());
    runtime.event("job.run".into(),json!({"kind":"mail","paths":["/home/user/report.pdf"]})).unwrap();
    rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let done=runtime.try_completion().unwrap();
    assert!(done.error.is_none(),"{:?}",done.error);
    let reply=done.reply.unwrap();
    let calls=keep.lock().unwrap().clone();
    (reply.rc,reply.body,calls)
}
#[test]
fn mail_falls_back_only_after_explicit_busy_and_keeps_the_same_paths(){
    let (rc,body,calls)=execute(vec![(10,json!({"error_code":"BUSY","message":"composing"})),(0,json!({}))]);
    assert_eq!(rc,0);assert_eq!(body["added"],true);assert_eq!(calls.len(),2);
    assert_eq!(calls[0].1,"email.compose");assert_eq!(calls[1].1,"email.compose.attachment.add");
    assert_eq!(calls[0].2["body"]["attachments"],calls[1].2["body"]["paths"]);
    for refusal in [(10,json!({"error_code":"DENIED"})),(-2,json!("reply timed out"))]{
        let (rc,body,calls)=execute(vec![refusal]);
        assert!(rc>=10);assert_eq!(calls.len(),1);
        assert_ne!(body["error_code"],Value::Null);
    }
}
#[test]
fn missing_service_launches_once_and_never_retries_a_delivered_compose(){
    let (rc,_,calls)=execute(vec![(10,json!("Service 'email' not found")),(0,json!({})),(0,json!({}))]);
    assert_eq!(rc,0);assert_eq!(calls.len(),3);assert_eq!(calls[1].0,"apps");assert_eq!(calls[1].1,"apps.launch");
    assert_eq!(calls[1].2["body"]["id"],"dev.mixos.email");assert_eq!(calls[0].2,calls[2].2);
}
