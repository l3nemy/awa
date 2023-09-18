use winit::{
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{app::App, platform_specific};

pub(crate) struct MainLoop {
    // TODO: Add user event
    event_loop: EventLoop<()>,

    window: Window,

    app: App,

    input_helper: winit_input_helper::WinitInputHelper,
}

impl MainLoop {
    pub(crate) fn new() -> Self {
        let event_loop = EventLoop::new();

        //TODO: Multiple monitor support
        let monitor = event_loop
            .primary_monitor()
            .expect("Failed to get primary monitor");

        // TODO: size variation
        let window = WindowBuilder::new()
            .with_inner_size(monitor.size())
            .with_decorations(false)
            .with_active(false)
            .with_title("Awa Desktop")
            .build(&event_loop)
            .unwrap();
        platform_specific::set_desktop_window(&window);

        let app = App::new(&window);
        let input_helper = winit_input_helper::WinitInputHelper::new();

        Self {
            event_loop,
            window,
            app,
            input_helper,
        }
    }

    pub(crate) fn run(self) -> ! {
        let Self {
            event_loop,
            window,
            mut app,
            input_helper: mut input,
        } = self;

        event_loop.run(move |event, _, control_flow| {
            if input.update(&event) {
                if input.close_requested() || input.key_pressed(VirtualKeyCode::Escape) {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                if let Some(new_scale_factor) = input.scale_factor_changed() {
                    app.update_scale_factor(new_scale_factor);
                }

                if let Some(s) = input.window_resized() {
                    if let Err(e) = app.update_surface_size(s) {
                        eprintln!("pixels.resize_surface() failed: {}", e);
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                }

                //TODO: Handle keyboard, mouse event

                window.request_redraw();
            }

            app.handle_input(&event);

            #[allow(clippy::collapsible_match)]
            match event {
                Event::WindowEvent { event, .. } => {
                    //TODO: Handle egui input

                    if let WindowEvent::CloseRequested = event {
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                    }
                }

                Event::MainEventsCleared => {
                    window.request_redraw();
                }

                Event::RedrawRequested(_) => {
                    if let Err(e) = app.render() {
                        eprintln!("app.render() failed: {}", e);
                        *control_flow = ControlFlow::Exit;
                    }
                }
                _ => *control_flow = ControlFlow::Poll,
            }
        });
    }
}
