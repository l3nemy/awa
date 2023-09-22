use winit::{
    dpi::{LogicalPosition, Position},
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::{Window, WindowBuilder},
};

use crate::{app::App, frame_mgr::FrameManager, platform_specific};


#[derive(Debug, Clone, Copy)]
pub(crate) enum Message {
    Quit,

}

pub(crate) struct MainLoop {
    event_loop: EventLoop<Message>,

    window: Window,

    frame_mgr: FrameManager,

    app: App,

    input_helper: winit_input_helper::WinitInputHelper,
}

impl MainLoop {
    pub(crate) fn new(framerate: f64) -> Self {
        let event_loop = EventLoopBuilder::<Message>::with_user_event().build();

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
            window,
            mut frame_mgr,
            app,
            input_helper: mut input,
        } = self;

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Event<'static, Message>>();

        let app_ref = app.clone();

        runtime.spawn(async move {
            loop {
                if !frame_mgr.next_frame(&app_ref).await {
                    panic!("Failed to render");
                }
            }
        });

        let app_ref2 = app.clone();
        let event_loop_proxy = event_loop.create_proxy();

        runtime.spawn(async move {
            loop {
                if let Some(event) = rx.recv().await {
                    app_ref2.handle_input(&event).await;
                    match event {
                        Event::WindowEvent { event, .. } => match &event {
                            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                                app_ref2.update_scale_factor(*scale_factor).await;
                            }

                            WindowEvent::Resized(size) => {
                                if let Err(e) = app_ref2.update_surface_size(*size).await {
                                    eprintln!("Error resizing: {}", e);
                                    event_loop_proxy.send_event(Message::Quit).unwrap();
                                }
                            }

                            _ => {}
                        },

                        Event::MainEventsCleared => {
                            app_ref2.update().await;
                        }

                        _ => {}
                    }
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

                window.request_redraw();
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

                Event::UserEvent(Message::Quit) => {
                    *control_flow = ControlFlow::Exit;
                }

                _ => {}
            }
            tx.send(event.to_static().unwrap()).unwrap();
        })
    }
}
