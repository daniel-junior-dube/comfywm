/*
..####....####...##...##..##...##...####...##..##..#####..
.##..##..##..##..###.###..###.###..##..##..###.##..##..##.
.##......##..##..##.#.##..##.#.##..######..##.###..##..##.
.##..##..##..##..##...##..##...##..##..##..##..##..##..##.
..####....####...##...##..##...##..##..##..##..##..#####..
..........................................................
.######..##..##..######..######..#####...#####...#####...######..######..######..#####..
...##....###.##....##....##......##..##..##..##..##..##..##........##....##......##..##.
...##....##.###....##....####....#####...#####...#####...####......##....####....#####..
...##....##..##....##....##......##..##..##......##..##..##........##....##......##..##.
.######..##..##....##....######..##..##..##......##..##..######....##....######..##..##.
........................................................................................
*/

use std::process::Command;
use std::thread;

use wlroots::terminate as wlr_terminate;

use common::command_type::CommandType;
use compositor::commands::Command as CompositorCommand;
use compositor::ComfyKernel;
use layout::LayoutDirection;

pub struct CommandInterpreter;
impl CommandInterpreter {
	pub fn execute(command: &CompositorCommand, comfy_kernel: &mut ComfyKernel) {
		info!(
			"Executing command: {:?} with args: {:?}",
			command.command_type, command.args
		);
		match command.command_type {
			CommandType::SetInsertDirectionUp => handle_set_insert_direction_up(command, comfy_kernel),
			CommandType::SetInsertDirectionDown => handle_set_insert_direction_down(command, comfy_kernel),
			CommandType::SetInsertDirectionLeft => handle_set_insert_direction_left(command, comfy_kernel),
			CommandType::SetInsertDirectionRight => handle_set_insert_direction_right(command, comfy_kernel),
			CommandType::MoveActiveFocusUp => handle_move_active_focus_up(command, comfy_kernel),
			CommandType::MoveActiveFocusDown => handle_move_active_focus_down(command, comfy_kernel),
			CommandType::MoveActiveFocusLeft => handle_move_active_focus_left(command, comfy_kernel),
			CommandType::MoveActiveFocusRight => handle_move_active_focus_right(command, comfy_kernel),
			CommandType::MoveActiveWindowUp => handle_move_active_window_up(command, comfy_kernel),
			CommandType::MoveActiveWindowDown => handle_move_active_window_down(command, comfy_kernel),
			CommandType::MoveActiveWindowLeft => handle_move_active_window_left(command, comfy_kernel),
			CommandType::MoveActiveWindowRight => handle_move_active_window_right(command, comfy_kernel),
			CommandType::ToggleActiveWindowFullscreen => handle_toggle_active_window_fullscreen(command, comfy_kernel),
			CommandType::Exec => handle_exec(command, comfy_kernel),
			CommandType::CloseActiveWindow => handle_close_active_window(command, comfy_kernel),
			CommandType::Terminate => handle_terminate(command, comfy_kernel),
		}
	}
}

/*
..####...######..######..######..##..##...####...######..#####...######.
.##......##........##......##....###.##..##......##......##..##....##...
..####...####......##......##....##.###...####...####....#####.....##...
.....##..##........##......##....##..##......##..##......##..##....##...
..####...######....##....######..##..##...####...######..##..##....##...
........................................................................
.#####...######..#####...######...####...######..######...####...##..##.
.##..##....##....##..##..##......##..##....##......##....##..##..###.##.
.##..##....##....#####...####....##........##......##....##..##..##.###.
.##..##....##....##..##..##......##..##....##......##....##..##..##..##.
.#####...######..##..##..######...####.....##....######...####...##..##.
........................................................................
*/

fn handle_set_insert_direction_up(_: &CompositorCommand, comfy_kernel: &mut ComfyKernel) {
	comfy_kernel.set_cursor_direction(LayoutDirection::Up);
}

fn handle_set_insert_direction_down(_: &CompositorCommand, comfy_kernel: &mut ComfyKernel) {
	comfy_kernel.set_cursor_direction(LayoutDirection::Down);
}

fn handle_set_insert_direction_left(_: &CompositorCommand, comfy_kernel: &mut ComfyKernel) {
	comfy_kernel.set_cursor_direction(LayoutDirection::Left);
}

fn handle_set_insert_direction_right(_: &CompositorCommand, comfy_kernel: &mut ComfyKernel) {
	comfy_kernel.set_cursor_direction(LayoutDirection::Right);
}

/*
.##...##...####...##..##..######...####....####...######..######..##..##..######.
.###.###..##..##..##..##..##......##..##..##..##....##......##....##..##..##.....
.##.#.##..##..##..##..##..####....######..##........##......##....##..##..####...
.##...##..##..##...####...##......##..##..##..##....##......##.....####...##.....
.##...##...####.....##....######..##..##...####.....##....######....##....######.
.................................................................................
.##...##..######..##..##..#####....####...##...##.
.##...##....##....###.##..##..##..##..##..##...##.
.##.#.##....##....##.###..##..##..##..##..##.#.##.
.#######....##....##..##..##..##..##..##..#######.
..##.##...######..##..##..#####....####....##.##..
..................................................
*/

fn handle_move_active_focus_up(_: &CompositorCommand, comfy_kernel: &mut ComfyKernel) {
	comfy_kernel.move_cursor_in_active_output(LayoutDirection::Up);
}

