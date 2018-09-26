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
	// ? A surface commit is done when changes made to a surface a completed and ready to be applied
	fn on_commit(&mut self, _: WLRCompositorHandle, surface: WLRSurfaceHandle) {
		println!("Commiting for surface {:?}", surface);
	}
}
