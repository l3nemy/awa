use pixels::{Pixels, SurfaceTexture};
use winit::{dpi::PhysicalSize, event::Event, window::Window};
use winit_input_helper::WinitInputHelper;

pub(crate) struct App {
    input_helper: WinitInputHelper,

    pixels: Pixels,

    size: PhysicalSize<u32>,

    // TODO: Use scale factor for HIDPI
    scale_factor: f64,
}

impl App {
    pub(crate) fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, window);
        Self {
            input_helper: WinitInputHelper::new(),
            pixels: Pixels::new(size.width, size.height, surface_texture)
                .expect("Failed to create pixels object"),
            size,
            scale_factor: window.scale_factor(),
        }
    }

    pub(crate) fn update_surface_size<S>(&mut self, size: S) -> Result<(), pixels::TextureError>
    where
        S: Into<PhysicalSize<u32>>,
    {
        let size: PhysicalSize<u32> = size.into();
        self.size = size;
        self.pixels.resize_surface(size.width, size.height)
    }

    pub(crate) fn update_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
    }

    pub(crate) fn handle_input(&mut self, event: &Event<()>) {
        if self.input_helper.update(event) {
            self.update();
        }
    }

    pub(crate) fn update(&mut self) {}

    pub(crate) fn render(&mut self) -> Result<(), pixels::Error> {
        let frame = self.pixels.frame_mut();
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % self.size.width as usize) as i32;
            let y = (i / self.size.width as usize) as i32;

            pixel.copy_from_slice(&[(x % 256) as u8, (y % 256) as u8, 0, 255]);
        }
        self.pixels.render_with(|encoder, render_target, ctx| {
            ctx.scaling_renderer.render(encoder, render_target);
            Ok(())
        })
    }
}
