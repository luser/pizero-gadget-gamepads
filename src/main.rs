use anyhow::{bail, Context, Result};
use env_logger::Builder;
use log::{info, LevelFilter};
use pizero_gadget_gamepads::hori_pokken::*;
use std::fs;
use std::os::unix::fs::symlink;
use std::os::unix::prelude::OsStrExt;
use std::path::{Path, PathBuf};

const CONFIGFS_GADGET_PATH: &str = "/sys/kernel/config/usb_gadget/";
const GADGET_NAME: &str = "gadget_gamepads";

#[derive(Debug)]
struct HIDGadgetDevice {
    function_name: String,
    hidg_path: String,
}

#[derive(Debug)]
struct HIDGadget {
    path: PathBuf,
    devices: Vec<HIDGadgetDevice>,
}

impl Drop for HIDGadget {
    fn drop(&mut self) {
        // Empty out the UDC file to disconnect the device
        fs::write(&self.path.join("UDC"), b"").unwrap();
        // The ordering of these is important:
        for device in self.devices.iter() {
            // First, unlink the function from the config
            fs::remove_file(&self.path.join("configs/c.1").join(&device.function_name)).unwrap();
            // Now remove the function definition
            fs::remove_dir(&self.path.join("functions").join(&device.function_name)).unwrap();
        }
        for item in &[
            // Remove config strings
            "configs/c.1/strings/0x409",
            // Remove config
            "configs/c.1",
            // Remove gadget strings
            "strings/0x409",
        ] {
            fs::remove_dir(&self.path.join(item)).unwrap()
        }
        // Finally, remove the entire gadget dir
        fs::remove_dir(&self.path).unwrap()
    }
}

fn write_file(path: &Path, contents: impl AsRef<[u8]>) -> Result<()> {
    fs::write(path, contents).with_context(|| format!("Failed to write to: {path:?}"))?;
    Ok(())
}

fn create_gadget_function(
    gadget_path: &Path,
    config_path: &Path,
    name: &str,
    descriptor: &[u8],
    report_size: usize,
) -> Result<HIDGadgetDevice> {
    let path = gadget_path.join("functions").join(name);
    fs::create_dir_all(&path)?;
    info!("creating gadget function: {path:?}");
    // Populate function attributes
    for (attr, contents) in &[
        ("protocol", b"1".as_slice()),
        ("subclass", b"1"),
        ("report_length", report_size.to_string().as_bytes()),
        ("report_desc", descriptor),
    ] {
        write_file(&path.join(attr), contents)?;
    }
    // Link the function to the config
    let target = config_path.join(name);
    info!("Adding symlink from function to {target:?}");
    symlink(&path, &target)?;
    // Read the device number out
    let dev_num = fs::read_to_string(&path.join("dev"))?;
    let hidg_path = if let Some((major, minor)) = dev_num.trim_end().split_once(':') {
        if major == "239" {
            format!("/dev/hidg{minor}")
        } else {
            bail!("Unsupported major device: {major}");
        }
    } else {
        bail!("Bad device number?");
    };
    Ok(HIDGadgetDevice {
        function_name: name.to_owned(),
        hidg_path,
    })
}

impl HIDGadget {
    fn create(descriptor: &[u8], report_size: usize, count: usize) -> Result<Self> {
        let path = PathBuf::from(CONFIGFS_GADGET_PATH).join(GADGET_NAME);
        fs::create_dir(&path)?;
        info!("creating gadget at path: {path:?}");
        // Populate top-level attributes
        for (attr, contents) in &[
            ("idVendor", b"0x0f0d"),  // Hori
            ("idProduct", b"0x0092"), // Pokken Controller
            ("bcdDevice", b"0x0100"), // v1.0.0
            ("bcdUSB", b"0x0200"),    // USB 2.0
        ] {
            write_file(&path.join(attr), contents)?;
        }
        // Populate en-US strings
        let strings = path.join("strings/0x409");
        fs::create_dir_all(&strings)?;
        for (attr, contents) in &[
            ("serialnumber", b"0".as_slice()),
            // Unsure if these need to be set exactly?
            ("manufacturer", b"HORI CO.,LTD."),
            ("product", b"POKKEN CONTROLLER"),
        ] {
            write_file(&strings.join(attr), contents)?;
        }
        // Create config and populate a few attributes
        let config = path.join("configs/c.1");
        fs::create_dir_all(&config)?;
        write_file(&config.join("MaxPower"), b"250")?;
        let config_strings = config.join("strings/0x409");
        fs::create_dir_all(&config_strings)?;
        write_file(
            &config_strings.join("configuration"),
            b"USB Gadget Gamepads",
        )?;
        // Create functions and link them to the config
        let mut devices = vec![];
        for i in 0..count {
            let name = format!("hid.usb{i}");
            let device = create_gadget_function(&path, &config, &name, descriptor, report_size)?;
            devices.push(device);
        }

        // Enable the gadget
        info!("Attempting to enable gadget");
        let udc_entries = fs::read_dir("/sys/class/udc")?.collect::<Result<Vec<_>, _>>()?;
        let udc = if let Some(entry) = udc_entries.first() {
            entry.file_name()
        } else {
            bail!("No UDC found!")
        };
        info!("udc: {udc:?}");
        write_file(&path.join("UDC"), udc.as_bytes())?;

        Ok(HIDGadget { path, devices })
    }
}

fn main() -> Result<()> {
    Builder::new()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();
    let gadget = HIDGadget::create(
        HORI_POKKEN_PAD_DESCRIPTOR,
        std::mem::size_of::<HoriPokkenPadReport>(),
        4,
    )?;
    info!("Created gadget: {gadget:?}");
    std::thread::sleep(std::time::Duration::from_secs(10));
    info!("Shutting down");
    Ok(())
}
