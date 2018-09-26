use wlroots::{
	CompositorHandle as WLRCompositorHandle, SurfaceHandler as WLRSurfaceHandler,
	XdgV6ShellHandler as WLRXdgV6ShellHandler, XdgV6ShellManagerHandler as WLRXdgV6ShellManagerHandler,
	XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle,
};

use state::State;
use surface::Surface;

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
			let state: &mut State = compositor.into();
			let weak = shell;
			if let Some(index) = state.shells.iter().position(|s| *s == weak) {
				state.shells.remove(index);
			}
		}).unwrap();
	}
}

pub struct XdgV6ShellManager;
impl WLRXdgV6ShellManagerHandler for XdgV6ShellManager {
	fn new_surface(
		&mut self,
		compositor: WLRCompositorHandle,
		shell: WLRXdgV6ShellSurfaceHandle,
	) -> (Option<Box<WLRXdgV6ShellHandler>>, Option<Box<WLRSurfaceHandler>>) {
		dehandle!(
				@compositor = {compositor};
				@shell = {shell};
				shell.ping();
				let state: &mut State = compositor.into();
				state.shells.push(shell.weak_reference());
				@layout = {&state.layout};
				for (mut output, _) in layout.outputs() => {
						@output = {output};
						output.schedule_frame()
				}
				()
			);
		(Some(Box::new(XdgV6ShellHandler)), Some(Box::new(Surface)))
	}
}
