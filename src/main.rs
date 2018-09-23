/*

MM'""""'YMM MMP"""""YMM M"""""`'"""`YM MM""""""""`M M""MMMM""M M""MMM""MMM""M M"""""`'"""`YM
M' .mmm. `M M' .mmm. `M M  mm.  mm.  M MM  mmmmmmmM M. `MM' .M M  MMM  MMM  M M  mm.  mm.  M
M  MMMMMooM M  MMMMM  M M  MMM  MMM  M M'      MMMM MM.    .MM M  MMP  MMP  M M  MMM  MMM  M
M  MMMMMMMM M  MMMMM  M M  MMM  MMM  M MM  MMMMMMMM MMMb  dMMM M  MM'  MM' .M M  MMM  MMM  M
M. `MMM' .M M. `MMM' .M M  MMM  MMM  M MM  MMMMMMMM MMMM  MMMM M  `' . '' .MM M  MMM  MMM  M
MM.     .dM MMb     dMM M  MMM  MMM  M MM  MMMMMMMM MMMM  MMMM M    .d  .dMMM M  MMM  MMM  M
MMMMMMMMMMM MMMMMMMMMMM MMMMMMMMMMMMMM MMMMMMMMMMMM MMMMMMMMMM MMMMMMMMMMMMMM MMMMMMMMMMMMMM

	Authors:
		- Daniel-Junior Dubé
		- Félix Chabot
	Date:
		September 2018
*/

#[macro_use]
extern crate wlroots;

#[allow(unused_imports)]
use libc;

use wlroots::key_events::KeyEvent as WLRKeyEvent;
use wlroots::utils::{
	current_time,
	init_logging as wlr_init_logging,
	WLR_DEBUG,
};
use wlroots::xkbcommon::xkb::keysyms;
use wlroots::{
	project_box,
	Area,
	Capability,
	CompositorBuilder as WLRCompositorBuilder,
	CompositorHandle as WLRCompositorHandle,
	InputManagerHandler as WLRInputManagerHandler,
	KeyboardHandle as WLRKeyboardHandle,
	KeyboardHandler as WLRKeyboardHandler,
	Origin,
	OutputBuilder as WLROutputBuilder,
	OutputBuilderResult as WLROutputBuilderResult,
	OutputHandle as WLROutputHandle,
	OutputHandler as WLROutputHandler,
	OutputManagerHandler as WLROutputManagerHandler,
	Renderer,
	Seat,
	SeatHandle as WLRSeatHandle,
	SeatHandler as WLRSeatHandler,
	Size,
	SurfaceHandle as WLRSurfaceHandle,
	SurfaceHandler as WLRSurfaceHandler,
	XdgV6ShellHandler as WLRXdgV6ShellHandler,
	XdgV6ShellManagerHandler as WLRXdgV6ShellManagerHandler,
	XdgV6ShellState::TopLevel as WLRTopLevel,
	XdgV6ShellSurfaceHandle as WLRXdgV6ShellSurfaceHandle,
};

/*
..####...######...####...######..######.
.##........##....##..##....##....##.....
..####.....##....######....##....####...
.....##....##....##..##....##....##.....
..####.....##....##..##....##....######.
........................................
*/

struct State {
	keyboard: Option<WLRKeyboardHandle>,
	shells: Vec<WLRXdgV6ShellSurfaceHandle>,
	seat_handle: Option<WLRSeatHandle>,
	current_rotation: f32,
}

impl State {
	fn new() -> Self {
		State {
			keyboard: None,
			shells: vec![],
			seat_handle: None,
			current_rotation: 0.0,
		}
	}
}

compositor_data!(State);

/*
..####...##..##..######..#####...##..##..######.
.##..##..##..##....##....##..##..##..##....##...
.##..##..##..##....##....#####...##..##....##...
.##..##..##..##....##....##......##..##....##...
..####....####.....##....##.......####.....##...
................................................
*/

