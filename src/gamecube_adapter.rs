/*
Adapted from https://github.com/arpruss/switchgamecubeusbadapter
See also: https://gbatemp.net/threads/gamecube-controller-adapter-usb-hid-data-dump.606682/
 */
const DESCRIPTOR: &[u8] = &[
    0x05, 0x05, // Usage Page (Game Ctrls)
    0x09, 0x00, // Usage (Undefined)
    0xA1, 0x01, // Collection (Application)
    0x85, 0x11, //   Report ID (17) RUMBLE
    0x19, 0x00, //   Usage Minimum (Undefined)
    0x2A, 0xFF, 0x00, //   Usage Maximum (0xFF)
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x05, //   Report Count (5)
    0x91,
    0x00, //   Output (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0xC0, // End Collection
    0xA1, 0x01, // Collection (Application)
    0x85, 0x21, //   Report ID (33) MAIN
    0x19, 0x00, //   Usage Minimum (Undefined)
    0x2A, 0xFF, 0x00, //   Usage Maximum (0xFF)
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x25, //   Report Count (37)
    0x81, 0x00, //   Input (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xC0, // End Collection
    0xA1, 0x01, // Collection (Application)
    0x85, 0x12, //   Report ID (18)
    0x19, 0x00, //   Usage Minimum (Undefined)
    0x2A, 0xFF, 0x00, //   Usage Maximum (0xFF)
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x01, //   Report Count (1)
    0x91,
    0x00, //   Output (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0xC0, // End Collection
    0xA1, 0x01, // Collection (Application)
    0x85, 0x22, //   Report ID (34)
    0x19, 0x00, //   Usage Minimum (Undefined)
    0x2A, 0xFF, 0x00, //   Usage Maximum (0xFF)
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x19, //   Report Count (25)
    0x81, 0x00, //   Input (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xC0, // End Collection
    0xA1, 0x01, // Collection (Application)
    0x85, 0x13, //   Report ID (19)
    0x19, 0x00, //   Usage Minimum (Undefined)
    0x2A, 0xFF, 0x00, //   Usage Maximum (0xFF)
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x01, //   Report Count (1)
    0x91,
    0x00, //   Output (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0xC0, // End Collection
    0xA1, 0x01, // Collection (Application)
    0x85, 0x23, //   Report ID (35)
    0x19, 0x00, //   Usage Minimum (Undefined)
    0x2A, 0xFF, 0x00, //   Usage Maximum (0xFF)
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x02, //   Report Count (2)
    0x81, 0x00, //   Input (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xC0, // End Collection
    0xA1, 0x01, // Collection (Application)
    0x85, 0x14, //   Report ID (20)
    0x19, 0x00, //   Usage Minimum (Undefined)
    0x2A, 0xFF, 0x00, //   Usage Maximum (0xFF)
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x01, //   Report Count (1)
    0x91,
    0x00, //   Output (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0xC0, // End Collection
    0xA1, 0x01, // Collection (Application)
    0x85, 0x24, //   Report ID (36)
    0x19, 0x00, //   Usage Minimum (Undefined)
    0x2A, 0xFF, 0x00, //   Usage Maximum (0xFF)
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x02, //   Report Count (2)
    0x81, 0x00, //   Input (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xC0, // End Collection
    0xA1, 0x01, // Collection (Application)
    0x85, 0x15, //   Report ID (21)
    0x19, 0x00, //   Usage Minimum (Undefined)
    0x2A, 0xFF, 0x00, //   Usage Maximum (0xFF)
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x01, //   Report Count (1)
    0x91,
    0x00, //   Output (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0xC0, // End Collection
    0xA1, 0x01, // Collection (Application)
    0x85, 0x25, //   Report ID (37)
    0x19, 0x00, //   Usage Minimum (Undefined)
    0x2A, 0xFF, 0x00, //   Usage Maximum (0xFF)
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x02, //   Report Count (2)
    0x81, 0x00, //   Input (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xC0, // End Collection
];
