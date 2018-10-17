/*
..####....####...##...##..##...##...####...##..##..#####...######..##..##..#####...######.
.##..##..##..##..###.###..###.###..##..##..###.##..##..##....##.....####...##..##..##.....
.##......##..##..##.#.##..##.#.##..######..##.###..##..##....##......##....#####...####...
.##..##..##..##..##...##..##...##..##..##..##..##..##..##....##......##....##......##.....
..####....####...##...##..##...##..##..##..##..##..#####.....##......##....##......######.
..........................................................................................
*/

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandType {
	Terminate,
	Exec,
	MoveActiveWindowUp,
	MoveActiveWindowDown,
	MoveActiveWindowLeft,
	MoveActiveWindowRight,
	MoveFocusToNextWindow,
	MoveFocusToPreviousWindow,
}

impl CommandType {
	pub fn from_str(value: &str) -> Result<CommandType, String> {
		match value {
			"terminate" => Ok(CommandType::Terminate),
			"exec" => Ok(CommandType::Exec),
			"move_active_window_up" => Ok(CommandType::MoveActiveWindowUp),
			"move_active_window_down" => Ok(CommandType::MoveActiveWindowDown),
			"move_active_window_left" => Ok(CommandType::MoveActiveWindowLeft),
			"move_active_window_right" => Ok(CommandType::MoveActiveWindowRight),
			"move_focus_to_next_window" => Ok(CommandType::MoveFocusToNextWindow),
			"move_focus_to_previous_window" => Ok(CommandType::MoveFocusToPreviousWindow),
			_ => Err(format!("The command type {} is invalid.", value)),
		}
	}

	pub fn to_str(&self) -> &'static str {
		match self {
			CommandType::Terminate => "terminate",
			CommandType::Exec => "exec",
			CommandType::MoveActiveWindowUp => "move_active_window_up",
			CommandType::MoveActiveWindowDown => "move_active_window_down",
			CommandType::MoveActiveWindowLeft => "move_active_window_left",
			CommandType::MoveActiveWindowRight => "move_active_window_right",
			CommandType::MoveFocusToNextWindow => "move_focus_to_next_window",
			CommandType::MoveFocusToPreviousWindow => "move_focus_to_previous_window",
		}
	}
}
