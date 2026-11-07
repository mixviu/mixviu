// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Gold's ordered pointer gestures, adapted to a reusable compound surface.
//! Application focus/clipboard effects return to Mix as widget intents.
use super::{Terminal, input, model::{MouseModifiers, SelectionSide, SelectionType}, sessions::Sessions};
use egui::{Event, PointerButton, Pos2, Rect};
use std::{sync::{Arc,Mutex}, time::{Duration,Instant}};

#[derive(Clone)]
pub(super) struct Placement {
    pub handle: String,
    pub id: egui::Id,
    pub area: Rect,
    pub grid: Rect,
    pub cell: (f32,f32),
    pub dimensions: (u16,u16),
}
#[derive(Clone,Debug,PartialEq,Eq)]
struct Hit {handle:String,col:u16,row:u16,side:SelectionSide}
#[derive(Clone)]
struct Press {terminal:Arc<Mutex<Terminal>>,last:Hit,button:u8,local:bool,origin:Pos2,mods:MouseModifiers}
pub(super) enum Effect {
    Focus(String),
    Read {handle:String,primary:bool},
    Write {text:String,primary:bool},
    Wheel {handle:String,col:u16,row:u16,unit:egui::MouseWheelUnit,delta:egui::Vec2,modifiers:egui::Modifiers},
}
#[derive(Default)]
pub(super) struct Mouse {
    pressed:Option<Press>,clicks:Option<(Hit,Instant,u8)>,hover:Option<Hit>,
    position:Option<Pos2>,modifiers:egui::Modifiers,
}
fn code(button:PointerButton)->Option<u8> {
    match button {PointerButton::Primary=>Some(0),PointerButton::Middle=>Some(1),PointerButton::Secondary=>Some(2),_=>None}
}
fn mods(value:egui::Modifiers)->MouseModifiers {MouseModifiers {shift:value.shift,alt:value.alt,ctrl:value.ctrl}}
fn hit(placements:&[Placement],position:Pos2,target:Option<&str>)->Option<Hit> {
    let p=placements.iter().find(|p|target.map_or_else(||position.x>=p.area.min.x && position.x<p.area.max.x && position.y>=p.area.min.y && position.y<p.area.max.y,|target|p.handle==target))?;
    let relative=position-p.grid.min;
    let (col,row)=input::pointer_cell(egui::pos2(relative.x,relative.y),0.0,p.cell,p.dimensions);
    let side=if relative.x-f32::from(col)*p.cell.0<p.cell.0/2.0 {SelectionSide::Left} else {SelectionSide::Right};
    Some(Hit {handle:p.handle.clone(),col,row,side})
}
impl Mouse {
    pub(super) fn position(&self)->Option<Pos2> {self.position}
    pub(super) fn grabbed(&self)->bool {self.pressed.is_some()}
    pub(super) fn cancel(&mut self) {
        self.clicks=None;
        if let Some(press)=self.pressed.take() {
            let terminal=press.terminal.lock().unwrap();
            if !press.local {terminal.mouse_button(press.last.col,press.last.row,press.button,false,press.mods);}
            else if press.button==0 {terminal.selection_clear();}
        }
    }
    fn click(&mut self,hit:Hit,now:Instant)->SelectionType {
        let count=self.clicks.as_ref().map_or(1,|(last,when,count)| {
            if last.handle==hit.handle && last.col==hit.col && last.row==hit.row && now.saturating_duration_since(*when)<=Duration::from_millis(300) {count%3+1} else {1}
        });
        self.clicks=Some((hit,now,count));
        match count {2=>SelectionType::Semantic,3=>SelectionType::Lines,_=>SelectionType::Simple}
    }
    pub(super) fn update(&mut self,placements:&[Placement],sessions:&Sessions,events:&[Event],focused:bool,copy_on_select:bool,now:Instant)->Vec<Effect> {
        let mut out=Vec::new();
        if !focused || self.pressed.as_ref().is_some_and(|p|!placements.iter().any(|v|v.handle==p.last.handle) || sessions.get(&p.last.handle).is_err()) {self.cancel();}
        if !focused {return out;}
        for event in events {
            match event {
                Event::ModifiersChanged(value)=>self.modifiers=*value,
                Event::PointerMoved(position)=> {
                    self.position=Some(*position);
                    let current=hit(placements,*position,None);
                    let changed=self.hover!=current;self.hover=current;
                    if (changed || (self.hover.is_none() && self.pressed.is_some())) && (self.hover.is_some() || self.pressed.is_some()) {self.moved(placements,sessions,*position,false,copy_on_select,&mut out);}
                }
                Event::PointerButton {pos,button,pressed,modifiers}=> {
                    self.modifiers=*modifiers;self.position=Some(*pos);
                    if *pressed {
                        if self.pressed.is_some() {self.cancel();}
                        self.hover=hit(placements,*pos,None);
                        let Some(button)=code(*button) else {continue};
                        let Some(hit)=self.hover.clone() else {continue};
                        let Ok(terminal)=sessions.get(&hit.handle) else {continue};
                        sessions.activity();out.push(Effect::Focus(hit.handle.clone()));
                        let local=modifiers.shift || !terminal.lock().unwrap().mouse_reporting();
                        if local && button==0 {sessions.clear_other_selections(&hit.handle);}
                        self.pressed=Some(Press {terminal:terminal.clone(),last:hit.clone(),button,local,origin:*pos,mods:mods(*modifiers)});
                        if !local {self.clicks=None;terminal.lock().unwrap().mouse_button(hit.col,hit.row,button,true,mods(*modifiers));}
                        else if button==0 {
                            let kind=self.click(hit.clone(),now);
                            terminal.lock().unwrap().selection_start(hit.col,hit.row,hit.side,kind);
                        } else if button==1 {self.clicks=None;out.push(Effect::Read {handle:hit.handle,primary:true});}
                    } else if code(*button).is_some_and(|button|self.pressed.as_ref().is_some_and(|p|p.button==button)) {
                        self.moved(placements,sessions,*pos,true,copy_on_select,&mut out);
                    }
                }
                Event::MouseWheel {unit,delta,modifiers}=> {
                    if let Some(hit)=self.position.and_then(|pos|hit(placements,pos,None)) {
                        sessions.activity();out.push(Effect::Wheel {handle:hit.handle,col:hit.col,row:hit.row,unit:*unit,delta:*delta,modifiers:*modifiers});
                    }
                }
                Event::PointerGone=>{self.hover=None;self.position=None;},
                _=>{}
            }
        }
        out
    }
    fn moved(&mut self,placements:&[Placement],sessions:&Sessions,position:Pos2,released:bool,copy_on_select:bool,out:&mut Vec<Effect>) {
        let press=self.pressed.clone();
        let Some(hit)=hit(placements,position,press.as_ref().map(|p|p.last.handle.as_str())) else {self.cancel();return};
        let Ok(terminal)=sessions.get(&hit.handle) else {self.cancel();return};
        if released {self.pressed=None;} else if let Some(press)=self.pressed.as_mut() {press.last=hit.clone();}
        let terminal=terminal.lock().unwrap();
        match press {
            Some(press) if press.local && press.button==0=> {
                terminal.selection_update(hit.col,hit.row,hit.side);
                if (position.x-press.origin.x).abs()>5.0 || (position.y-press.origin.y).abs()>5.0 {self.clicks=None;}
                if released && let Some(text)=terminal.selection_finish() {
                    if copy_on_select {out.push(Effect::Write {text:text.clone(),primary:false});}
                    out.push(Effect::Write {text,primary:true});
                }
            }
            Some(press) if !press.local=> {
                if released {terminal.mouse_button(hit.col,hit.row,press.button,false,press.mods);}
                else {terminal.mouse_motion(hit.col,hit.row,press.button,mods(self.modifiers));}
            }
            None if !released=>{terminal.mouse_motion(hit.col,hit.row,3,mods(self.modifiers));}
            _=>{}
        }
    }
}
impl Drop for Mouse {fn drop(&mut self) {self.cancel();}}

