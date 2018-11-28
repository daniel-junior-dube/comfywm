use std::{collections::HashMap, time::Duration};

use wlroots::key_events::KeyEvent as WLRKeyEvent;
use wlroots::{
	Capability, Compositor as WLRCompositor, CompositorBuilder as WLRCompositorBuilder, Cursor as WLRCursor,
	CursorHandle as WLRCursorHandle, KeyboardHandle as WLRKeyboardHandle, OutputLayout as WLROutputLayout,
	OutputLayoutHandle as WLROutputLayoutHandle, Seat as WLRSeat, SeatHandle as WLRSeatHandle,
	XCursorManager as WLRXCursorManager, XdgV6ShellState as WLRXdgV6ShellState,
	XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle,
};

use wlroots::wlroots_sys::{
	protocols::server_decoration::server::org_kde_kwin_server_decoration_manager::Mode as ServerDecorationMode,
	wlr_axis_orientation, wlr_axis_source,
};

pub mod commands;
pub mod output;
pub mod shell;
pub mod surface;
pub mod window;
pub mod workspace;

use self::commands::Command;
use self::output::{OutputData, OutputLayoutHandler, OutputManagerHandler};
use self::shell::XdgV6ShellManagerHandler;
use self::window::Window;
use self::workspace::Workspace;
use config::Config;
use input::cursor::CursorHandler;
use input::keyboard::XkbKeySet;
use input::seat::SeatHandler;
use input::InputManagerHandler;
use layout::LayoutDirection;

/*
..####....####...##...##..#####....####....####...######..######...####...#####..
.##..##..##..##..###.###..##..##..##..##..##........##......##....##..##..##..##.
.##......##..##..##.#.##..#####...##..##...####.....##......##....##..##..#####..
.##..##..##..##..##...##..##......##..##......##....##......##....##..##..##..##.
..####....####...##...##..##.......####....####...######....##.....####...##..##.
.................................................................................
*/

pub fn generate_default_compositor() -> WLRCompositor {
	// ? WIP: Initialize the cursor structures
	let cursor = WLRCursor::create(Box::new(CursorHandler));
	let mut xcursor_manager =
		WLRXCursorManager::create("default".to_string(), 24).expect("Could not create xcursor manager");
	xcursor_manager.load(1.0);
	cursor
		.run(|c| xcursor_manager.set_cursor_image("left_ptr".to_string(), c))
		.unwrap();

	// ? WIP: Initialize the output layout structure
	let layout = WLROutputLayout::create(Box::new(OutputLayoutHandler));

	// ? WIP: Initialize the compositor structure
	let mut compositor = WLRCompositorBuilder::new()
		.gles2(true)
		.data_device(true)
		.server_decoration_manager(true)
		.input_manager(Box::new(InputManagerHandler))
		.output_manager(Box::new(OutputManagerHandler))
		.xdg_shell_v6_manager(Box::new(XdgV6ShellManagerHandler))
		.build_auto(ComfyKernel::new(layout, xcursor_manager, cursor));

	// ? Use the server-side decoration mode to avoid client-side decoration
	// * Note: `ServerDecorationMode::None` does not seem to work
	if let Some(ref mut decoration_manager) = compositor.server_decoration_manager {
		decoration_manager.set_default_mode(ServerDecorationMode::Server);
	}

	// ? WIP: Initialize and add the seat structures to the kernel
	{
		let seat_handle = WLRSeat::create(&mut compositor, "seat0".into(), Box::new(SeatHandler));
		seat_handle
			.run(|seat| {
				seat.set_capabilities(Capability::all());
			}).unwrap();
		let comfy_kernel: &mut ComfyKernel = (&mut compositor).into();
		comfy_kernel.seat_handle = Some(seat_handle);
	}

	compositor
}

/*
..####....####...##...##..######..##..##..##..##..######..#####...##..##..######..##.....
.##..##..##..##..###.###..##.......####...##.##...##......##..##..###.##..##......##.....
.##......##..##..##.#.##..####......##....####....####....#####...##.###..####....##.....
.##..##..##..##..##...##..##........##....##.##...##......##..##..##..##..##......##.....
..####....####...##...##..##........##....##..##..######..##..##..##..##..######..######.
.........................................................................................
*/

pub struct ComfyKernel {
	pub xcursor_manager: WLRXCursorManager,
	pub cursor_handle: WLRCursorHandle,
	pub keyboard_handle: Option<WLRKeyboardHandle>,
	pub output_layout_handle: WLROutputLayoutHandle,
	pub active_output_name: String,
	pub output_data_map: HashMap<String, OutputData>,
	pub workspace_pool: Vec<Workspace>,
	pub seat_handle: Option<WLRSeatHandle>,
	pub config: Config,
	pub currently_pressed_keys: XkbKeySet,
	pub cursor_direction: LayoutDirection,
}

