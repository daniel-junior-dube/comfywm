use wlroots::{
	CompositorHandle as WLRCompositorHandle, InputManagerHandler as WLRInputManagerHandler,
	KeyboardHandle as WLRKeyboardHandle, KeyboardHandler as WLRKeyboardHandler, PointerHandle as WLRPointerHandle,
	PointerHandler as WLRPointerHandler,
};

use compositor::State;
use input::cursor::PointerHandler;
use input::keyboard::KeyboardHandler;

pub mod cursor;
pub mod keyboard;
pub mod seat;

/*
.######..##..##..#####...##..##..######.
...##....###.##..##..##..##..##....##...
...##....##.###..#####...##..##....##...
...##....##..##..##......##..##....##...
.######..##..##..##.......####.....##...
........................................
*/

pub struct InputManagerHandler;
impl WLRInputManagerHandler for InputManagerHandler {
	fn pointer_added(&mut self, _: WLRCompositorHandle, _: WLRPointerHandle) -> Option<Box<WLRPointerHandler>> {
		Some(Box::new(PointerHandler))
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
			state.keyboard_handle = Some(keyboard.weak_reference());
			@seat = {state.seat_handle.as_ref().unwrap()};
			seat.set_keyboard(keyboard.input_device())
		);
		Some(Box::new(KeyboardHandler))
	}
}
