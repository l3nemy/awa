use winit::{
    dpi::{LogicalPosition, Position},
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{app::App, frame_mgr::FrameManager, platform_specific};

pub(crate) struct MainLoop {
    // TODO: Add user event
    event_loop: EventLoop<()>,

    window: Window,

    frame_mgr: FrameManager,

    app: App,

    input_helper: winit_input_helper::WinitInputHelper,
}

impl MainLoop {
    pub(crate) fn new(framerate: f64) -> Self {
        //let event_loop = EventLoopBuilder::<Message>::with_user_event().build();
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
            .with_position(Position::Logical(LogicalPosition::new(0., 0.)))
            .with_window_level(winit::window::WindowLevel::AlwaysOnBottom)
            .build(&event_loop)
            .unwrap();

        platform_specific::set_desktop_window(&window);

        let app = App::new(&window);
        let input_helper = winit_input_helper::WinitInputHelper::new();

        Self {
            event_loop,
            window,
            frame_mgr: FrameManager::new(framerate),
            app,
            input_helper,
        }
    }

    pub(crate) fn run(self) -> ! {
        let Self {
            event_loop,
            window: _window,
            mut frame_mgr,
            app,
            input_helper: mut input,
        } = self;

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        //let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Event<'static, ()>>();

        let app_ref = app.clone();

        runtime.spawn(async move {
            loop {
                if !frame_mgr.next_frame(&app_ref) {
                    panic!("Failed to render");
                }
            }
        });

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
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
                app.update();
                //app.handle_input(&event);
                //window.request_redraw();
            }

            #[allow(clippy::collapsible_match)]
            #[allow(clippy::single_match)]
            match &event {
                Event::WindowEvent { event, .. } => {
                    //TODO: Handle egui input

                    if let WindowEvent::CloseRequested = event {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                Event::MainEventsCleared => {
                    app.update();
                }
                _ => {}
            }
        })
    }
}
