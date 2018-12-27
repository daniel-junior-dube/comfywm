use wlroots::{
	CompositorHandle as WLRCompositorHandle, SurfaceHandler as WLRSurfaceHandler,
	XdgV6ShellHandler as WLRXdgV6ShellHandler, XdgV6ShellManagerHandler as WLRXdgV6ShellManagerHandler,
	XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle,
};

use compositor::surface::SurfaceHandler;
use compositor::ComfyKernel;
use utils::handle_helper::shell_handle_helper;

/*
..####...##..##..######..##......##.....
.##......##..##..##......##......##.....
..####...######..####....##......##.....
.....##..##..##..##......##......##.....
..####...##..##..######..######..######.
........................................
*/

pub struct XdgV6ShellHandler;
impl WLRXdgV6ShellHandler for XdgV6ShellHandler {
	#[wlroots_dehandle(compositor)]
	fn destroyed(&mut self, compositor_handle: WLRCompositorHandle, shell_handle: WLRXdgV6ShellSurfaceHandle) {
		// ? We only add the shell handle as a window if it's a top level
		use compositor_handle as compositor;

		if shell_handle_helper::is_top_level(&shell_handle) {
			let comfy_kernel: &mut ComfyKernel = compositor.into();
			comfy_kernel.find_and_remove_window(&shell_handle);
		}
	}
}

pub struct XdgV6ShellManagerHandler;
impl WLRXdgV6ShellManagerHandler for XdgV6ShellManagerHandler {
	#[wlroots_dehandle(compositor)]
	fn new_surface(
		&mut self,
		compositor_handle: WLRCompositorHandle,
		shell_handle: WLRXdgV6ShellSurfaceHandle,
	) -> (Option<Box<WLRXdgV6ShellHandler>>, Option<Box<WLRSurfaceHandler>>) {
		// ? We only add the shell handle as a window if it's a top level
		if shell_handle_helper::is_top_level(&shell_handle) {
			use compositor_handle as compositor;
			let comfy_kernel: &mut ComfyKernel = compositor.into();
			comfy_kernel.add_window_to_active_workspace(shell_handle);
		}
		(Some(Box::new(XdgV6ShellHandler)), Some(Box::new(SurfaceHandler)))
	}
}
