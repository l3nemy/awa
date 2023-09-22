use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

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
    input_helper: WinitInputHelper,

    pixels: Mutex<Pixels>,

    video: Mutex<Video>,

    size: Mutex<PhysicalSize<u32>>,

    // TODO: Use scale factor for HIDPI
    scale_factor: Mutex<f64>,
}

impl AppInner {
    pub(crate) fn update_surface_size<S>(&self, size: S) -> Result<(), anyhow::Error>
    where
        S: Into<PhysicalSize<u32>>,
    {
        let size: PhysicalSize<u32> = size.into();
        *self.size.lock().unwrap() = size;
        self.video.lock().unwrap().update_surface_size(size)?;
        self.pixels
            .lock()
            .unwrap()
            .resize_surface(size.width, size.height)
            .map_err(anyhow::Error::from)
    }

    pub(crate) fn update(&self) {
        self.video.lock().unwrap().update().unwrap();
    }

    pub(crate) fn update_scale_factor(&self, scale_factor: f64) {
        *self.scale_factor.lock().unwrap() = scale_factor;
    }

    // TODO(l3nemy): remove this mut
    pub(crate) fn handle_input<T>(&mut self, event: &Event<T>) {
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
            _inner: Arc::new(AppInner {
                input_helper: WinitInputHelper::new(),
                pixels: Mutex::new(pixels),
                video: Mutex::new(video),
                size: Mutex::new(size),
                scale_factor: Mutex::new(window.scale_factor()),
            }),
        }
    }

    pub(crate) fn render(&self) -> Result<(), pixels::Error> {
        /*
        let frame = self.pixels.frame_mut();
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % self.size.width as usize) as i32;
            let y = (i / self.size.width as usize) as i32;

            pixel.copy_from_slice(&[(x % 256) as u8, (y % 256) as u8, 0, 255]);
        }*/
        let mut pixels = self._inner.pixels.lock().unwrap();
        let video = self._inner.video.lock().unwrap();

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
