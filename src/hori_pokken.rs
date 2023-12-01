use crate::HIDGamepad;
use gilrs::{
    ev::{state::GamepadState, Code},
    Axis, Button, Gamepad,
};
use scroll::{IOwrite, Pwrite, SizeWith};
use std::{collections::HashMap, fmt::Write};

/// A HID descriptor that is compatible with the HORI Pokken Pad.
///
/// This is known to work with the Switch as a wired controller. It matches
/// the original descriptor precisely except for extending the number of buttons
/// to 16. The original descriptor used two bytes for buttons, but only included
/// 13 actual buttons.
const HORI_POKKEN_PAD_DESCRIPTOR: &[u8] = &[
    0x05, 0x01, // USAGE_PAGE (Generic Desktop)
    0x09, 0x05, // USAGE (Game Pad)
    0xa1, 0x01, // COLLECTION (Application)
    0x15, 0x00, //   LOGICAL_MINIMUM (0)
    0x25, 0x01, //   LOGICAL_MAXIMUM (1)
    0x35, 0x00, //   PHYSICAL_MINIMUM (0)
    0x45, 0x01, //   PHYSICAL_MAXIMUM (1)
    0x75, 0x01, //   REPORT_SIZE (1)
    0x95, 0x0e, //   REPORT_COUNT (14)
    0x05, 0x09, //   USAGE_PAGE (Button)
    0x19, 0x01, //   USAGE_MINIMUM (Button 1)
    0x29, 0x0e, //   USAGE_MAXIMUM (Button 14)
    0x81, 0x02, //   INPUT (Data,Var,Abs)
    0x95, 0x02, //   REPORT_COUNT (2)
    0x81, 0x01, //   INPUT (Const,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x05, 0x01, //   USAGE_PAGE (Generic Desktop)
    0x25, 0x07, //   LOGICAL_MAXIMUM (7)
    0x46, 0x3b, 0x01, //   PHYSICAL_MAXIMUM (315)
    0x75, 0x04, //   REPORT_SIZE (4)
    0x95, 0x01, //   REPORT_COUNT (1)
    0x65, 0x14, //   UNIT (Eng Rot:Angular Pos)
    0x09, 0x39, //   USAGE (Hat switch)
    0x81, 0x42, //   INPUT (Data,Var,Abs,Null)
    0x65, 0x00, //   UNIT (None)
    0x95, 0x01, //   REPORT_COUNT (1)
    0x81, 0x01, //   INPUT (Cnst,Ary,Abs)
    0x26, 0xff, 0x00, //   LOGICAL_MAXIMUM (255)
    0x46, 0xff, 0x00, //   PHYSICAL_MAXIMUM (255)
    0x09, 0x30, //   USAGE (X)
    0x09, 0x31, //   USAGE (Y)
    0x09, 0x32, //   USAGE (Z)
    0x09, 0x35, //   USAGE (Rz)
    0x75, 0x08, //   REPORT_SIZE (8)
    0x95, 0x04, //   REPORT_COUNT (4)
    0x81, 0x02, //   INPUT (Data,Var,Abs)
    0x06, 0x00, 0xff, //   USAGE_PAGE (Vendor Defined)
    0x09, 0x20, //   USAGE
    0x95, 0x01, //   REPORT_COUNT (1)
    0x81, 0x02, //   INPUT (Data,Var,Abs)
    0x0a, 0x21, 0x26, //   USAGE
    0x95, 0x08, //   REPORT_COUNT (8)
    0x91, 0x02, //   OUTPUT (Data,Var,Abs)
    0xc0, // END_COLLECTION
];

/// A HID report for the HORI Pokken Pad that matches the above descriptor.
#[derive(Debug, Default, PartialEq, Pwrite, IOwrite, SizeWith)]
pub struct HoriPokkenPadReport {
    /// 14 button values + 2 unused bits.
    pub buttons: u16,
    /// The d-pad (as a hat switch).
    ///
    /// Only the first nibble is used for the report.
    pub dpad: u8,
    /// Left  Stick X
    pub lx: u8,
    /// Left  Stick Y
    pub ly: u8,
    /// Right Stick X
    pub rx: u8,
    /// Right Stick Y
    pub ry: u8,
    /// Unknown purpose
    pub _vendor_spec: u8,
}