fn handle_move_active_focus_down(_: &CompositorCommand, comfy_kernel: &mut ComfyKernel) {
	comfy_kernel.move_cursor_in_active_output(LayoutDirection::Down);
}

fn handle_move_active_focus_left(_: &CompositorCommand, comfy_kernel: &mut ComfyKernel) {
	comfy_kernel.move_cursor_in_active_output(LayoutDirection::Left);
}

fn handle_move_active_focus_right(_: &CompositorCommand, comfy_kernel: &mut ComfyKernel) {
	comfy_kernel.move_cursor_in_active_output(LayoutDirection::Right);
}

/*
.##...##...####...##..##..######...####....####...######..######..##..##..######.
.###.###..##..##..##..##..##......##..##..##..##....##......##....##..##..##.....
.##.#.##..##..##..##..##..####....######..##........##......##....##..##..####...
.##...##..##..##...####...##......##..##..##..##....##......##.....####...##.....
.##...##...####.....##....######..##..##...####.....##....######....##....######.
.................................................................................
.##...##..######..##..##..#####....####...##...##.
.##...##....##....###.##..##..##..##..##..##...##.
.##.#.##....##....##.###..##..##..##..##..##.#.##.
.#######....##....##..##..##..##..##..##..#######.
..##.##...######..##..##..#####....####....##.##..
..................................................
*/

fn handle_move_active_window_up(_: &CompositorCommand, comfy_kernel: &mut ComfyKernel) {
	comfy_kernel.move_active_window(LayoutDirection::Up);
}

fn handle_move_active_window_down(_: &CompositorCommand, comfy_kernel: &mut ComfyKernel) {
	comfy_kernel.move_active_window(LayoutDirection::Down);
}

fn handle_move_active_window_left(_: &CompositorCommand, comfy_kernel: &mut ComfyKernel) {
	comfy_kernel.move_active_window(LayoutDirection::Left);
}

fn handle_move_active_window_right(_: &CompositorCommand, comfy_kernel: &mut ComfyKernel) {
	comfy_kernel.move_active_window(LayoutDirection::Right);
}

/*
.######...####....####....####...##......######.
...##....##..##..##......##......##......##.....
...##....##..##..##.###..##.###..##......####...
...##....##..##..##..##..##..##..##......##.....
...##.....####....####....####...######..######.
................................................
.######..##..##..##......##.......####....####...#####...######..######..##..##.
.##......##..##..##......##......##......##..##..##..##..##......##......###.##.
.####....##..##..##......##.......####...##......#####...####....####....##.###.
.##......##..##..##......##..........##..##..##..##..##..##......##......##..##.
.##.......####...######..######...####....####...##..##..######..######..##..##.
................................................................................
*/

fn handle_toggle_active_window_fullscreen(_: &CompositorCommand, comfy_kernel: &mut ComfyKernel) {
	comfy_kernel.toggle_active_window_fullscreen();
}

/*
.######..##..##..######...####..
.##.......####...##......##..##.
.####......##....####....##.....
.##.......####...##......##..##.
.######..##..##..######...####..
................................
*/

fn handle_exec(command: &CompositorCommand, _: &mut ComfyKernel) {
	let command_clone = command.clone();
	let nb_of_arguments = command_clone.args.len();
	if nb_of_arguments == 0 {
		error!("Tried to execute an 'Exec' command without providing any arguments!");
	} else {
		thread::spawn(move || {
			let executable = &command_clone.args[0];
			let mut process_command = Command::new(executable);
			if nb_of_arguments > 1 {
				process_command.args(&command_clone.args[1..nb_of_arguments - 1]);
			}

			match process_command.output() {
				Ok(output) => info!(
					"The command {} returned {}",
					command_clone.args.join(" "),
					output.status
				),
				Err(e) => error!("The command {} failed with: {}", command_clone.args.join(" "), e),
			};
		});
	}
}

/*
..####...##.......####....####...######...........####....####...######..######..##..##..######.
.##..##..##......##..##..##......##..............##..##..##..##....##......##....##..##..##.....
.##......##......##..##...####...####............######..##........##......##....##..##..####...
.##..##..##......##..##......##..##..............##..##..##..##....##......##.....####...##.....
..####...######...####....####...######..........##..##...####.....##....######....##....######.
................................................................................................
.##...##..######..##..##..#####....####...##...##.
.##...##....##....###.##..##..##..##..##..##...##.
.##.#.##....##....##.###..##..##..##..##..##.#.##.
.#######....##....##..##..##..##..##..##..#######.
..##.##...######..##..##..#####....####....##.##..
..................................................
*/

fn handle_close_active_window(_: &CompositorCommand, comfy_kernel: &mut ComfyKernel) {
	if let Some(window) = comfy_kernel.get_active_window() {
		window.close();
	}
}

/*
.######..######..#####...##...##..######..##..##...####...######..######.
...##....##......##..##..###.###....##....###.##..##..##....##....##.....
...##....####....#####...##.#.##....##....##.###..######....##....####...
...##....##......##..##..##...##....##....##..##..##..##....##....##.....
...##....######..##..##..##...##..######..##..##..##..##....##....######.
.........................................................................
*/

fn handle_terminate(_: &CompositorCommand, _: &mut ComfyKernel) {
	info!("Goodbye!");
	wlr_terminate();
}
