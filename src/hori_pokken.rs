/// A HID descriptor that is compatible with the HORI Pokken Pad.
///
/// This is known to work with the Switch as a wired controller. It matches
/// the original descriptor precisely except for extending the number of buttons
/// to 16. The original descriptor used two bytes for buttons, but only included
/// 13 actual buttons.
pub const HORI_POKKEN_PAD_DESCRIPTOR: &[u8] = &[
    0x05, 0x01, // USAGE_PAGE (Generic Desktop)
    0x09, 0x05, // USAGE (Game Pad)
    0xa1, 0x01, // COLLECTION (Application)
    0x15, 0x00, //   LOGICAL_MINIMUM (0)
    0x25, 0x01, //   LOGICAL_MAXIMUM (1)
    0x35, 0x00, //   PHYSICAL_MINIMUM (0)
    0x45, 0x01, //   PHYSICAL_MAXIMUM (1)
    0x75, 0x01, //   REPORT_SIZE (1)
    0x95, 0x10, //   REPORT_COUNT (16)
    0x05, 0x09, //   USAGE_PAGE (Button)
    0x19, 0x01, //   USAGE_MINIMUM (Button 1)
    0x29, 0x10, //   USAGE_MAXIMUM (Button 16)
    0x81, 0x02, //   INPUT (Data,Var,Abs)
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
pub struct HoriPokkenPadReport {
    /// 16 button values
    pub buttons: u16,
    /// The d-pad (as a hat switch)
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

/*TODO: figure this out, see if it's useful.

use usbd_hid::descriptor::generator_prelude::*;

#[gen_hid_descriptor((collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = MOUSE) = {(collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = MOUSE) = {(collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = JOYSTICK) = {
    (usage_page = BUTTON, usage_min = BUTTON_1, usage_max = BUTTON_13) = {
        #[packed_bits 16] buttons=input;
    };
    (usage_page = GENERIC_DESKTOP,) = {

    };
})]
struct HoriPokkenPadDescriptor {
    buttons: u16,
    dpad: u8,
}
*/
