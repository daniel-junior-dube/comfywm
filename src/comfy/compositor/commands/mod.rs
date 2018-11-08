use std::str::FromStr;

pub mod interpreter;

use common::command_type::CommandType;

/*
..####....####...##...##..##...##...####...##..##..#####....####..
.##..##..##..##..###.###..###.###..##..##..###.##..##..##..##.....
.##......##..##..##.#.##..##.#.##..######..##.###..##..##...####..
.##..##..##..##..##...##..##...##..##..##..##..##..##..##......##.
..####....####...##...##..##...##..##..##..##..##..#####....####..
..................................................................
*/

#[derive(Clone)]
pub struct Command {
	pub command_type: CommandType,
	pub args: Vec<String>,
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

	pub fn from_str(command_str: &str) -> Result<Command, String> {
		let mut splitted_str: Vec<&str> = command_str.split(" ").collect();

		if splitted_str.len() == 0 {
			return Err("The command is empty.".to_string());
		}

		let command_type = CommandType::from_str(splitted_str[0]).unwrap();

		let args_str = splitted_str
			.split_off(1)
			.iter()
			.map(|elem| String::from(*elem))
			.collect();

		Ok(Command::new_with_args(command_type, args_str))
	}
}