// TODO: handle main seat features like notifying keyboard/cursor events
impl ComfyKernel {
	pub fn new(
		output_layout_handle: WLROutputLayoutHandle,
		xcursor_manager: WLRXCursorManager,
		cursor_handle: WLRCursorHandle,
	) -> Self {
		ComfyKernel {
			xcursor_manager,
			cursor_handle,
			keyboard_handle: None,
			output_layout_handle: output_layout_handle,
			active_output_name: String::from(""),
			output_data_map: HashMap::<String, OutputData>::new(),
			workspace_pool: vec![],
			seat_handle: None,
			config: Config::load(),
			currently_pressed_keys: XkbKeySet::new(),
			cursor_direction: LayoutDirection::Right,
		}
	}

	/// Schedule a frame of the output which the name match with the provided string
	pub fn schedule_frame_for_output(&self, output_name: &str) {
		let output_layout_handle = self.output_layout_handle.clone();
		with_handles!([(layout: {output_layout_handle})] => {
			let mut found = false;
			layout.outputs().iter_mut().any(|(ref mut output_handle, _)| {
				with_handles!([(output: {output_handle})] => {
					if output.name() == output_name {
						output.schedule_frame();
						found = true;
					}
				}).unwrap();
				found
			});
		}).unwrap();
	}

	pub fn move_cursor_in_active_output(&mut self, direction: LayoutDirection) {
		let mut shell_handle_option = None;
		if let Some(OutputData { workspace, .. }) = self.output_data_map.get_mut(&self.active_output_name) {
			shell_handle_option = workspace
				.window_layout
				.get_shell_handle_relative_to_active_node(&direction);
		} else {
			error!(
				"Failed to get output data for active output: {}",
				self.active_output_name
			);
		}
		if let Some(shell_handle) = shell_handle_option {
			self.apply_keyboard_focus(&shell_handle);
		}
		self.schedule_frame_for_output(&self.active_output_name);
	}

	pub fn set_cursor_direction(&mut self, direction: LayoutDirection) {
		self.cursor_direction = direction;
	}

	pub fn move_active_window(&mut self, direction: LayoutDirection) {
		if let Some(OutputData { workspace, .. }) = self.output_data_map.get_mut(&self.active_output_name) {
			workspace.window_layout.move_active_window(&direction);
		} else {
			error!(
				"Failed to get output data for active output: {}",
				self.active_output_name
			);
		}
		self.schedule_frame_for_output(&self.active_output_name);
	}

	/// Add the provided shell handle as a new window inside the active workspace
	pub fn add_window_to_active_workspace(&mut self, shell_handle: WLRXdgV6ShellSurfaceHandle) {
		let current_cursor_direction = self.cursor_direction.clone();
		if let Some(OutputData { workspace, .. }) = self.output_data_map.get_mut(&self.active_output_name) {
			// TODO: Handle manual direction change for insertion
			match workspace
				.window_layout
				.add_shell_handle(shell_handle, &current_cursor_direction, true, true)
			{
				Err(e) => error!("{}", e),
				Ok(_) => {}
			}
		} else {
			error!(
				"Failed to get output data for active output: {}",
				self.active_output_name
			);
		}

		self.schedule_frame_for_output(&self.active_output_name);
	}

	/// Finds and removes the window bound to the provided shell handle from the containing output.
	pub fn find_and_remove_window(&mut self, shell_handle: WLRXdgV6ShellSurfaceHandle) {
		let mut fallback_shell_handle_option = None;
		let mut name_of_container_output = None;
		for (output_name, output_data) in self.output_data_map.iter_mut() {
			if output_data.workspace.window_layout.contains_shell_handle(&shell_handle) {
				match output_data
					.workspace
					.window_layout
					.remove_window_from_shell_handle(&shell_handle, true)
				{
					Err(e) => error!("{}", e),
					Ok(_) => {
						fallback_shell_handle_option = output_data.workspace.window_layout.get_active_shell_handle();
						name_of_container_output = Some(output_name.clone());
						break;
					}
				}
			}
		}

		// TODO: Should fallback focus only if the containing output is the active one
		if let Some(fallback_shell_handle) = fallback_shell_handle_option {
			self.apply_keyboard_focus(&fallback_shell_handle);
		}
		if let Some(output_name) = name_of_container_output {
			self.schedule_frame_for_output(&output_name)
		}
	}

