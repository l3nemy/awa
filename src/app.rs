use std::{ops::Deref, sync::Arc};

use tokio::sync::Mutex;

use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use winit::{dpi::PhysicalSize, event::Event, window::Window};
use winit_input_helper::WinitInputHelper;

use crate::video::Video;

pub(crate) struct App {
    _inner: Arc<AppInner>,
}

impl Clone for App {
    fn clone(&self) -> Self {
        Self {
            _inner: self._inner.clone(),
        }
    }
}

impl Deref for App {
    type Target = AppInner;

    fn deref(&self) -> &Self::Target {
        &self._inner
    }
}

pub(crate) struct AppInner {
    input_helper: Mutex<WinitInputHelper>,

    pixels: Mutex<Pixels>,

    video: Mutex<Video>,

    size: Mutex<PhysicalSize<u32>>,

    // TODO: Use scale factor for HIDPI
    scale_factor: Mutex<f64>,
}

impl AppInner {
    pub(crate) async fn update_surface_size<S>(&self, size: S) -> Result<(), anyhow::Error>
    where
        S: Into<PhysicalSize<u32>>,
    {
        let size: PhysicalSize<u32> = size.into();

        let (mut s, mut video, mut pixels) =
            tokio::join![self.size.lock(), self.video.lock(), self.pixels.lock()];
        *s = size;
        video.update_surface_size(size)?;
        pixels
            .resize_surface(size.width, size.height)
            .map_err(anyhow::Error::from)
    }

    pub(crate) async fn update(&self) {
        self.video.lock().await.update().unwrap();
    }

    pub(crate) async fn update_scale_factor(&self, scale_factor: f64) {
        *self.scale_factor.lock().await = scale_factor;
    }

    pub(crate) async fn handle_input<'e, T>(&self, event: &Event<'e, T>) {
        if self.input_helper.lock().await.update(event) {
            self.update().await;
        }
    }
}

impl App {
    pub(crate) fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, window);

        let pixels = PixelsBuilder::new(size.width, size.height, surface_texture)
            .present_mode(pixels::wgpu::PresentMode::AutoNoVsync)
            .clear_color(pixels::wgpu::Color {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 0.,
            })
            .build()
            .expect("Failed to create pixels object");
        let video = Video::new(size).unwrap();

        Self {
            _inner: Arc::new(AppInner {
                input_helper: Mutex::new(WinitInputHelper::new()),
                pixels: Mutex::new(pixels),
                video: Mutex::new(video),
                size: Mutex::new(size),
                scale_factor: Mutex::new(window.scale_factor()),
            }),
        }
    }

    pub(crate) async fn render(&self) -> Result<(), pixels::Error> {
        /*
        let frame = self.pixels.frame_mut();
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % self.size.width as usize) as i32;
            let y = (i / self.size.width as usize) as i32;

            pixel.copy_from_slice(&[(x % 256) as u8, (y % 256) as u8, 0, 255]);
        }*/

        let (mut pixels, video) =
            tokio::join![self._inner.pixels.lock(), self._inner.video.lock(),];

        if video.render(pixels.frame_mut()) {
            pixels.render_with(|encoder, render_target, ctx| {
                ctx.scaling_renderer.render(encoder, render_target);
                Ok(())
            })
        } else {
            Ok(())
        }
    }
}