impl std::fmt::Display for HoriPokkenPadReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('[')?;
        for (i, name) in BUTTON_NAMES.into_iter().enumerate() {
            if self.buttons & (1 << (i as u8)) != 0 {
                f.write_str(name)?;
            } else {
                for _ in 0..name.len() {
                    f.write_char(f.fill())?;
                }
            }
            f.write_char(' ')?;
        }
        f.write_char(']')?;

        Ok(())
    }
}

const BUTTON_NAMES: &[&str] = &[
    "Y", "B", "A", "X", "L", "R", "ZL", "ZR", "-", "+", "LS", "RS", "Home",
];

const BUTTON_ORDER: &[Button] = &[
    // Switch button Y
    Button::West,
    // Switch button B
    Button::South,
    // Switch button A
    Button::East,
    // Switch button X
    Button::North,
    // Switch button L
    Button::LeftTrigger,
    // Switch button R
    Button::RightTrigger,
    // Switch button ZL
    Button::LeftTrigger2,
    // Switch button ZR
    Button::RightTrigger2,
    // Switch button SELECT
    Button::Select,
    // Switch button START
    Button::Start,
    // Switch button LS
    Button::LeftThumb,
    // Switch button RS
    Button::RightThumb,
    // Switch button HOME
    Button::Mode,
    // Capture button is unmapped
];

#[derive(Debug)]
pub struct HoriPokkenPad;

fn get_axis(axis: Axis, state: &GamepadState, map: &HashMap<Axis, Code>) -> u8 {
    map.get(&axis)
        .map(|c| {
            let v = (state.value(*c) + 1.0) / 2.0 * 255.0;
            if v.is_normal() {
                v.round() as u8
            } else {
                128
            }
        })
        .unwrap_or(128)
}

fn get_button(button: Button, state: &GamepadState, map: &HashMap<Button, Code>) -> bool {
    map.get(&button)
        .map(|code| state.is_pressed(*code))
        .unwrap_or(false)
}

impl HIDGamepad for HoriPokkenPad {
    const DESCRIPTOR: &'static [u8] = HORI_POKKEN_PAD_DESCRIPTOR;
    const MANUFACTURER: &'static str = "HORI CO.,LTD.";
    const PRODUCT: &'static str = "POKKEN CONTROLLER";
    const VENDOR_ID: &'static [u8; 6] = b"0x0f0d"; // Hori
    const PRODUCT_ID: &'static [u8; 6] = b"0x0092"; // Pokken Controller
    const ANALOG_BUTTONS: bool = false;
    type Report = HoriPokkenPadReport;

    fn fill_report(
        gamepad: &Gamepad,
        button_mapping: &HashMap<Button, Code>,
        axis_mapping: &HashMap<Axis, Code>,
    ) -> Self::Report {
        let state = gamepad.state();
        // Map buttons.
        let mut buttons = 0;
        for (i, b) in BUTTON_ORDER.iter().enumerate() {
            if get_button(*b, state, button_mapping) {
                buttons |= 1 << i;
            }
        }
        // Map axes.
        let lx = get_axis(Axis::LeftStickX, state, axis_mapping);
        let ly = get_axis(Axis::LeftStickY, state, axis_mapping);
        let rx = get_axis(Axis::RightStickX, state, axis_mapping);
        let ry = get_axis(Axis::RightStickY, state, axis_mapping);
        // Map d-pad.
        let up = get_button(Button::DPadUp, state, button_mapping);
        let right = get_button(Button::DPadRight, state, button_mapping);
        let down = get_button(Button::DPadDown, state, button_mapping);
        let left = get_button(Button::DPadLeft, state, button_mapping);
        let dpad = match (up, right, down, left) {
            // Up
            (true, false, false, false) => 0x00,
            // Up-right
            (true, true, false, false) => 0x01,
            // Right
            (false, true, false, false) => 0x02,
            // Down-right
            (false, true, true, false) => 0x03,
            // Down
            (false, false, true, false) => 0x04,
            // Down-left
            (false, false, true, true) => 0x05,
            // Left
            (false, false, false, true) => 0x06,
            // Up-left
            (true, false, false, true) => 0x07,
            // Default, no direction pressed.
            _ => 0x08,
        };
        HoriPokkenPadReport {
            buttons,
            dpad,
            lx,
            ly,
            rx,
            ry,
            _vendor_spec: 0,
        }
    }
}
