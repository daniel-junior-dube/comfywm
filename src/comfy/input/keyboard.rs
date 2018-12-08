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
	fn handle_key_press(&mut self, comfy_kernel: &mut ComfyKernel, key_event: &WLRKeyEvent) {
		let key_set = XkbKeySet::from_vec_without_check(&key_event.pressed_keys());

		comfy_kernel.currently_pressed_keys.set_to_union(&key_set);

		if let Some(command) = comfy_kernel.command_for_keyset(&comfy_kernel.currently_pressed_keys) {
			CommandInterpreter::execute(&command, comfy_kernel);
		} else {
			comfy_kernel.notify_keyboard(key_event);
		}
	}

	fn handle_key_release(&mut self, comfy_kernel: &mut ComfyKernel, key_event: &WLRKeyEvent) {
		let key_set = XkbKeySet::from_vec_without_check(&key_event.pressed_keys());
		comfy_kernel.currently_pressed_keys.set_to_difference(&key_set);
		comfy_kernel.notify_keyboard(key_event);
	}
}

impl WLRKeyboardHandler for KeyboardHandler {
	#[wlroots_dehandle(compositor, seat, keyboard)]
	fn modifiers(&mut self, compositor_handle: WLRCompositorHandle, keyboard_handle: WLRKeyboardHandle) {
		use compositor_handle as compositor;
		use keyboard_handle as keyboard;

		let comfy_kernel: &mut ComfyKernel = compositor.into();

		let seat_handle = comfy_kernel.seat_handle.clone().unwrap();
		use seat_handle as seat;

		// TODO: Should we prevent the notification of the mod key if super mode is engaged?
		let mut modifiers = keyboard.get_modifier_masks();
		seat.keyboard_notify_modifiers(&mut modifiers);
	}

	#[wlroots_dehandle(compositor)]
	fn on_key(&mut self, compositor_handle: WLRCompositorHandle, _: WLRKeyboardHandle, key_event: &WLRKeyEvent) {
		use compositor_handle as compositor;
		let comfy_kernel: &mut ComfyKernel = compositor.into();

		if key_event.key_state() == WLR_KEY_PRESSED {
			self.handle_key_press(comfy_kernel, key_event);
		} else {
			self.handle_key_release(comfy_kernel, key_event);
		}
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

	// Set the the keysyms_set as the difference of the current one with the provided one
	pub fn set_to_difference(&mut self, key_set: &XkbKeySet) {
		self.keysyms_set = self.keysyms_set.difference(&key_set.keysyms_set).cloned().collect();
	}

	// Set the the keysyms_set as the difference of the current one with the provided one
	pub fn set_to_union(&mut self, key_set: &XkbKeySet) {
		self.keysyms_set = self.keysyms_set.union(&key_set.keysyms_set).cloned().collect();
	}

	/// Use the provided vector of xkb key codes to build an XkbKeySet which contains a set of xkb keys (u32).
	/// Doesn't check if the vec is empty or contains duplicates or invalid keysyms
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