	/// Sets the shell of the provided shell handle as activated which means it will gain focus.
	pub fn apply_keyboard_focus(&mut self, shell_handle: &WLRXdgV6ShellSurfaceHandle) {
		with_handles!([(seat: {self.seat_handle.clone().unwrap()}), (keyboard: {self.keyboard_handle.clone().unwrap()})] => {
			shell_handle.run(
				|shell| {
					shell.ping();
					let surface = shell.surface();
					surface.run(|surface| {
						if let Some(&mut WLRXdgV6ShellState::TopLevel(ref mut toplevel)) = shell.state() {
							toplevel.set_activated(true);
						}
						seat.set_keyboard(keyboard.input_device());
						seat.keyboard_notify_enter(
							surface,
							&mut keyboard.keycodes(),
							&mut keyboard.get_modifier_masks()
						);
					}).unwrap();
				}
			).unwrap();
			()
		}).unwrap();

		// ? Finds the containing layout to find the containing node and set it as last activated
		for (_output_name, output_data) in self.output_data_map.iter_mut() {
			if output_data.workspace.window_layout.contains_shell_handle(&shell_handle) {
				output_data.workspace.window_layout.set_as_last_activated(&shell_handle);
			}
		}
	}

	pub fn transfer_click_to_seat(&mut self, time: Duration, button: u32, state: u32) {
		with_handles!([(seat: {self.seat_handle.clone().unwrap()})] => {
			seat.pointer_notify_button(time, button, state);
		}).unwrap();
	}

	pub fn transfer_motion_to_seat(&mut self, time: Duration) {
		with_handles!([(cursor: {&self.cursor_handle.clone()}), (seat: {self.seat_handle.clone().unwrap()})] => {
			let (x, y) = cursor.coords();

			if let Some(window) = self.get_window_at_coordinates(x, y) {
				let origin = window.area.origin;
				let surface_x = x - (origin.x as f64);
				let surface_y = y - (origin.y as f64);
				seat.pointer_notify_motion(time, surface_x, surface_y);
			}
		}).unwrap();
	}

	pub fn apply_focus_under_cursor(&mut self) {
		with_handles!([
			(cursor: {&self.cursor_handle.clone()}),
			(seat: {&self.seat_handle.clone().unwrap()}),
			(keyboard: {self.keyboard_handle.clone().unwrap()})
		] => {
			let (x, y) = cursor.coords();

			if let Some(window) = self.get_window_at_coordinates(x, y) {
				let shell_handle = window.shell_handle;
				let origin = window.area.origin;
				let surface_x = x - (origin.x as f64);
				let surface_y = y - (origin.y as f64);
				shell_handle.run(
					|shell| {
						shell.ping();
						let surface = shell.surface();
						surface.run(|surface| {
							if !seat.pointer_surface_has_focus(surface) {
								if let Some(&mut WLRXdgV6ShellState::TopLevel(ref mut toplevel)) = shell.state() {
									toplevel.set_activated(true);
								}
								seat.pointer_notify_enter(surface, surface_x, surface_y);
								seat.set_keyboard(keyboard.input_device());
								seat.keyboard_notify_enter(
									surface,
									&mut keyboard.keycodes(),
									&mut keyboard.get_modifier_masks()
								);
							}
						}).unwrap();
					}
				).unwrap();
			} else {
				seat.pointer_clear_focus();
			}
		}).unwrap();
	}

	pub fn transfer_scroll_to_seat(
		&mut self,
		time: Duration,
		orientation: wlr_axis_orientation,
		value: f64,
		value_discrete: i32,
		source: wlr_axis_source,
	) {
		with_handles!([(seat: {self.seat_handle.clone().unwrap()})] => {
			seat.pointer_notify_axis(time, orientation, value, value_discrete, source);
		}).unwrap();
	}

	fn get_window_at_coordinates(&mut self, x: f64, y: f64) -> Option<Window> {
		let mut window = None;
		for (_, output_data) in self.output_data_map.iter_mut() {
			window = output_data.workspace.window_layout.find_window_at(x, y);
			if window.is_some() {
				break;
			}
		}
		window
	}

	/// Returns the command associated with the provided key_set if any.
	pub fn command_for_keyset(&self, key_set: &XkbKeySet) -> Option<Command> {
		if self.config.keybindings.bindings.contains_key(&key_set) {
			let command = self.config.keybindings.bindings.get(&key_set).unwrap().clone();
			Some(command)
		} else {
			None
		}
	}

	pub fn notify_keyboard(&mut self, key_event: &WLRKeyEvent) {
		let seat_handle = self.seat_handle.clone().unwrap();
		with_handles!([(seat: {seat_handle})] => {
			debug!("Notifying seat of keypress: time_msec: '{:?}' keycode: '{}' key_state: '{}'", key_event.time_msec(), key_event.keycode(), key_event.key_state() as u32);
			seat.keyboard_notify_key(
				key_event.time_msec(),
				key_event.keycode(),
				key_event.key_state() as u32
			);
		}).unwrap();
	}
}

compositor_data!(ComfyKernel);
