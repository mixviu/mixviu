// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native supervised Bus ingress. No application socket or alternate transport.
use bus::{SupervisedClient, native_client::BoundedIncomingEvent};
use serde_json::{Value, json};
use std::sync::{Arc, mpsc};
use tokio::sync::oneshot;
pub struct Request {
    pub command: String,
    pub body: Value,
    pub reply: oneshot::Sender<(u8, Value)>,
}
pub struct NativeBus {
    pub requests: mpsc::Receiver<Request>,
    stop: Option<oneshot::Sender<()>>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl NativeBus {
    pub fn start(name: String, url: String, wake: Arc<dyn Fn() + Send + Sync>) -> Self {
        let (tx, requests) = mpsc::sync_channel(32);
        let (stop, mut stopped) = oneshot::channel();
        let thread=std::thread::Builder::new().name("gallery-bus".into()).spawn(move||{
            let rt=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            rt.block_on(async move {
                let mut commands=["gallery.describe","gallery.state","gallery.execute","gallery.ui.tree","gallery.ui.focus","gallery.ui.hover","gallery.ui.click","gallery.ui.key","gallery.ui.text","gallery.ui.scroll","gallery.ui.drag","gallery.quit"].into_iter().map(|n|bus::CommandDescriptor::new(n,&[],"See gallery.describe for the typed contract",n.ends_with("state")||n.ends_with("describe")||n.ends_with("tree"))).collect::<Vec<_>>();
                for widget in crate::controller::composition(){for action in crate::controller::actions(&widget){commands.push(bus::CommandDescriptor::new(&format!("gallery.widget.{}.{action}",widget.id),&["value"],"Typed widget action; same path as pointer/keyboard",false));}}
                let options=SupervisedClient::connect_options(&name,&url).bounded_incoming_rpc(32).with_commands(commands);
                let client=tokio::select!{_=&mut stopped=>return,c=options.connect()=>match c{Ok(c)=>c,Err(e)=>{eprintln!("Bus connect: {e}");return;}}};
                let mut incoming=client.incoming_bounded().expect("bounded ingress");
                loop {
                    let command=tokio::select!{_=&mut stopped=>break,c=incoming.recv()=>match c{Some(BoundedIncomingEvent::Command(c))=>c,Some(_)=>continue,None=>break}};
                    if command.id.is_none(){continue;}
                    let(reply,response)=oneshot::channel();
                    let request=Request{command:command.command.clone(),body:command.args.clone(),reply};
                    let answer=match tx.try_send(request){
                        Ok(())=>{wake();tokio::select!{_=&mut stopped=>break,r=tokio::time::timeout(std::time::Duration::from_secs(10),response)=>match r{Ok(Ok(answer))=>answer,_=>(20,json!({"error":"gallery owner did not complete request"}))}}},
                        Err(_)=>(10,json!({"error_code":"BUSY"})),
                    };
                    let _=client.respond(&command,answer.0,&answer.1.to_string()).await;
                }
                client.close().await;
            });
        }).unwrap();
        Self {
            requests,
            stop: Some(stop),
            thread: Some(thread),
        }
    }
}
impl Drop for NativeBus {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
