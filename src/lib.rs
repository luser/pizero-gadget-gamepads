use gilrs::{ev::Code, Axis, Button, Gamepad};
use scroll::{
    ctx::{SizeWith, TryIntoCtx},
    Endian, LE,
};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

pub mod gamecube_adapter;
pub mod hid_gadget;
pub mod hori_pokken;

pub trait HIDGamepad: Debug {
    /// The HID descriptor for this gamepad
    const DESCRIPTOR: &'static [u8];
    /// USB device manufacturer name
    const MANUFACTURER: &'static str;
    /// USB device product name
    const PRODUCT: &'static str;
    /// The USB Vendor ID, in hex with a 0x prefix
    const VENDOR_ID: &'static [u8; 6];
    /// The USB Product ID, in hex with a 0x prefix
    const PRODUCT_ID: &'static [u8; 6];
    /// Does this gamepad have analog buttons (including triggers)
    const ANALOG_BUTTONS: bool;
    /// The format of the HID report to send
    type Report: SizeWith<Endian> + TryIntoCtx<Endian, Error = scroll::Error> + Display;

    fn report_size() -> usize {
        <Self::Report as SizeWith<Endian>>::size_with(&LE)
    }
    fn fill_report(
        gamepad: &Gamepad,
        button_mapping: &HashMap<Button, Code>,
        axis_mapping: &HashMap<Axis, Code>,
    ) -> Self::Report;
}