#[cfg(test)]
mod tests {
    use super::*;
    use rio_vt::{corcovado::channel,event::Msg};
    fn setup(reporting:bool)->(Sessions,Vec<Placement>,channel::Receiver<Msg>) {
        let sessions=Sessions::fixture();
        sessions.fixture_pane("vt-1",if reporting {b"\x1b[?1003h\x1b[?1006h"} else {b"abcdef"});
        sessions.fixture_pane("vt-2",b"other");
        let receiver=sessions.get("vt-1").unwrap().lock().unwrap().listener.test_input_receiver();
        let places=(0..2).map(|i| {
            let area=Rect::from_min_size(egui::pos2(i as f32*800.0,0.0),egui::vec2(800.0,240.0));
            Placement {handle:format!("vt-{}",i+1),id:egui::Id::new(i),area,grid:area,cell:(10.0,10.0),dimensions:(80,24)}
        }).collect();
        (sessions,places,receiver)
    }
    fn button(x:f32,y:f32,button:PointerButton,pressed:bool,shift:bool)->Event {
        Event::PointerButton {pos:egui::pos2(x,y),button,pressed,modifiers:egui::Modifiers {shift,..Default::default()}}
    }
    fn bytes(receiver:&channel::Receiver<Msg>)->Vec<u8> {
        let mut bytes=Vec::new();while let Ok(message)=receiver.try_recv() {if let Msg::Input(value)=message {bytes.extend_from_slice(&value);}}bytes
    }
    #[test]
    fn reporting_grab_keeps_owner_and_press_modifiers_until_matching_release() {
        let (sessions,places,receiver)=setup(true);let mut mouse=Mouse::default();
        mouse.update(&places,&sessions,&[
            button(12.0,12.0,PointerButton::Primary,true,false),
            button(12.0,12.0,PointerButton::Secondary,false,false),
            button(900.0,400.0,PointerButton::Primary,false,true),
        ],true,false,Instant::now());
        assert_eq!(bytes(&receiver),b"\x1b[<0;2;2M\x1b[<0;80;24m");
        assert!(sessions.call("selection","vt-2",serde_json::json!({})).unwrap().is_null());
    }
    #[test]
    fn all_three_buttons_and_hover_motion_reach_the_program_at_event_coordinates() {
        let (sessions,places,receiver)=setup(true);let mut mouse=Mouse::default();
        for (button,code) in [(PointerButton::Primary,0),(PointerButton::Middle,1),(PointerButton::Secondary,2)] {
            mouse.update(&places,&sessions,&[button_event(button,true),button_event(button,false)],true,false,Instant::now());
            assert_eq!(bytes(&receiver),format!("\x1b[<{code};2;2M\x1b[<{code};2;2m").as_bytes());
        }
        mouse.update(&places,&sessions,&[Event::PointerMoved(egui::pos2(32.0,22.0)),Event::PointerMoved(egui::pos2(33.0,23.0))],true,false,Instant::now());
        assert_eq!(bytes(&receiver),b"\x1b[<35;4;3M");
    }
    fn button_event(which:PointerButton,pressed:bool)->Event {button(12.0,12.0,which,pressed,false)}
    #[test]
    fn hidden_and_unfocused_grabs_cancel_once_and_never_retarget_a_sibling() {
        let (sessions,places,receiver)=setup(true);let mut mouse=Mouse::default();
        mouse.update(&places,&sessions,&[button_event(PointerButton::Secondary,true)],true,false,Instant::now());
        assert_eq!(bytes(&receiver),b"\x1b[<2;2;2M");
        mouse.update(&places[1..],&sessions,&[],true,false,Instant::now());
        mouse.update(&places[1..],&sessions,&[],false,false,Instant::now());
        assert_eq!(bytes(&receiver),b"\x1b[<2;2;2m");
        assert!(mouse.pressed.is_none());
    }
    #[test]
    fn local_selection_uses_half_cells_and_copies_primary_and_optional_clipboard() {
        for copy in [false,true] {
            let (sessions,places,_)=setup(false);let mut mouse=Mouse::default();
            let other=sessions.get("vt-2").unwrap();
            other.lock().unwrap().selection_start(0,0,SelectionSide::Left,SelectionType::Simple);
            other.lock().unwrap().selection_update(3,0,SelectionSide::Right);
            assert!(other.lock().unwrap().selection_text().is_some());
            let effects=mouse.update(&places,&sessions,&[
                button(2.0,2.0,PointerButton::Primary,true,false),
                Event::PointerMoved(egui::pos2(38.0,2.0)),
                button(38.0,2.0,PointerButton::Primary,false,false),
            ],true,copy,Instant::now());
            let writes=effects.iter().filter_map(|effect|match effect {Effect::Write {text,primary}=>Some((text.as_str(),*primary)),_=>None}).collect::<Vec<_>>();
            assert_eq!(writes,if copy {vec![("abcd",false),("abcd",true)]} else {vec![("abcd",true)]});
            assert!(other.lock().unwrap().selection_text().is_none());
        }
    }
    #[test]
    fn shift_latches_local_ownership_and_middle_read_names_the_original_pane() {
        let (sessions,places,receiver)=setup(true);let mut mouse=Mouse::default();
        let effects=mouse.update(&places,&sessions,&[button(12.0,12.0,PointerButton::Middle,true,true),button(12.0,12.0,PointerButton::Middle,false,false)],true,false,Instant::now());
        assert!(effects.iter().any(|e|matches!(e,Effect::Read {handle,primary:true} if handle=="vt-1")));
        assert!(bytes(&receiver).is_empty());
    }
    #[test]
    fn multi_clicks_use_gold_same_cell_window_and_cycle_without_drag_or_time_leakage() {
        let mut mouse=Mouse::default();let now=Instant::now();
        let hit=Hit {handle:"vt-1".into(),col:3,row:2,side:SelectionSide::Left};
        assert_eq!(mouse.click(hit.clone(),now),SelectionType::Simple);
        assert_eq!(mouse.click(hit.clone(),now+Duration::from_millis(100)),SelectionType::Semantic);
        assert_eq!(mouse.click(hit.clone(),now+Duration::from_millis(200)),SelectionType::Lines);
        assert_eq!(mouse.click(hit.clone(),now+Duration::from_millis(300)),SelectionType::Simple);
        assert_eq!(mouse.click(hit.clone(),now+Duration::from_millis(601)),SelectionType::Simple);
        assert_eq!(mouse.click(Hit {handle:"vt-2".into(),..hit},now+Duration::from_millis(602)),SelectionType::Simple);
        let (sessions,places,_)=setup(false);let mut mouse=Mouse::default();
        mouse.update(&places,&sessions,&[button(12.0,2.0,PointerButton::Primary,true,false),button(12.0,2.0,PointerButton::Primary,false,false)],true,false,now);
        let second=mouse.update(&places,&sessions,&[button(12.0,2.0,PointerButton::Primary,true,false),button(12.0,2.0,PointerButton::Primary,false,false)],true,false,now+Duration::from_millis(100));
        assert!(second.iter().any(|effect|matches!(effect,Effect::Write {text,primary:true} if text=="abcdef")));
    }
    #[test]
    fn wheel_owner_follows_ordered_positions_instead_of_the_coalesced_hover() {
        let (sessions,places,_)=setup(false);let mut mouse=Mouse::default();
        let wheel=||Event::MouseWheel {unit:egui::MouseWheelUnit::Line,delta:egui::vec2(0.0,1.0),modifiers:Default::default()};
        let effects=mouse.update(&places,&sessions,&[Event::PointerMoved(egui::pos2(12.0,12.0)),wheel(),Event::PointerMoved(egui::pos2(800.0,12.0)),wheel()],true,false,Instant::now());
        let handles=effects.iter().filter_map(|effect|match effect {Effect::Wheel {handle,..}=>Some(handle.as_str()),_=>None}).collect::<Vec<_>>();
        assert_eq!(handles,["vt-1","vt-2"]);
    }
}
