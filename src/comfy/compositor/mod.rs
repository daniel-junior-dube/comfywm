use std::path::Path;
use std::{collections::HashMap, time::Duration};

use wlroots::key_events::KeyEvent as WLRKeyEvent;
use wlroots::pointer_events::AbsoluteMotionEvent;
use wlroots::{
	Output as WLROutput, Area, Origin, Size,
	Capability, Compositor as WLRCompositor, CompositorBuilder as WLRCompositorBuilder, Cursor as WLRCursor,
	CursorHandle as WLRCursorHandle, KeyboardHandle as WLRKeyboardHandle, OutputLayout as WLROutputLayout,
	OutputHandle as WLROutputHandle, OutputLayoutHandle as WLROutputLayoutHandle, Seat as WLRSeat, SeatHandle as WLRSeatHandle,
	Texture, XCursorManager as WLRXCursorManager,
	XdgV6ShellState as WLRXdgV6ShellState, XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle,
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
use utils::graphics::texture_helper;
use utils::handle_helper::output_helper;

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

	// ? Load wallpaper
	// TODO: Make a method to dynamically reload a wallpaper (live reload wallpaper)
	{
		let mut gles2 = &mut compositor.renderer.as_mut().unwrap();
		let comfy_kernel: &mut ComfyKernel = (&mut compositor.data).downcast_mut().unwrap();
		match texture_helper::load_texture(
			&mut gles2,
			&Path::new(&comfy_kernel.config.theme.wallpaper_path.clone()),
		) {
			Ok(wallpaper_texture) => comfy_kernel.wallpaper_texture = Some(wallpaper_texture),
			Err(e) => error!("{}", e),
		}
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
	pub wallpaper_texture: Option<Texture<'static>>,
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
			wallpaper_texture: None,
		}
	}

	/// Returns true if the provided surface is a top level surface.
	#[wlroots_dehandle(output_layout)]
	fn generate_output_area(&self, output: &mut WLROutput) -> Area {
		let output_layout_handle = &self.output_layout_handle;
		use output_layout_handle as output_layout;
		let mut x: f64 = 0.0;
		let mut y: f64 = 0.0;
		output_layout.output_coords(output, &mut x, &mut y);
		let (width, height) = output.effective_resolution();
		Area::new(Origin::new(x as i32, y as i32), Size::new(width, height))
	}

	#[wlroots_dehandle(output, cursor, output_layout)]
	pub fn add_output(&mut self, output_handle: &WLROutputHandle, set_as_active_output: bool) {
		use output_handle as output;

		let output_name = output.name();
		info!("New output detected, named: {}", output_name);

		if set_as_active_output {
			self.active_output_name = output_name.clone();
		}
		{
			let xcursor_manager = &mut self.xcursor_manager;
			// TODO use output config if present instead of auto
			let output_layout_handle = &mut self.output_layout_handle;
			let cursor_handle = &mut self.cursor_handle;

			use cursor_handle as cursor;
			{
				use output_layout_handle as output_layout;
				output_layout.add_auto(output);
				cursor.attach_output_layout(output_layout);
			}
			xcursor_manager.load(output.scale());
			xcursor_manager.set_cursor_image("left_ptr".to_string(), cursor);
			let (x, y) = cursor.coords();
			// https://en.wikipedia.org/wiki/Mouse_warping
			cursor.warp(None, x, y);
		}

		let output_area = self.generate_output_area(output);
		let output_data_map = &mut self.output_data_map;
		debug!("New output_area: {:?}", output_area);
		output_data_map.insert(output.name(), OutputData::new(output_area));
	}

	/// Schedule a frame of the output which the name match with the provided string
	#[wlroots_dehandle(layout, output)]
	pub fn schedule_frame_for_output(&self, output_name: &str) {
		let output_layout_handle = self.output_layout_handle.clone();
		use output_layout_handle as layout;
		let mut found = false;
		layout.outputs().iter_mut().any(|(ref mut output_handle, _)| {
			use output_handle as output;
			if output.name() == output_name {
				output.schedule_frame();
				found = true;
			}
			found
		});
	}

	/// Sets or unsets the fullscreen active window.
	pub fn toggle_active_window_fullscreen(&mut self) {
		if let Some(OutputData { workspace, .. }) = self.output_data_map.get_mut(&self.active_output_name) {
			workspace.window_layout.toggle_active_window_fullscreen();
		}
	}

	/// Move the 'cursor' in the layout in a given direction.
	pub fn move_cursor_in_active_output(&mut self, direction: LayoutDirection) {
		let mut shell_handle_option = None;
		if let Some(OutputData { workspace, .. }) = self.output_data_map.get_mut(&self.active_output_name) {
			if !workspace.window_layout.has_fullscreen_window() {
				shell_handle_option = workspace
					.window_layout
					.get_shell_handle_relative_to_active_node(&direction);
			}
		} else {
			error!(
				"Failed to get output data for active output: {}",
				self.active_output_name
			);
		}
		if let Some(shell_handle) = shell_handle_option {
			self.apply_keyboard_focus(&shell_handle);
			self.schedule_frame_for_output(&self.active_output_name);
		}
	}

	/// Sets the direction of the 'cursor' of the layout.
	pub fn set_cursor_direction(&mut self, direction: LayoutDirection) {
		self.cursor_direction = direction;
	}

	pub fn get_active_window(&mut self) -> Option<Window> {
		if let Some(OutputData { workspace, .. }) = self.output_data_map.get_mut(&self.active_output_name) {
			return workspace.window_layout.get_active_window();
		}
		None
	}

	/// Moves the active window in the active layout in a given direction.
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
		let mut active_shell_option = None;
		if let Some(OutputData { workspace, .. }) = self.output_data_map.get_mut(&self.active_output_name) {
			// TODO: Handle manual direction change for insertion
			active_shell_option = match workspace.window_layout.add_shell_handle(
				shell_handle,
				&current_cursor_direction,
				self.config.theme.border_size,
				true,
				true,
			) {
				Err(e) => {
					error!("{}", e);
					None
				}
				Ok(_) => workspace.window_layout.get_active_shell_handle(),
			}
		} else {
			error!(
				"Failed to get output data for active output: {}",
				self.active_output_name
			);
		}

		if let Some(active_shell) = active_shell_option {
			self.apply_keyboard_focus(&active_shell);
			self.schedule_frame_for_output(&self.active_output_name);
		}
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
		if let Some(output_name) = name_of_container_output {
			if self.is_active_output(&output_name) {
				if let Some(fallback_shell_handle) = fallback_shell_handle_option {
					self.apply_keyboard_focus(&fallback_shell_handle);
				}
			}
			self.schedule_frame_for_output(&output_name)
		}
	}

	/// Sets the shell of the provided shell handle as activated which means it will gain focus.
	#[wlroots_dehandle(seat, keyboard, shell, surface)]
	pub fn apply_keyboard_focus(&mut self, shell_handle: &WLRXdgV6ShellSurfaceHandle) {
		let seat_handle = self.seat_handle.clone().unwrap();
		let keyboard_handle = self.keyboard_handle.clone().unwrap();

		use keyboard_handle as keyboard;
		use seat_handle as seat;
		use shell_handle as shell;

		shell.ping();

		let surface_handle = shell.surface();
		use surface_handle as surface;

		if let Some(&mut WLRXdgV6ShellState::TopLevel(ref mut toplevel)) = shell.state() {
			toplevel.set_activated(true);
		}
		seat.set_keyboard(keyboard.input_device());
		seat.keyboard_notify_enter(surface, &mut keyboard.keycodes(), &mut keyboard.get_modifier_masks());

		// ? Finds the containing layout to find the containing node and set it as last activated
		for (_output_name, output_data) in self.output_data_map.iter_mut() {
			if output_data.workspace.window_layout.contains_shell_handle(&shell_handle) {
				output_data.workspace.window_layout.set_as_last_activated(&shell_handle);
			}
		}
	}

	/// Returns the name of the output and output-related coordinates at the intersects with the given position.
	fn get_output_intersection_at(&self, x: f64, y: f64) -> Option<(String, f64, f64)> {
		let mut output_name_option = None;
		// ? Finds the containing layout to find the containing node and set it as last activated
		for (output_name, output_data) in self.output_data_map.iter() {
			if let Some((output_x, output_y)) = output_data.workspace.window_layout.intersection_at(x, y) {
				output_name_option = Some((output_name.clone(), output_x, output_y));
				break;
			}
		}
		output_name_option
	}

	/// Returns the coordinates of the cursor location relative to the currently active output.
	#[wlroots_dehandle(cursor)]
	pub fn get_cursor_coordinates(&self) -> (f64, f64) {
		let cursor_handle = &self.cursor_handle;
		use cursor_handle as cursor;
		cursor.coords()
	}

	fn is_active_output(&self, output_name: &str) -> bool {
		output_name == self.active_output_name
	}

	fn set_output_under_cursor_as_active(&mut self) {
		let (cursor_x, cursor_y) = self.get_cursor_coordinates();
		if let Some((output_name, _output_x, _output_y)) = self.get_output_intersection_at(cursor_x, cursor_y) {
			if !self.is_active_output(&output_name) {
				self.active_output_name = output_name.clone();
				debug!("New active output: {}", self.active_output_name);
			}
		}
	}

	#[wlroots_dehandle(seat, subsurface)]
	pub fn apply_focus_under_cursor(&mut self) {
		debug!("apply_focus_under_cursor");
	        let seat_handle = self.seat_handle.clone().unwrap();
		self.set_output_under_cursor_as_active();
		debug!("apply_focus_under_cursor active output: {}", self.active_output_name);
	        let seat_handle = self.seat_handle.clone().unwrap();
		let mut shell_handle_option = None;
		self.output_data_map.get(&self.active_output_name).map(|output_data| {
			let (cursor_x, cursor_y) = self.get_cursor_coordinates();
			debug!("apply_focus_under_cursor cursor pos: {} {}", cursor_x, cursor_y);
			if let Some((window, subsurface_handle, surface_x, surface_y)) = output_data.get_window_and_subsurface_at(cursor_x, cursor_y) {
				use seat_handle as seat;
				use subsurface_handle as subsurface;
				if !seat.pointer_surface_has_focus(subsurface) {
					seat.pointer_notify_enter(subsurface, surface_x, surface_y);
					shell_handle_option = Some(window.shell_handle.clone());
				}
			} else {
				use seat_handle as seat;
				seat.pointer_clear_focus();
			}
		});
		if let Some(shell_handle) = shell_handle_option {
			self.apply_keyboard_focus(&shell_handle);
		}
	}

	#[wlroots_dehandle(seat)]
	pub fn transfer_click_to_seat(&mut self, time: Duration, button: u32, state: u32) {
		if let Some(ref seat_handle) = self.seat_handle {
			use seat_handle as seat;
			seat.pointer_notify_button(time, button, state);
		}
	}

	#[wlroots_dehandle(seat)]
	pub fn transfer_motion_to_seat(&mut self, time: Duration) {
		if let Some(ref seat_handle) = self.seat_handle.clone() {
			use seat_handle as seat;
			let (cursor_x, cursor_y) = self.get_cursor_coordinates();
			if let Some((output_name, output_x, output_y)) = self.get_output_intersection_at(cursor_x, cursor_y) {
				let output_data = self.output_data_map.get(&output_name).unwrap();
				if let Some((_window, _subsurface_handle, surface_x, surface_y)) = output_data.get_window_and_subsurface_at(cursor_x, cursor_y) {
					seat.pointer_notify_motion(time, surface_x, surface_y);
				}
			}
		}
	}

	#[wlroots_dehandle(seat)]
	pub fn transfer_scroll_to_seat(
		&mut self,
		time: Duration,
		orientation: wlr_axis_orientation,
		value: f64,
		value_discrete: i32,
		source: wlr_axis_source,
	) {
		if let Some(ref seat_handle) = self.seat_handle {
			use seat_handle as seat;
			seat.pointer_notify_axis(time, orientation, value, value_discrete, source);
		}
	}

	#[wlroots_dehandle(cursor)]
	pub fn warp_cursor(&self, event: &AbsoluteMotionEvent) {
		let cursor_handle = &self.cursor_handle;
		use cursor_handle as cursor;

		let (absolute_x, absolute_y) = event.pos();
		cursor.warp_absolute(event.device(), absolute_x, absolute_y);
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

	#[wlroots_dehandle(seat)]
	pub fn notify_keyboard(&mut self, key_event: &WLRKeyEvent) {
		let seat_handle = self.seat_handle.clone().unwrap();
		use seat_handle as seat;

		debug!(
			"Notifying seat of keypress: time_msec: '{:?}' keycode: '{}' key_state: '{}'",
			key_event.time_msec(),
			key_event.keycode(),
			key_event.key_state() as u32
		);
		seat.keyboard_notify_key(key_event.time_msec(), key_event.keycode(), key_event.key_state() as u32);
	}
}

compositor_data!(ComfyKernel);
