use main_loop::MainLoop;

mod app;
mod audio;
mod frame_mgr;
mod main_loop;
mod platform_specific;
mod video;

fn main() {
    MainLoop::new(30.).run();
}
