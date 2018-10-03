use wlroots::pointer_events::{AbsoluteMotionEvent, ButtonEvent, MotionEvent};
use wlroots::{
	CompositorHandle as WLRCompositorHandle, CursorHandler as WLRCursorHandler, PointerHandle as WLRPointerHandle,
	PointerHandler as WLRPointerHandler, XdgV6ShellState as WLRXdgV6ShellState,
};

use compositor::ComfyKernel;

/*
..####...##..##..#####....####....####...#####..
.##..##..##..##..##..##..##......##..##..##..##.
.##......##..##..#####....####...##..##..#####..
.##..##..##..##..##..##......##..##..##..##..##.
..####....####...##..##...####....####...##..##.
................................................
*/

pub struct CursorHandler;
impl WLRCursorHandler for CursorHandler {}

pub struct PointerHandler;
impl WLRPointerHandler for PointerHandler {
	fn on_motion_absolute(&mut self, compositor: WLRCompositorHandle, _: WLRPointerHandle, event: &AbsoluteMotionEvent) {
		dehandle!(
			@compositor = {compositor};
			let comfy_kernel: &mut ComfyKernel = compositor.into();
			let (x, y) = event.pos();
			@cursor = {&comfy_kernel.cursor_handle};
			cursor.warp_absolute(event.device(), x, y)
			/*
				TODO: If 'select on hover mode', set window 'activated' on mouse intersection
			*/
		);
	}

	fn on_motion(&mut self, compositor: WLRCompositorHandle, _: WLRPointerHandle, event: &MotionEvent) {
		dehandle!(
			@compositor = {compositor};
			let comfy_kernel: &mut ComfyKernel = compositor.into();
			let (delta_x, delta_y) = event.delta();
			@cursor = {&comfy_kernel.cursor_handle};
			cursor.move_to(event.device(), delta_x, delta_y)
		);
	}

	fn on_button(&mut self, compositor: WLRCompositorHandle, _: WLRPointerHandle, _: &ButtonEvent) {
		dehandle!(
			@compositor = {compositor};
			let comfy_kernel: &mut ComfyKernel = compositor.into();
			let seat = comfy_kernel.seat_handle.clone().unwrap();
			let keyboard = comfy_kernel.keyboard_handle.clone().unwrap();
			@seat = {seat};
			@keyboard = {keyboard};
			if comfy_kernel.shells.len() > 0 {
				comfy_kernel.shells[comfy_kernel.shells.len()-1].run(
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
						}).unwrap();
					}
				).unwrap();
			};
			()
		);
	}
}
