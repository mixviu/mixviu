use editor_model::fake::FakeEditd;
use serde_json::{Value,json};
use viu::documents::Documents;

pub fn settle(docs:&Documents,key:&str,server:&mut FakeEditd) {
    for _ in 0..128 {
        let effects=docs.call("effects",key,json!({})).unwrap();
        for request in effects["out"].as_array().unwrap() {
            let body:Value=serde_json::from_str(request["body"].as_str().unwrap()).unwrap();
            let reply=server.handle("ced",request["command"].as_str().unwrap(),&body);
            docs.call("reply",key,json!({"op_id":request["op_id"],"rc":reply.rc,"body":reply.body})).unwrap();
            for event in server.take_events() { docs.call("event",key,serde_json::to_value(event).unwrap()).unwrap(); }
        }
        if docs.call("state",key,json!({})).unwrap()["idle"]==true{return;}
    }
    panic!("mirror did not drain");
}
pub fn seed(docs:&Documents,key:&str,path:&str,text:&str)->FakeEditd {
    let mut server=FakeEditd::new("12345678");server.create(Some(path),text);
    let open=server.handle("ced","edit.open",&json!({"path":path}));assert_eq!(open.rc,0);
    docs.call("open",key,open.body).unwrap();settle(docs,key,&mut server);server
}

