use wlroots::key_events::KeyEvent as WLRKeyEvent;
use wlroots::wlroots_sys::wlr_key_state::WLR_KEY_PRESSED;
use wlroots::xkbcommon::xkb::keysyms;
use wlroots::{
	terminate as wlr_terminate, CompositorHandle as WLRCompositorHandle, KeyboardHandle as WLRKeyboardHandle,
	KeyboardHandler as WLRKeyboardHandler,
};

use std::process::Command;
use std::thread;

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
							Command::new("weston-terminal").output().unwrap();
						});
						return
					}
				}
			};
			let comfy_kernel: &mut ComfyKernel = compositor.into();
			let seat_handle = comfy_kernel.seat_handle.clone().unwrap();
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
