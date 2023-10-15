use std::sync::Arc;

use tokio::sync::Mutex;

use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use winit::{dpi::PhysicalSize, event::Event, window::Window};
use winit_input_helper::WinitInputHelper;

use crate::video::Video;

pub(crate) struct App {
    _inner: Arc<Mutex<AppInner>>,
}

impl Clone for App {
    fn clone(&self) -> Self {
        Self {
            _inner: self._inner.clone(),
        }
    }
}

pub(crate) struct AppInner {
    input_helper: WinitInputHelper,

    pixels: Pixels,

    video: Video,

    // TODO: Use scale factor for HIDPI
    scale_factor: f64,
}

impl AppInner {
    pub(crate) fn render(&mut self) -> Result<(), pixels::Error> {
        if self.video.render(self.pixels.frame_mut()) {
            self.pixels.render_with(|encoder, render_target, ctx| {
                ctx.scaling_renderer.render(encoder, render_target);
                Ok(())
            })
        } else {
            Ok(())
        }
    }

    #[inline]
    pub(crate) fn update(&mut self) {
        self.video.update().unwrap();
    }

    pub(crate) async fn update_surface_size<S>(&mut self, size: S) -> Result<(), anyhow::Error>
    where
        S: Into<PhysicalSize<u32>>,
    {
        let size: PhysicalSize<u32> = size.into();

        self.video.update_surface_size(size)?;
        self.pixels
            .resize_surface(size.width, size.height)
            .map_err(anyhow::Error::from)
    }

    #[inline]
    pub(crate) fn update_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
    }

    #[inline]
    pub(crate) async fn handle_input<'e, T>(&mut self, event: &Event<'e, T>) {
        if self.input_helper.update(event) {
            self.update();
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
            _inner: Arc::new(Mutex::new(AppInner {
                input_helper: WinitInputHelper::new(),
                pixels,
                video,
                scale_factor: window.scale_factor(),
            })),
        }
    }

    #[inline]
    async fn inner(&self) -> tokio::sync::MutexGuard<'_, AppInner> {
        self._inner.lock().await
    }

    #[inline]
    pub(crate) async fn render(&self) -> Result<(), pixels::Error> {
        self.inner().await.render()
    }

    #[inline]
    pub(crate) async fn update(&self) {
        self.inner().await.update();
    }

    #[inline]
    pub(crate) async fn update_surface_size<S>(&self, size: S) -> Result<(), anyhow::Error>
    where
        S: Into<PhysicalSize<u32>>,
    {
        self.inner().await.update_surface_size(size).await
    }

    #[inline]
    pub(crate) async fn update_scale_factor(&self, scale_factor: f64) {
        self.inner().await.update_scale_factor(scale_factor);
    }

    #[inline]
    pub(crate) async fn handle_input<'e, T>(&self, event: &Event<'e, T>) {
        self.inner().await.handle_input(event).await;
    }
}
