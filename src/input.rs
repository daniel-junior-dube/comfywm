use wlroots::key_events::KeyEvent as WLRKeyEvent;
use wlroots::pointer_events::{AbsoluteMotionEvent, ButtonEvent, MotionEvent};
use wlroots::wlroots_sys::wlr_key_state::WLR_KEY_PRESSED;
use wlroots::xkbcommon::xkb::keysyms;
use wlroots::{
	terminate as wlr_terminate, CompositorHandle as WLRCompositorHandle, CursorHandler as WLRCursorHandler,
	InputManagerHandler as WLRInputManagerHandler, KeyboardHandle as WLRKeyboardHandle,
	KeyboardHandler as WLRKeyboardHandler, PointerHandle as WLRPointerHandle, PointerHandler as WLRPointerHandler,
	XdgV6ShellState as WLRXdgV6ShellState,
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
							Command::new("weston-terminal").output().unwrap();
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
..####...##..##..#####....####....####...#####..
.##..##..##..##..##..##..##......##..##..##..##.
.##......##..##..#####....####...##..##..#####..
.##..##..##..##..##..##......##..##..##..##..##.
..####....####...##..##...####....####...##..##.
................................................
*/

pub struct Cursor;
impl WLRCursorHandler for Cursor {}

pub struct Pointer;
impl WLRPointerHandler for Pointer {
	fn on_motion_absolute(&mut self, compositor: WLRCompositorHandle, _: WLRPointerHandle, event: &AbsoluteMotionEvent) {
		dehandle!(
			@compositor = {compositor};
			let state: &mut State = compositor.into();
			let (x, y) = event.pos();
			@cursor = {&state.cursor};
			cursor.warp_absolute(event.device(), x, y)
		);
	}

	fn on_motion(&mut self, compositor: WLRCompositorHandle, _: WLRPointerHandle, event: &MotionEvent) {
		dehandle!(
			@compositor = {compositor};
			let state: &mut State = compositor.into();
			let (delta_x, delta_y) = event.delta();
			@cursor = {&state.cursor};
			cursor.move_to(event.device(), delta_x, delta_y)
		);
	}

	fn on_button(&mut self, compositor: WLRCompositorHandle, _: WLRPointerHandle, _: &ButtonEvent) {
		dehandle!(
			@compositor = {compositor};
			let state: &mut State = compositor.into();
			let seat = state.seat_handle.clone().unwrap();
			let keyboard = state.keyboard.clone().unwrap();
			@seat = {seat};
			@keyboard = {keyboard};
			if state.shells.len() > 0 {
				state.shells[0].run(
					|shell| {
						let surface = shell.surface();
						surface.run(|surface| {
							match shell.state() {
								Some(&mut WLRXdgV6ShellState::TopLevel(ref mut toplevel)) => {
									toplevel.set_activated(true);
								}
								_ => {}
							};
							seat.set_keyboard(keyboard.input_device());
							seat.keyboard_notify_enter(
								surface,
								&mut keyboard.keycodes(),
								&mut keyboard.get_modifier_masks()
							);
						});
					}
				).unwrap();
			};
			()
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
	fn pointer_added(&mut self, _: WLRCompositorHandle, _: WLRPointerHandle) -> Option<Box<WLRPointerHandler>> {
		Some(Box::new(Pointer))
	}

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
