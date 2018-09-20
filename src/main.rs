#[macro_use]
extern crate wlroots;

use wlroots::utils::{
	init_logging as wlr_init_logging,
	WLR_DEBUG
};
use wlroots::{
	CompositorBuilder,
	CompositorHandle as WLRCompositorHandle,
	KeyboardHandle as WLRKeyboardHandle,
	KeyboardHandler as WLRKeyboardHandler,
	InputManagerHandler as WLRInputManagerHandler
};
use wlroots::key_events::KeyEvent as WLRKeyEvent;
use wlroots::xkbcommon::xkb::keysyms::KEY_Escape;

struct KeyboardHandler;

impl WLRKeyboardHandler for KeyboardHandler {
	fn on_key(&mut self, _:WLRCompositorHandle, keyboard:WLRKeyboardHandle, key_event:&WLRKeyEvent) {
		let keys = key_event.pressed_keys();
		println!("Key press detected!");
		with_handles!([(keyboard: {keyboard})] => {
			for key in keys {
				match key {
					KEY_Escape => {
						wlroots::terminate();
					},
					_ => {}
				}
			}
		}).unwrap();
	}
}

struct InputManager;

impl WLRInputManagerHandler for InputManager {
	fn keyboard_added(&mut self, _:WLRCompositorHandle, _:WLRKeyboardHandle) -> Option<Box<WLRKeyboardHandler>> {
		Some(Box::new(KeyboardHandler))
	}
}

fn main() {
	//let args: Vec<String> = env::args().collect();
	wlr_init_logging(WLR_DEBUG, None);
	CompositorBuilder::new()
		.input_manager(Box::new(InputManager))
		.build_auto(())
		.run()
}
