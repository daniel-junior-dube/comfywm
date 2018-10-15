use std::collections::HashMap;

use wlroots::xkbcommon::xkb::keysyms;
use wlroots::{
	Capability, Compositor as WLRCompositor, CompositorBuilder as WLRCompositorBuilder, Cursor as WLRCursor,
	CursorHandle as WLRCursorHandle, KeyboardHandle as WLRKeyboardHandle, OutputLayout as WLROutputLayout,
	OutputLayoutHandle as WLROutputLayoutHandle, Seat as WLRSeat, SeatHandle as WLRSeatHandle,
	XCursorManager as WLRXCursorManager, XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle, XdgV6ShellState as WLRXdgV6ShellState
};

use wlroots::wlroots_sys::protocols::server_decoration::server::org_kde_kwin_server_decoration_manager::Mode as ServerDecorationMode;

pub mod commands;
pub mod output;
pub mod shell;
pub mod surface;
pub mod workspace;

use self::commands::Command;
use self::output::{OutputData, OutputLayoutHandler, OutputManagerHandler};
use self::shell::XdgV6ShellManagerHandler;
use self::workspace::Workspace;
use common::command_type::CommandType;
use input::cursor::CursorHandler;
use input::keyboard::XkbKeySet;
use input::seat::SeatHandler;
use input::InputManagerHandler;

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
.##...##...####...#####...######.
.###.###..##..##..##..##..##.....
.##.#.##..##..##..##..##..####...
.##...##..##..##..##..##..##.....
.##...##...####...#####...######.
.................................
*/

pub struct SuperModeState {
	pub detailed_mode_is_enabled: bool,
	pub xkb_key_set: XkbKeySet,
}

impl SuperModeState {
	pub fn new() -> Self {
		SuperModeState {
			detailed_mode_is_enabled: false,
			xkb_key_set: XkbKeySet::new(),
		}
	}
}

pub enum CompositorMode {
	NormalMode,
	SuperMode(SuperModeState),
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
	pub current_mode: CompositorMode,
	pub super_mode_xkb_key: u32,
	pub available_commands: HashMap<XkbKeySet, Command>,
}

impl ComfyKernel {
	pub fn new(
		output_layout_handle: WLROutputLayoutHandle,
		xcursor_manager: WLRXCursorManager,
		cursor_handle: WLRCursorHandle,
	) -> Self {
		let mut available_commands = HashMap::<XkbKeySet, Command>::new();

		available_commands.insert(
			XkbKeySet::from_string("1".to_string()).unwrap(),
			Command::new_with_args(CommandType::Exec, vec!["weston-terminal".to_string()]),
		);

		available_commands.insert(
			XkbKeySet::from_string("2".to_string()).unwrap(),
			Command::new_with_args(CommandType::Exec, vec!["gnome-terminal".to_string()]),
		);

		available_commands.insert(
			XkbKeySet::from_string("3".to_string()).unwrap(),
			Command::new_with_args(CommandType::Exec, vec!["firefox".to_string()]),
		);

		available_commands.insert(
			XkbKeySet::from_string("Escape".to_string()).unwrap(),
			Command::new(CommandType::Terminate),
		);

		available_commands.insert(
			XkbKeySet::from_string("w".to_string()).unwrap(),
			Command::new(CommandType::MoveActiveWindowUp),
		);

		available_commands.insert(
			XkbKeySet::from_string("s".to_string()).unwrap(),
			Command::new(CommandType::MoveActiveWindowDown),
		);

		available_commands.insert(
			XkbKeySet::from_string("a".to_string()).unwrap(),
			Command::new(CommandType::MoveActiveWindowLeft),
		);

		available_commands.insert(
			XkbKeySet::from_string("d".to_string()).unwrap(),
			Command::new(CommandType::MoveActiveWindowRight),
		);

		ComfyKernel {
			xcursor_manager,
			cursor_handle,
			keyboard_handle: None,
			output_layout_handle: output_layout_handle,
			active_output_name: String::from(""),
			output_data_map: HashMap::<String, OutputData>::new(),
			workspace_pool: vec![],
			seat_handle: None,
			current_mode: CompositorMode::NormalMode,
			super_mode_xkb_key: keysyms::KEY_Control_R,
			available_commands: available_commands,
		}
	}

	pub fn add_window_to_active_workspace(&mut self, shell_handle: WLRXdgV6ShellSurfaceHandle) {
		let active_output_name = self.active_output_name.clone();
		let output_layout_handle = self.output_layout_handle.clone();

		with_handles!([(layout: {output_layout_handle})] => {
			if let Some(OutputData {workspace, ..}) = self.output_data_map.get_mut(&active_output_name) {
				workspace.add_window(shell_handle);
			}

			for (mut output_handle, _) in layout.outputs() {
				with_handles!([(output: {output_handle})] => {
					output.schedule_frame()
				}).unwrap();
			}
		}).unwrap();
	}

	pub fn find_and_remove_window(&mut self, shell_handle: WLRXdgV6ShellSurfaceHandle) {
		let output_layout_handle = self.output_layout_handle.clone();

		with_handles!([(_output_layout: {output_layout_handle})] => {
			// TODO: Use output_layout to find output name, which can be used to get the appropriate output
			let mut fallback_shell_handle_option = None;
			for output_data in self.output_data_map.values_mut() {
				if output_data.workspace.contains_window(&shell_handle) {
					fallback_shell_handle_option = output_data.workspace.remove_window(&shell_handle);
				}
			}
			if let Some(fallback_shell_handle) = fallback_shell_handle_option {
				self.set_activated(&fallback_shell_handle);
			}
			/*
			for (mut output_handle, _) in layout.outputs() {
				with_handles!([(output: {output_handle})] => {
					output.schedule_frame()
				}).unwrap();
			} */
		}).unwrap();
	}

	pub fn set_activated(&mut self, shell_handle: &WLRXdgV6ShellSurfaceHandle) {
		dehandle!(
			let seat_handle = self.seat_handle.clone().unwrap();
			let keyboard_handle = self.keyboard_handle.clone().unwrap();
			@seat = {seat_handle};
			@keyboard = {keyboard_handle};
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
		);
	}
}

compositor_data!(ComfyKernel);
