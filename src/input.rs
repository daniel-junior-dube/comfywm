use wlroots::key_events::KeyEvent as WLRKeyEvent;
use wlroots::wlroots_sys::wlr_key_state::WLR_KEY_PRESSED;
use wlroots::xkbcommon::xkb::keysyms;
use wlroots::{
	terminate as wlr_terminate, CompositorHandle as WLRCompositorHandle, InputManagerHandler as WLRInputManagerHandler,
	KeyboardHandle as WLRKeyboardHandle, KeyboardHandler as WLRKeyboardHandler,
};

use std::process::Command;
use std::thread;

use state::State;

/*
.##..##..######..##..##..#####....####....####...#####...#####..
.##.##...##.......####...##..##..##..##..##..##..##..##..##..##.
.####....####......##....#####...##..##..######..#####...##..##.
.##.##...##........##....##..##..##..##..##..##..##..##..##..##.
.##..##..######....##....#####....####...##..##..##..##..#####..
................................................................
*/

pub struct KeyboardHandler;
impl WLRKeyboardHandler for KeyboardHandler {
	fn on_key(&mut self, compositor: WLRCompositorHandle, _keyboard: WLRKeyboardHandle, key_event: &WLRKeyEvent) {
		dehandle!(
			@compositor = {compositor};
			for key in key_event.pressed_keys() {
				if key == keysyms::KEY_Escape {
					wlr_terminate();
				} else if key_event.key_state() == WLR_KEY_PRESSED {
					if key == keysyms::KEY_F1 {
						thread::spawn(move || {
							Command::new("gnome-terminal").output().unwrap();
						});
						return
					}
				}
			};
			let state: &mut State = compositor.into();
			let seat_handle = state.seat_handle.clone().unwrap();
			@seat = {seat_handle};
			println!("Notifying seat of keypress: time_msec: '{:?}' keycode: '{}' key_state: '{}'", key_event.time_msec(), key_event.keycode(), key_event.key_state() as u32);
			seat.keyboard_notify_key(
				key_event.time_msec(),
				key_event.keycode(),
				key_event.key_state() as u32
			)
		);
	}
}

/*
.######..##..##..#####...##..##..######.
...##....###.##..##..##..##..##....##...
...##....##.###..#####...##..##....##...
...##....##..##..##......##..##....##...
.######..##..##..##.......####.....##...
........................................
*/

pub struct InputManager;
impl WLRInputManagerHandler for InputManager {
	fn keyboard_added(
		&mut self,
		compositor: WLRCompositorHandle,
		keyboard: WLRKeyboardHandle,
	) -> Option<Box<WLRKeyboardHandler>> {
		dehandle!(
			@compositor = {compositor};
			@keyboard = {keyboard};
			let state: &mut State = compositor.into();
			state.keyboard = Some(keyboard.weak_reference());
			@seat = {state.seat_handle.as_ref().unwrap()};
			seat.set_keyboard(keyboard.input_device())
		);
		Some(Box::new(KeyboardHandler))
	}
}