struct Output;
impl WLROutputHandler for Output {
	fn on_frame(&mut self, compositor: WLRCompositorHandle, output: WLROutputHandle) {
		dehandle!(
			@compositor = {compositor};
			@output = {output};
			let state: &mut State = compositor.data.downcast_mut().unwrap();
			let renderer = compositor.renderer
				.as_mut()
				.expect("Compositor was not loaded with a renderer");
			let mut render_context = renderer.render(output, None);
			render_context.clear([135.0/255.0, 7.0/255.0, 52.0/255.0, 1.0]);
			render_shells(state, &mut render_context)
		);
	}
}

struct OutputManager;
impl WLROutputManagerHandler for OutputManager {
	fn output_added<'output>(
		&mut self,
		_: WLRCompositorHandle,
		builder: WLROutputBuilder<'output>,
	) -> Option<WLROutputBuilderResult<'output>> {
		Some(builder.build_best_mode(Output))
	}
}

/*
..####...######...####...######.
.##......##......##..##....##...
..####...####....######....##...
.....##..##......##..##....##...
..####...######..##..##....##...
................................
*/

struct SeatHandler;
impl WLRSeatHandler for SeatHandler {}

/*
..####...##..##..#####...######...####....####...######.
.##......##..##..##..##..##......##..##..##..##..##.....
..####...##..##..#####...####....######..##......####...
.....##..##..##..##..##..##......##..##..##..##..##.....
..####....####...##..##..##......##..##...####...######.
........................................................
*/

struct Surface;
impl WLRSurfaceHandler for Surface {
	fn on_commit(&mut self, _: WLRCompositorHandle, surface: WLRSurfaceHandle) {
		println!("Commiting for surface {:?}", surface);
	}
}

/*
..####...##..##..######..##......##.....
.##......##..##..##......##......##.....
..####...######..####....##......##.....
.....##..##..##..##......##......##.....
..####...##..##..######..######..######.
........................................
*/

struct XdgV6ShellHandlerEx;
impl WLRXdgV6ShellHandler for XdgV6ShellHandlerEx {
	fn destroyed(&mut self, compositor: WLRCompositorHandle, shell: WLRXdgV6ShellSurfaceHandle) {
		with_handles!([(compositor: {compositor})] => {
			let state: &mut State = compositor.into();
			let weak = shell;
			if let Some(index) = state.shells.iter().position(|s| *s == weak) {
				state.shells.remove(index);
			}
		}).unwrap();
	}
}

struct XdgV6ShellManager;
impl WLRXdgV6ShellManagerHandler for XdgV6ShellManager {
	fn new_surface(
		&mut self,
		compositor: WLRCompositorHandle,
		shell: WLRXdgV6ShellSurfaceHandle,
	) -> (Option<Box<WLRXdgV6ShellHandler>>, Option<Box<WLRSurfaceHandler>>) {
		dehandle!(
				@compositor = {compositor};
				@shell = {shell};
				shell.ping();
				match shell.state().unwrap() {
					WLRTopLevel(top_level) => {
						let top_level_title = top_level.title();
						println!("INFO: top_level_title = {}", top_level_title);
						if top_level.added() {
							println!("INFO: is added!");
						}
						top_level.set_activated(true);
					},
					_ => {}
				};
				let state: &mut State = compositor.into();
				state.shells.push(shell.weak_reference());
				/* @layout = {&state.layout};
				for (mut output, _) in layout.outputs() => {
						@output = {output};
						output.schedule_frame()
				} */
				()
			);
		(Some(Box::new(XdgV6ShellHandlerEx)), Some(Box::new(Surface)))
	}
}

