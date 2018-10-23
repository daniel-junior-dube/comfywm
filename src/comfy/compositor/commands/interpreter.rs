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

pub struct CommandInterpreter;
impl CommandInterpreter {
	pub fn execute(command: &CompositorCommand, comfy_kernel: &mut ComfyKernel) {
		println!(
			"Executing command: {:?} with args: {:?}",
			command.command_type, command.args
		);
		match command.command_type {
			CommandType::MoveActiveWindowUp => {
				handle_move_active_window_up(command, comfy_kernel);
			}
			CommandType::MoveActiveWindowDown => {
				handle_move_active_window_down(command, comfy_kernel);
			}
			CommandType::MoveActiveWindowLeft => {
				handle_move_active_window_left(command, comfy_kernel);
			}
			CommandType::MoveActiveWindowRight => {
				handle_move_active_window_right(command, comfy_kernel);
			}
			CommandType::Exec => {
				handle_exec(command, comfy_kernel);
			}
			CommandType::Terminate => {
				handle_terminate(command, comfy_kernel);
			}
			_ => {
				error!("Command type unknown!");
			}
		}
	}
}

fn handle_move_active_window_up(_: &CompositorCommand, _: &mut ComfyKernel) {
	println!("handle_move_active_window_up");
}

fn handle_move_active_window_down(_: &CompositorCommand, _: &mut ComfyKernel) {
	println!("handle_move_active_window_down");
}

fn handle_move_active_window_left(_: &CompositorCommand, _: &mut ComfyKernel) {
	println!("handle_move_active_window_left");
}

fn handle_move_active_window_right(_: &CompositorCommand, _: &mut ComfyKernel) {
	println!("handle_move_active_window_right");
}

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
			let output = process_command.output().unwrap();
			println!("Output of {} : {:?}", executable, output);
		});
	}
}

fn handle_terminate(_: &CompositorCommand, _: &mut ComfyKernel) {
	info!("Goodbye!");
	wlr_terminate();
}
