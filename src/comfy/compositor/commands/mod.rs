/*
..####....####...##...##..##...##...####...##..##..#####....####..
.##..##..##..##..###.###..###.###..##..##..###.##..##..##..##.....
.##......##..##..##.#.##..##.#.##..######..##.###..##..##...####..
.##..##..##..##..##...##..##...##..##..##..##..##..##..##......##.
..####....####...##...##..##...##..##..##..##..##..#####....####..
..................................................................
*/

pub mod interpreter;

use common::command_type::CommandType;

#[derive(Clone)]
pub struct Command {
	command_type: CommandType,
	args: Vec<String>,
}

impl Command {
	pub fn new(command_type: CommandType) -> Self {
		Command {
			command_type: command_type,
			args: vec![],
		}
	}

	pub fn new_with_args(command_type: CommandType, args: Vec<String>) -> Self {
		Command {
			command_type: command_type,
			args: args,
		}
	}
}
