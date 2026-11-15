// SPDX-License-Identifier: MIT OR Apache-2.0
//! Devices admitted by libinput, not nodes merely present in /dev/input.
use comp_model::snapshot::InputDeviceSnapshot;
use smithay::{
    backend::{input::InputEvent, libinput::LibinputInputBackend},
    reexports::input::{Device, DeviceCapability},
};
use std::{cell::RefCell, collections::BTreeMap};

#[derive(Default)]
pub(super) struct Inventory {
    devices: BTreeMap<String, InputDeviceSnapshot>,
    events: u64,
}

impl Inventory {
    pub(super) fn observe(&mut self, event: &InputEvent<LibinputInputBackend>) {
        match event {
            InputEvent::DeviceAdded { device } => {
                let row = describe(device);
                if self.devices.get(device.sysname()) != Some(&row) {
                    info!(
                        "input opened: node={} name={:?} capabilities={:?}",
                        row.node, row.name, row.capabilities
                    );
                }
                self.devices.insert(device.sysname().to_owned(), row);
            }
            InputEvent::DeviceRemoved { device } => {
                if let Some(row) = self.devices.remove(device.sysname()) {
                    info!(
                        "input closed: node={} name={:?} capabilities={:?}",
                        row.node, row.name, row.capabilities
                    );
                }
            }
            _ => self.events = self.events.saturating_add(1),
        }
    }

    pub(super) fn snapshot(&self) -> (Vec<InputDeviceSnapshot>, u64) {
        (self.devices.values().cloned().collect(), self.events)
    }
}

fn describe(device: &Device) -> InputDeviceSnapshot {
    let capabilities = [
        (DeviceCapability::Keyboard, "keyboard"),
        (DeviceCapability::Pointer, "pointer"),
        (DeviceCapability::Touch, "touch"),
        (DeviceCapability::TabletTool, "tablet_tool"),
        (DeviceCapability::TabletPad, "tablet_pad"),
        (DeviceCapability::Gesture, "gesture"),
        (DeviceCapability::Switch, "switch"),
    ]
    .into_iter()
    .filter(|(cap, _)| device.has_capability(*cap))
    .map(|(_, name)| name.to_owned())
    .collect();
    InputDeviceSnapshot {
        name: device.name().into_owned(),
        node: format!("/dev/input/{}", device.sysname()),
        capabilities,
    }
}

thread_local! {
    static DEVICES: RefCell<Inventory> = RefCell::new(Inventory::default());
}

pub(super) fn observe(event: &InputEvent<LibinputInputBackend>) {
    DEVICES.with_borrow_mut(|devices| devices.observe(event));
}

pub fn snapshot() -> (Vec<InputDeviceSnapshot>, u64) {
    DEVICES.with_borrow(Inventory::snapshot)
}

#[cfg(test)]
mod uinput_tests;
