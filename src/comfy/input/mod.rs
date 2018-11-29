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
	#[wlroots_dehandle(compositor, keyboard, seat)]
	fn keyboard_added(
		&mut self,
		compositor_handle: WLRCompositorHandle,
		keyboard_handle: WLRKeyboardHandle,
	) -> Option<Box<WLRKeyboardHandler>> {
		{
			use compositor_handle as compositor;
			use keyboard_handle as keyboard;
			let comfy_kernel: &mut ComfyKernel = compositor.into();
			comfy_kernel.keyboard_handle = Some(keyboard.weak_reference());
			let seat_handle = comfy_kernel.seat_handle.as_ref().unwrap();
			use seat_handle as seat;
			seat.set_keyboard(keyboard.input_device());
		}
		Some(Box::new(KeyboardHandler))
	}
}