/// Render the shells in the current compositor state on the given output
/// renderer.
fn render_shells(state: &mut State, renderer: &mut Renderer) {
	let shells: Vec<WLRXdgV6ShellSurfaceHandle> = state.shells.clone();
	for shell in shells {
		dehandle!(
			@shell = {shell};
			@surface = {shell.surface()};
			let (width, height) = surface.current_state().size();
			let (render_width, render_height) =
				(
					width * renderer.output.scale() as i32,
					height * renderer.output.scale() as i32
				);
			let (lx, ly) = (0.0, 0.0);
			let render_box = Area::new(
				Origin::new(lx as i32, ly as i32),
				Size::new(render_width,render_height)
			);

			state.current_rotation = state.current_rotation + 0.01;

			// This conditionnal is to prevent an error caused by a macro, this will
			// replace with a check on the intersection with the OutputLayout
			if true {
				let transform = renderer.output.get_transform().invert();
				let matrix = project_box(
					render_box,
					transform,
					state.current_rotation,
					renderer.output.transform_matrix()
				);
				if let Some(texture) = surface.texture().as_ref() {
					renderer.render_texture_with_matrix(texture, matrix);
				}
				surface.send_frame_done(current_time());
			};
			()
		);
	}
}

/*
.##..##..######..##..##..#####....####....####...#####...#####..
.##.##...##.......####...##..##..##..##..##..##..##..##..##..##.
.####....####......##....#####...##..##..######..#####...##..##.
.##.##...##........##....##..##..##..##..##..##..##..##..##..##.
.##..##..######....##....#####....####...##..##..##..##..#####..
................................................................
*/

struct KeyboardHandler;
impl WLRKeyboardHandler for KeyboardHandler {
	fn on_key(&mut self, compositor: WLRCompositorHandle, _keyboard: WLRKeyboardHandle, key_event: &WLRKeyEvent) {
		dehandle!(
			@compositor = {compositor};
			for key in key_event.pressed_keys() {
				if key == keysyms::KEY_Escape {
					wlroots::terminate();
				}
			};
			let state: &mut State = compositor.into();
			let seat_handle = state.seat_handle.clone().unwrap();
			@seat = {seat_handle};
			println!("Notifying seat of keypress: time_msec: '{:?}' keycode: '{}' key_state: '{}'", key_event.time_msec(), key_event.keycode(), key_event.key_state() as u32);
			seat.keyboard_notify_key(
				key_event.time_msec(),
				key_event.keycode(),
				key_event.key_state() as u32
			)
		);
	}
}

/*
.######..##..##..#####...##..##..######.
...##....###.##..##..##..##..##....##...
...##....##.###..#####...##..##....##...
...##....##..##..##......##..##....##...
.######..##..##..##.......####.....##...
........................................
*/

struct InputManager;
impl WLRInputManagerHandler for InputManager {
	fn keyboard_added(
		&mut self,
		compositor: WLRCompositorHandle,
		keyboard: WLRKeyboardHandle,
	) -> Option<Box<WLRKeyboardHandler>> {
		dehandle!(
			@compositor = {compositor};
			@keyboard = {keyboard};
			let state: &mut State = compositor.into();
			state.keyboard = Some(keyboard.weak_reference());
			@seat = {state.seat_handle.as_ref().unwrap()};
			seat.set_keyboard(keyboard.input_device())
		);
		Some(Box::new(KeyboardHandler))
	}
}

/*
.##.....##....###....####.##....##
.###...###...##.##....##..###...##
.####.####..##...##...##..####..##
.##.###.##.##.....##..##..##.##.##
.##.....##.#########..##..##..####
.##.....##.##.....##..##..##...###
.##.....##.##.....##.####.##....##
*/

fn main() {
	wlr_init_logging(WLR_DEBUG, None);

	let mut compositor = WLRCompositorBuilder::new()
		.gles2(true)
		.input_manager(Box::new(InputManager))
		.output_manager(Box::new(OutputManager))
		.xdg_shell_v6_manager(Box::new(XdgV6ShellManager))
		.data_device(true)
		.build_auto(State::new());

	{
		let seat_handle = Seat::create(&mut compositor, "seat0".into(), Box::new(SeatHandler));
		seat_handle
			.run(|seat| {
				seat.set_capabilities(Capability::all());
			}).unwrap();
		let state: &mut State = (&mut compositor).into();
		state.seat_handle = Some(seat_handle);
	}

	compositor.run()
}
