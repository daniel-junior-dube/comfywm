use wlroots::{
	CompositorHandle as WLRCompositorHandle, SurfaceHandle as WLRSurfaceHandle, SurfaceHandler as WLRSurfaceHandler,
};

/*
..####...##..##..#####...######...####....####...######.
.##......##..##..##..##..##......##..##..##..##..##.....
..####...##..##..#####...####....######..##......####...
.....##..##..##..##..##..##......##..##..##..##..##.....
..####....####...##..##..##......##..##...####...######.
........................................................
*/

pub struct Surface;
impl WLRSurfaceHandler for Surface {
	fn on_commit(&mut self, _: WLRCompositorHandle, surface: WLRSurfaceHandle) {
		println!("Commiting for surface {:?}", surface);
	}
}
