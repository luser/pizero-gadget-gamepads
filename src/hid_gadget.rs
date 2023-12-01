use anyhow::{anyhow, bail, Context, Result};
use log::{debug, error, info};
use scroll::{Pwrite, LE};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::marker::PhantomData;
use std::os::unix::fs::symlink;
use std::os::unix::prelude::OsStrExt;
use std::path::{Path, PathBuf};

use crate::HIDGamepad;

const CONFIGFS_GADGET_PATH: &str = "/sys/kernel/config/usb_gadget/";
const GADGET_NAME: &str = "gadget_gamepads";

#[derive(Debug)]
struct HIDGadgetDevice<G: HIDGamepad> {
    function_name: String,
    hidg_path: String,
    _gamepad: PhantomData<G>,
}

pub struct HIDGadgetDeviceFile<G: HIDGamepad> {
    hidg: File,
    index: usize,
    buf: Vec<u8>,
    dev: HIDGadgetDevice<G>,
}

impl<G: HIDGamepad> HIDGadgetDeviceFile<G> {
    pub fn path(&self) -> &str {
        &self.dev.hidg_path
    }

    /// Write the input report to the device.
    pub fn write_report(&mut self, report: G::Report) -> Result<()> {
        self.buf
            .as_mut_slice()
            .pwrite_with(report, 0, LE)
            .map_err(|_| anyhow!("Error writing report"))?;
        debug!("write_report: {:?}", self.buf.as_slice());
        let written = self.hidg.write(&self.buf)?;
        if written != G::report_size() {
            bail!("Didn't write full report");
        }
        Ok(())
    }

    /// Check if an output report is available from the device, and return it if so.
    /// TODO: actually implement this, probably using nix::poll::poll
    pub fn check_read_report(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct HIDGadget<G: HIDGamepad> {
    pub path: PathBuf,
    devices: Vec<Option<HIDGadgetDevice<G>>>,
}

impl<G: HIDGamepad> Drop for HIDGadget<G> {
    fn drop(&mut self) {
        // Empty out the UDC file to disconnect the device
        fs::write(&self.path.join("UDC"), b"").unwrap();
        // The ordering of these is important:
        for (i, device) in self.devices.iter_mut().enumerate() {
            if let Some(device) = device.take() {
                // First, unlink the function from the config
                fs::remove_file(&self.path.join("configs/c.1").join(&device.function_name))
                    .unwrap();
                // Now remove the function definition
                fs::remove_dir(&self.path.join("functions").join(&device.function_name)).unwrap();
            } else {
                error!("Gadget device {i} still in use while cleaning up!");
            }
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

fn create_gadget_function<G: HIDGamepad>(
    gadget_path: &Path,
    config_path: &Path,
    name: &str,
) -> Result<HIDGadgetDevice<G>> {
    let path = gadget_path.join("functions").join(name);
    fs::create_dir_all(&path)?;
    info!("creating gadget function: {path:?}");
    debug!(
        "report_length: {}, report_desc len: {}",
        G::report_size(),
        G::DESCRIPTOR.len()
    );
    // Populate function attributes
    for (attr, contents) in &[
        ("protocol", b"1".as_slice()),
        ("subclass", b"1"),
        ("report_length", G::report_size().to_string().as_bytes()),
        ("report_desc", G::DESCRIPTOR),
    ] {
        write_file(&path.join(attr), contents)?;
    }
    // Link the function to the config
    let target = config_path.join(name);
    info!("Adding symlink from function to {target:?}");
    symlink(&path, &target)?;
    // Read the device number of the hidg device so we can open it
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
        _gamepad: Default::default(),
    })
}

impl<G: HIDGamepad> HIDGadget<G> {
    pub fn create(count: usize) -> Result<Self> {
        let path = PathBuf::from(CONFIGFS_GADGET_PATH).join(GADGET_NAME);
        fs::create_dir(&path)?;
        info!("creating gadget at path: {path:?}");
        // Populate top-level attributes
        for (attr, contents) in &[
            ("idVendor", G::VENDOR_ID),
            ("idProduct", G::PRODUCT_ID),
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
            ("manufacturer", G::MANUFACTURER.as_bytes()),
            ("product", G::PRODUCT.as_bytes()),
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
            let device = create_gadget_function(&path, &config, &name)?;
            devices.push(Some(device));
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

    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// Attempt to open and take the first available gadget device for use.
    pub fn take_device(&mut self) -> Result<HIDGadgetDeviceFile<G>> {
        for (index, device) in self.devices.iter_mut().enumerate() {
            let res = device.take().map(|dev| {
                // Open the hidg device read+write
                match OpenOptions::new()
                    .create(false)
                    .read(true)
                    .write(true)
                    .open(&dev.hidg_path)
                {
                    Ok(file) => (dev, Ok(file)),
                    Err(e) => {
                        let msg = format!("Failed to open hidg device '{}': {}", &dev.hidg_path, e);
                        (dev, Err(anyhow!(msg)))
                    }
                }
            });
            match res {
                // This device is already taken.
                None => {}
                // There was an available device but we failed to open it, so put it back.
                Some((dev, Err(e))) => {
                    error!("{}", e);
                    let _ = device.insert(dev);
                    // Don't return here, maybe we can open another available device.
                }
                // There was an available device here and we opened it, so return it.
                Some((dev, Ok(hidg))) => {
                    let buf = vec![0; G::report_size()];
                    return Ok(HIDGadgetDeviceFile {
                        hidg,
                        index,
                        buf,
                        dev,
                    });
                }
            }
        }
        bail!("Couldn't find a usable gadget device")
    }

    /// Release device back into the pool of available devices.
    pub fn release_device(&mut self, device: HIDGadgetDeviceFile<G>) -> Result<()> {
        let HIDGadgetDeviceFile { index, dev, .. } = device;
        match self.devices.get_mut(index) {
            None => bail!(
                "Internal consistency error in release_device: index {index} is out of bounds!"
            ),
            Some(v) if v.is_some() => {
                bail!("Internal consistency error in release_device: index {index} is not empty!")
            }
            Some(v) => {
                let _ = v.insert(dev);
            }
        }
        Ok(())
    }
}
