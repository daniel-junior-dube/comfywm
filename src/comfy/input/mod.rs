use wlroots::{
	CompositorHandle as WLRCompositorHandle, InputManagerHandler as WLRInputManagerHandler,
	KeyboardHandle as WLRKeyboardHandle, KeyboardHandler as WLRKeyboardHandler, PointerHandle as WLRPointerHandle,
	PointerHandler as WLRPointerHandler,
};

use compositor::ComfyKernel;
use input::keyboard::KeyboardHandler;
use input::pointer::PointerHandler;

pub mod cursor;
pub mod keyboard;
pub mod pointer;
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
			let comfy_kernel: &mut ComfyKernel = compositor.into();
			comfy_kernel.keyboard_handle = Some(keyboard.weak_reference());
			@seat = {comfy_kernel.seat_handle.as_ref().unwrap()};
			seat.set_keyboard(keyboard.input_device())
		);
		Some(Box::new(KeyboardHandler))
	}
}
