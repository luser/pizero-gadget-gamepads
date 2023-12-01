use anyhow::{anyhow, bail, Result};
use env_logger::Builder;
use gilrs::{ev::Code, Button, Event, EventType, Gamepad, GamepadId, GilrsBuilder};
use gilrs::{Axis, MappingSource};
use log::{debug, error, info, log_enabled, Level, LevelFilter};
use signal_hook::consts::signal::*;
use signal_hook::flag as signal_flag;
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use pizero_gadget_gamepads::hid_gadget::*;
use pizero_gadget_gamepads::hori_pokken::*;
use pizero_gadget_gamepads::HIDGamepad;

struct RealGamepadToGadgetMapping<G: HIDGamepad> {
    /// The mapping of standardized buttons to event codes for this device.
    button_map: HashMap<Button, Code>,
    /// The mapping of standardized axes to event codes for this device.
    axis_map: HashMap<Axis, Code>,
    /// The gadget device to which we're routing its data.
    gadget_file: HIDGadgetDeviceFile<G>,
}

fn try_map_gamepad<G: HIDGamepad>(
    gamepad: &Gamepad,
    gadget: &mut HIDGadget<G>,
    mappings: &mut HashMap<GamepadId, RealGamepadToGadgetMapping<G>>,
) -> Result<()> {
    info!("Gamepad connected: {}", gamepad.name());
    if gamepad.mapping_source() != MappingSource::SdlMappings {
        bail!("Not using gamepad {}, no mapping data", gamepad.name());
    }
    let gadget_file = gadget.take_device()?;
    let mut button_map = HashMap::new();
    use Button::*;
    for button in [
        South,
        East,
        North,
        West,
        // Intentionally skipping C,
        // Intentionally skipping Z,
        LeftTrigger,
        LeftTrigger2,
        RightTrigger,
        RightTrigger2,
        Select,
        Start,
        Mode,
        LeftThumb,
        RightThumb,
        DPadUp,
        DPadDown,
        DPadLeft,
        DPadRight,
    ] {
        if let Some(code) = gamepad.button_code(button) {
            button_map.insert(button, code);
        }
    }
    let mut axis_map = HashMap::new();
    use Axis::*;
    for axis in [LeftStickX, LeftStickY, RightStickX, RightStickY] {
        if let Some(code) = gamepad.axis_code(axis) {
            axis_map.insert(axis, code);
        }
    }
    info!("Mapping {} to {}", gamepad.name(), gadget_file.path());
    if log_enabled!(Level::Debug) {
        let mut s = "  Axes:\n".to_owned();
        for (a, c) in axis_map.iter() {
            drop(writeln!(&mut s, "    {c} => {a:?}"));
        }
        drop(writeln!(&mut s, "  Buttons:"));
        for (b, c) in button_map.iter() {
            drop(writeln!(&mut s, "    {c} => {b:?}"));
        }
        debug!("Mapping:\n{s}");
    }
    mappings.insert(
        gamepad.id(),
        RealGamepadToGadgetMapping {
            button_map,
            axis_map,
            gadget_file,
        },
    );
    Ok(())
}

fn update_gamepad<G: HIDGamepad>(
    gamepad: &Gamepad,
    mapping: &mut RealGamepadToGadgetMapping<G>,
) -> Result<()> {
    let report = G::fill_report(gamepad, &mapping.button_map, &mapping.axis_map);
    if log_enabled!(Level::Debug) {
        debug!("{}: {report}", gamepad.name());
    }
    mapping.gadget_file.write_report(report)
}

fn create_and_run_gamepad_gadgets<G: HIDGamepad>(term: Arc<AtomicBool>) -> Result<()> {
    let mut gadget = HIDGadget::<HoriPokkenPad>::create(1)?;
    info!(
        "Created gadget '{:?}' with {} gamepads",
        gadget.path,
        gadget.device_count()
    );

    let mut gilrs = GilrsBuilder::new()
        .add_mappings(include_str!("../extra-mappings.txt"))
        .build()
        .map_err(|e| anyhow!("Error initializing gilrs: {e}"))?;
    let mut gamepad_mappings = HashMap::new();

    // Iterate over all connected gamepads
    for (_id, gamepad) in gilrs.gamepads() {
        if let Err(e) = try_map_gamepad(&gamepad, &mut gadget, &mut gamepad_mappings) {
            error!("{e}");
        }
    }

    while !term.load(Ordering::Relaxed) {
        while let Some(Event { id, event, .. }) = gilrs.next_event() {
            match event {
                EventType::Connected => {
                    let gamepad = gilrs.gamepad(id);
                    if let Err(e) = try_map_gamepad(&gamepad, &mut gadget, &mut gamepad_mappings) {
                        error!("{e}");
                    }
                }
                EventType::Disconnected => {
                    info!("Gamepad disconnected: {id}");
                    if let Some(RealGamepadToGadgetMapping { gadget_file, .. }) =
                        gamepad_mappings.remove(&id)
                    {
                        let _ = gadget.release_device(gadget_file);
                    }
                }
                EventType::Dropped => {}
                // If the gamepad we're providing doesn't have any analog buttons then we don't need
                // to handle ButtonChanged events, and since gilrs will generate one per press/release
                // that would cause us to write an extra report every time.
                EventType::ButtonChanged(_, _, _) if !G::ANALOG_BUTTONS => {}
                _ => {
                    let gamepad = gilrs.gamepad(id);
                    let mapping = gamepad_mappings.get_mut(&id);
                    if let Some(mapping) = mapping {
                        let _ = update_gamepad(&gamepad, mapping);
                    }
                }
            }
        }
    }
    // Release any in-use gadgets before cleaning up for real.
    for mapping in gamepad_mappings.into_values() {
        let RealGamepadToGadgetMapping { gadget_file, .. } = mapping;
        let _ = gadget.release_device(gadget_file);
    }
    Ok(())
}

fn main() -> Result<()> {
    Builder::new()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();
    let term = Arc::new(AtomicBool::new(false));
    signal_flag::register(SIGINT, Arc::clone(&term))?;
    signal_flag::register(SIGTERM, Arc::clone(&term))?;
    signal_flag::register(SIGQUIT, Arc::clone(&term))?;
    create_and_run_gamepad_gadgets::<HoriPokkenPad>(term)?;
    info!("Shutting down");
    Ok(())
}
