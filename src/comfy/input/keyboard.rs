use wlroots::key_events::KeyEvent as WLRKeyEvent;
use wlroots::wlroots_sys::wlr_key_state::WLR_KEY_PRESSED;
use wlroots::xkbcommon::xkb::{keysym_from_name, KEYSYM_NO_FLAGS};
use wlroots::{
	CompositorHandle as WLRCompositorHandle, KeyboardHandle as WLRKeyboardHandle, KeyboardHandler as WLRKeyboardHandler,
};

use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;

use compositor::commands::interpreter::CommandInterpreter;
use compositor::ComfyKernel;
use compositor::{CompositorMode, SuperModeState};

/*
.##..##..######..##..##..#####....####....####...#####...#####..
.##.##...##.......####...##..##..##..##..##..##..##..##..##..##.
.####....####......##....#####...##..##..######..#####...##..##.
.##.##...##........##....##..##..##..##..##..##..##..##..##..##.
.##..##..######....##....#####....####...##..##..##..##..#####..
................................................................
*/

pub struct KeyboardHandler;
impl KeyboardHandler {
	// NORMAL MODE
	fn handle_normal_mode_key_press(
		&mut self,
		comfy_kernel: &mut ComfyKernel,
		_: WLRKeyboardHandle,
		key_event: &WLRKeyEvent,
	) {
		if key_event.pressed_keys().contains(&comfy_kernel.super_mode_xkb_key) {
			comfy_kernel.current_mode = CompositorMode::SuperMode(SuperModeState::new());
		}
	}

	fn handle_normal_mode_key_release(&mut self, _: &mut ComfyKernel, _: WLRKeyboardHandle, _: &WLRKeyEvent) {}

	// SUPER MODE
	fn handle_super_mode_key_press(
		&mut self,
		comfy_kernel: &mut ComfyKernel,
		_: WLRKeyboardHandle,
		key_event: &WLRKeyEvent,
	) {
		// TODO: DJDUBE - Clean this mess!
		let key_set = XkbKeySet::from_vec_without_check(&key_event.pressed_keys());
		let xkb_keysyms_set_option = match comfy_kernel.current_mode {
			CompositorMode::NormalMode => None,
			CompositorMode::SuperMode(ref mut super_mode_state) => {
				super_mode_state.xkb_key_set.keysyms_set = super_mode_state
					.xkb_key_set
					.keysyms_set
					.union(&key_set.keysyms_set)
					.cloned()
					.collect();
				Some(super_mode_state.xkb_key_set.clone())
			}
		};

		if let Some(xkb_keysyms_set) = xkb_keysyms_set_option {
			debug!("super_mode_state.xkb_key_set.xkb_keysyms_set: {:?}", xkb_keysyms_set);
			if comfy_kernel.available_commands.contains_key(&xkb_keysyms_set.clone()) {
				let command = comfy_kernel
					.available_commands
					.get(&xkb_keysyms_set.clone())
					.unwrap()
					.clone();
				CommandInterpreter::execute(&command, comfy_kernel);
			}
		}
	}

	fn handle_super_mode_key_release(
		&mut self,
		comfy_kernel: &mut ComfyKernel,
		_keyboard: WLRKeyboardHandle,
		key_event: &WLRKeyEvent,
	) {
		// TODO: DJDUBE - Clean this mess!
		let key_set = XkbKeySet::from_vec_without_check(&key_event.pressed_keys());
		match comfy_kernel.current_mode {
			CompositorMode::NormalMode => {}
			CompositorMode::SuperMode(ref mut super_mode_state) => {
				super_mode_state.xkb_key_set.keysyms_set = super_mode_state
					.xkb_key_set
					.keysyms_set
					.difference(&key_set.keysyms_set)
					.cloned()
					.collect();
				debug!(
					"super_mode_state.xkb_key_set.keysyms_set: {:?}",
					super_mode_state.xkb_key_set.keysyms_set
				);
			}
		}

		if key_event.pressed_keys().contains(&comfy_kernel.super_mode_xkb_key) {
			comfy_kernel.current_mode = CompositorMode::NormalMode;
		}
	}
}
impl WLRKeyboardHandler for KeyboardHandler {
	fn modifiers(&mut self, compositor: WLRCompositorHandle, keyboard_handle: WLRKeyboardHandle) {
		dehandle!(
			@compositor = {compositor};
			let comfy_kernel: &mut ComfyKernel = compositor.into();

			let seat_handle = comfy_kernel.seat_handle.clone().unwrap();
			@seat = {seat_handle};

			@keyboard = {keyboard_handle};

			let mut modifiers = keyboard.get_modifier_masks();
			seat.keyboard_notify_modifiers(&mut modifiers);
			()
		);
	}

