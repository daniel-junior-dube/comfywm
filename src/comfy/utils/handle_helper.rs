/*
..####...##..##..######..##......##......##..##...####...##..##..#####...##......######.
.##......##..##..##......##......##......##..##..##..##..###.##..##..##..##......##.....
..####...######..####....##......##......######..######..##.###..##..##..##......####...
.....##..##..##..##......##......##......##..##..##..##..##..##..##..##..##......##.....
..####...##..##..######..######..######..##..##..##..##..##..##..#####...######..######.
........................................................................................
.##..##..######..##......#####...######..#####..
.##..##..##......##......##..##..##......##..##.
.######..####....##......#####...####....#####..
.##..##..##......##......##......##......##..##.
.##..##..######..######..##......######..##..##.
................................................
*/

pub mod shell_handle_helper {
	use wlroots::{XdgV6ShellState as WLRXdgV6ShellState, XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle};

	/// Returns the area (geometry) of the shell of the provided shell handle.
	pub fn is_top_level(shell_handle: &WLRXdgV6ShellSurfaceHandle) -> bool {
		shell_handle
			.run(|shell| match shell.state().unwrap() {
				WLRXdgV6ShellState::TopLevel(_) => true,
				_ => false,
			}).unwrap()
	}
}

pub mod surface_helper {
	use wlroots::Surface as WLRSurface;

	/// Returns true if the provided surface is a top level surface.
	pub fn is_top_level(surface: &WLRSurface) -> bool {
		if let Some(role) = surface.role() {
			return role == "xdg_toplevel_v6";
		}
		false
	}
}
