use wlroots::{
	CompositorHandle as WLRCompositorHandle, SurfaceHandler as WLRSurfaceHandler,
	XdgV6ShellHandler as WLRXdgV6ShellHandler, XdgV6ShellManagerHandler as WLRXdgV6ShellManagerHandler,
	XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle,
};

use compositor::surface::SurfaceHandler;
use compositor::ComfyKernel;

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
	fn destroyed(&mut self, compositor: WLRCompositorHandle, shell: WLRXdgV6ShellSurfaceHandle) {
		with_handles!([(compositor: {compositor})] => {
			let comfy_kernel: &mut ComfyKernel = compositor.into();
			let weak = shell;
			if let Some(index) = comfy_kernel.shells.iter().position(|s| *s == weak) {
				comfy_kernel.shells.remove(index);
			}
		}).unwrap();
	}
}

pub struct XdgV6ShellManagerHandler;
impl WLRXdgV6ShellManagerHandler for XdgV6ShellManagerHandler {
	fn new_surface(
		&mut self,
		compositor_handle: WLRCompositorHandle,
		shell_handle: WLRXdgV6ShellSurfaceHandle,
	) -> (Option<Box<WLRXdgV6ShellHandler>>, Option<Box<WLRSurfaceHandler>>) {
		shell_handle.run(|shell| shell.ping()).unwrap();
		compositor_handle.run(|compositor| {
			let comfy_kernel: &mut ComfyKernel = compositor.into();
			comfy_kernel.add_window_to_active_workspace(shell_handle);
		}).unwrap();
		(Some(Box::new(XdgV6ShellHandler)), Some(Box::new(SurfaceHandler)))
	}
}
