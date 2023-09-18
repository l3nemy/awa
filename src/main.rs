use main_loop::MainLoop;

mod app;
mod main_loop;
mod platform_specific;

fn main() {
    MainLoop::new().run();
}