	fn on_key(&mut self, compositor: WLRCompositorHandle, keyboard_handle: WLRKeyboardHandle, key_event: &WLRKeyEvent) {
		dehandle!(
			@compositor = {compositor};
			let comfy_kernel: &mut ComfyKernel = compositor.into();

			let seat_handle = comfy_kernel.seat_handle.clone().unwrap();
			@seat = {seat_handle};

			match comfy_kernel.current_mode {
				CompositorMode::NormalMode => {
					if key_event.key_state() == WLR_KEY_PRESSED {
						self.handle_normal_mode_key_press(comfy_kernel, keyboard_handle, key_event);
					} else {
						self.handle_normal_mode_key_release(comfy_kernel, keyboard_handle, key_event);
					}

					// TODO: DJDUBE - Put this is a log file
					debug!("Notifying seat of keypress: time_msec: '{:?}' keycode: '{}' key_state: '{}'", key_event.time_msec(), key_event.keycode(), key_event.key_state() as u32);
					seat.keyboard_notify_key(
						key_event.time_msec(),
						key_event.keycode(),
						key_event.key_state() as u32
					);
				},
				CompositorMode::SuperMode(_) => {
					if key_event.key_state() == WLR_KEY_PRESSED {
						self.handle_super_mode_key_press(comfy_kernel, keyboard_handle, key_event);
					} else {
						self.handle_super_mode_key_release(comfy_kernel, keyboard_handle, key_event);
					}
				}
			};
			()
		);
	}
}

/*
.##..##..##..##..#####...##..##..######..##..##...####...######..######.
..####...##.##...##..##..##.##...##.......####...##......##........##...
...##....####....#####...####....####......##.....####...####......##...
..####...##.##...##..##..##.##...##........##........##..##........##...
.##..##..##..##..#####...##..##..######....##.....####...######....##...
........................................................................
*/

#[derive(PartialEq, Debug, Eq, Clone)]
pub struct XkbKeySet {
	pub keysyms_set: HashSet<u32>,
}
impl XkbKeySet {
	pub fn new() -> Self {
		XkbKeySet {
			keysyms_set: HashSet::<u32>::new(),
		}
	}

	/// Parses the given string into a XkbKeySet which contains a set of xkb keys (u32).
	/// The provided string should correspond to a list of xkb keys separated only by '+' characters.
	///
	pub fn from_str(key_set_string: &str) -> Result<XkbKeySet, String> {
		if key_set_string.is_empty() {
			return Err("Provided string is empty".to_string());
		}

		let mut extracted_keysyms = HashSet::<u32>::new();
		for keysym_name in key_set_string.split("+") {
			let key = keysym_from_name(keysym_name, KEYSYM_NO_FLAGS);
			if key == 0 {
				return Err(format!("Encountered unknown keysym: {}", keysym_name));
			}
			if extracted_keysyms.contains(&key) {
				return Err(format!("Encountered duplicate keysym: {}", keysym_name));
			}
			extracted_keysyms.insert(key);
		}

		Ok(XkbKeySet {
			keysyms_set: extracted_keysyms,
		})
	}

	/// Use the provided vector of xkb key codes to build an XkbKeySet which contains a set of xkb keys (u32).
	///
	pub fn from_vec_without_check(xkb_key_codes: &[u32]) -> XkbKeySet {
		XkbKeySet {
			keysyms_set: HashSet::from_iter(xkb_key_codes.iter().cloned()),
		}
	}
}

impl Hash for XkbKeySet {
	fn hash<H: Hasher>(&self, state: &mut H) {
		let mut hash_code = 0;
		let nb_elements = self.keysyms_set.len() as u32;
		for val in &self.keysyms_set {
			hash_code = hash_code ^ (val * nb_elements);
		}
		hash_code.hash(state);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use wlroots::xkbcommon::xkb::keysyms;

	// generate_from_string

	#[test]
	fn generate_from_string_fails_with_wrong_keysym() {
		// ? Testing 'heck'
		assert!(XkbKeySet::from_str("heck").is_err());

		// ? Testing 'Control_L' + 'heck'
		assert!(XkbKeySet::from_str("Control_L+heck").is_err());
	}

	#[test]
	fn generate_from_string_fails_with_empty_string() {
		assert!(XkbKeySet::from_str("").is_err());
	}

	#[test]
	fn generate_from_string_fails_with_duplicates() {
		assert!(XkbKeySet::from_str("Control_L+Control_L").is_err());
	}

	#[test]
	fn generate_from_string_succeeds_with_valid_string() {
		// ? Testing 'plus'
		match XkbKeySet::from_str("plus") {
			Err(e) => {
				error!("ERROR: {}", e);
				assert!(false);
			}
			Ok(xkb_key_set) => {
				assert_eq!(xkb_key_set.keysyms_set, [keysyms::KEY_plus].iter().cloned().collect());
			}
		}

		// ? Testing 'Control_L' + 'a'
		match XkbKeySet::from_str("Control_L+a") {
			Err(e) => {
				error!("ERROR: {}", e);
				assert!(false);
			}
			Ok(xkb_key_set) => {
				assert_eq!(
					xkb_key_set.keysyms_set,
					[keysyms::KEY_Control_L, keysyms::KEY_a].iter().cloned().collect()
				);
			}
		}

		// ? Testing 'Control_L' + 'Alt_L' + 'Delete'
		match XkbKeySet::from_str("Control_L+Alt_L+Delete") {
			Err(e) => {
				error!("ERROR: {}", e);
				assert!(false);
			}
			Ok(xkb_key_set) => {
				assert_eq!(
					xkb_key_set.keysyms_set,
					[keysyms::KEY_Control_L, keysyms::KEY_Alt_L, keysyms::KEY_Delete]
						.iter()
						.cloned()
						.collect()
				);
			}
		}
	}
}
