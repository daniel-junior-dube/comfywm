/*
..####....####...##...##..##...##...####...##..##..#####...######..##..##..#####...######.
.##..##..##..##..###.###..###.###..##..##..###.##..##..##....##.....####...##..##..##.....
.##......##..##..##.#.##..##.#.##..######..##.###..##..##....##......##....#####...####...
.##..##..##..##..##...##..##...##..##..##..##..##..##..##....##......##....##......##.....
..####....####...##...##..##...##..##..##..##..##..#####.....##......##....##......######.
..........................................................................................
*/

#[derive(Clone, Debug, PartialEq, Eq, ToString, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum CommandType {
	Terminate,
	Exec,
	SetInsertDirectionUp,
	SetInsertDirectionDown,
	SetInsertDirectionLeft,
	SetInsertDirectionRight,
	MoveActiveFocusUp,
	MoveActiveFocusDown,
	MoveActiveFocusLeft,
	MoveActiveFocusRight,
	MoveActiveWindowUp,
	MoveActiveWindowDown,
	MoveActiveWindowLeft,
	MoveActiveWindowRight,
	PutActiveWindowToStack,
	PopWindowFromStack,
	ToggleActiveWindowFullscreen,
	ReloadConfig,
	CloseActiveWindow,
}
