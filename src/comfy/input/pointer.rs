use std::time::Duration;
use wlroots::pointer_events::{AbsoluteMotionEvent, AxisEvent, ButtonEvent, MotionEvent};

use wlroots::{
	CompositorHandle as WLRCompositorHandle, PointerHandle as WLRPointerHandle, PointerHandler as WLRPointerHandler,
};

use compositor::ComfyKernel;
use config::global::PointerFocusType;

// A pointer is a physical device (a mouse, a touchscreen, etc...) that subscribe to an event in order to communicate
// with the compositor.

pub struct PointerHandler;
impl WLRPointerHandler for PointerHandler {
	#[wlroots_dehandle(compositor)]
	fn on_motion_absolute(
		&mut self,
		compositor_handle: WLRCompositorHandle,
		_: WLRPointerHandle,
		event: &AbsoluteMotionEvent,
	) {
		debug!("PointerHandler: on_motion_absolute");
		use compositor_handle as compositor;
		let comfy_kernel: &mut ComfyKernel = compositor.into();

		comfy_kernel.warp_cursor(event);

		if comfy_kernel.config.global.pointer_focus_type == PointerFocusType::OnHover {
			comfy_kernel.apply_focus_under_cursor();
		}

		let duration = Duration::from_millis(event.time_msec() as u64);
		comfy_kernel.transfer_motion_to_seat(duration);
	}

	#[wlroots_dehandle(compositor, cursor)]
	fn on_motion(&mut self, compositor_handle: WLRCompositorHandle, _: WLRPointerHandle, event: &MotionEvent) {
		debug!("PointerHandler: on_motion");
		use compositor_handle as compositor;
		let comfy_kernel: &mut ComfyKernel = compositor.into();
		let (cursor_x, cursor_y) = comfy_kernel.get_cursor_coordinates();
		debug!("PointerHandler: on_motion - (cursor_x, cursor_y): ({}, {})", cursor_x, cursor_y);
		{
			let (delta_x, delta_y) = event.delta();
			let cursor_handle = &comfy_kernel.cursor_handle;
			use cursor_handle as cursor;
			cursor.move_to(event.device(), delta_x, delta_y);
		}
		if comfy_kernel.config.global.pointer_focus_type == PointerFocusType::OnHover {
			comfy_kernel.apply_focus_under_cursor();
		}
	}

	#[wlroots_dehandle(compositor)]
	fn on_button(&mut self, compositor_handle: WLRCompositorHandle, _: WLRPointerHandle, button_event: &ButtonEvent) {
		debug!("PointerHandler: on_button");
		use compositor_handle as compositor;

		let comfy_kernel: &mut ComfyKernel = compositor.into();
		if comfy_kernel.config.global.pointer_focus_type == PointerFocusType::OnClick {
			comfy_kernel.apply_focus_under_cursor();
		}

		let button = button_event.button();
		let state = button_event.state();
		let duration = Duration::from_millis(button_event.time_msec() as u64);
		comfy_kernel.transfer_click_to_seat(duration, button, state as u32)
	}

	#[wlroots_dehandle(compositor)]
	fn on_axis(&mut self, compositor_handle: WLRCompositorHandle, _: WLRPointerHandle, axis_event: &AxisEvent) {
		use compositor_handle as compositor;

		let comfy_kernel: &mut ComfyKernel = compositor.into();
		let duration = Duration::from_millis(axis_event.time_msec() as u64);
		let orientation = axis_event.orientation();
		let value = axis_event.delta();
		let source = axis_event.source();
		comfy_kernel.transfer_scroll_to_seat(duration, orientation, value, value as i32, source)
	}
}
