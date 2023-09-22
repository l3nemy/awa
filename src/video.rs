use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use cfg_if::cfg_if;
use gst::{element_error, prelude::*, ElementFactory, Fraction, GhostPad};
use gst_app::AppSinkCallbacks;
use gst_video::VideoCapsBuilder;

use winit::dpi::PhysicalSize;

pub(crate) struct Video {
    pipeline: gst::Pipeline,

    #[allow(dead_code)]
    appsink: gst_app::AppSink,
    bus: Arc<gst::Bus>,

    surface_size: PhysicalSize<u32>,
    repeat: bool,

    #[allow(dead_code)]
    framerate: f64,

    need_render: Arc<AtomicBool>,

    frame: Arc<Mutex<Vec<u8>>>,
}

impl Drop for Video {
    fn drop(&mut self) {
        self.pipeline.set_state(gst::State::Null).unwrap();
    }
}

//unsafe impl Send for Video {}

impl Video {
    fn enable_factory(name: &str, enable: bool) -> bool {
        let registry = gst::Registry::get();
        if let Some(factory) = ElementFactory::find(name) {
            let factory = factory.upcast::<gst::PluginFeature>();
            if enable {
                factory.set_rank(gst::Rank::Primary + 4);
            } else {
                factory.set_rank(gst::Rank::None);
            }
            registry.add_feature(&factory).is_ok()
        } else {
            false
        }
    }

    fn create_pipeline<S>(
        uri: &str,
        size: S,
    ) -> Result<(gst::Pipeline, gst::Pad, gst_app::AppSink), anyhow::Error>
    where
        S: Into<PhysicalSize<u32>>,
    {
        let size: PhysicalSize<u32> = size.into();

        // {playbin} -> {sinkbin} ({aspectratiocrop} -> {videoconvertscale} -> {videorate} -> {appsink})

        // playbin uri={uri} video-sink="aspectratiocrop aspect-ratio={width}/{height} ! videoconvertscale ! videorate ! appsink" audio-sink="autoaudiosink"

        let playbin = ElementFactory::make("playbin")
            .property("uri", uri)
            .build()?
            .downcast::<gst::Pipeline>()
            .unwrap();

        let audiosink = ElementFactory::make("autoaudiosink").build()?;

        let sinkbin = gst::Bin::builder().name("sinkbin").build();

        let caps = VideoCapsBuilder::new()
            //.framerate(Fraction::from(60))
            .width(size.width as _)
            .height(size.height as _)
            .format(gst_video::VideoFormat::Rgba)
            .build();

        let aspectratiocrop = ElementFactory::make("aspectratiocrop")
            .property(
                "aspect-ratio",
                Fraction::new(size.width as _, size.height as _),
            )
            .build()?;
        let videoconvertscale = ElementFactory::make("videoconvertscale").build()?;
        let videorate = ElementFactory::make("videorate").build()?;

        let fpsdisplaysink = ElementFactory::make("fpsdisplaysink").build()?;

        let appsink = gst_app::AppSink::builder()
            .enable_last_sample(true)
            .caps(&caps)
            .async_(true)
            .sync(true)
            .build()
            .upcast::<gst::Element>();

        let capsframerate = ElementFactory::make("capsfilter")
            .property(
                "caps",
                gst::Caps::builder("video/x-raw")
                    .field("framerate", Fraction::new(30, 1))
                    .field("format", "RGBA")
                    .build(),
            )
            .build()?;

        fpsdisplaysink.set_property("video-sink", &appsink);

        sinkbin.add_many([
            &aspectratiocrop,
            &videorate,
            &capsframerate,
            &videoconvertscale,
            &fpsdisplaysink,
        ])?;

        gst::Element::link_many([
            &aspectratiocrop,
            &videorate,
            &capsframerate,
            &videoconvertscale,
            &fpsdisplaysink,
        ])?;

        let pad = aspectratiocrop.static_pad("sink").unwrap();
        let ghost_pad = GhostPad::builder_with_target(&pad)?.build();
        ghost_pad.set_active(true)?;
        sinkbin.add_pad(&ghost_pad)?;

        playbin.set_property("video-sink", &sinkbin);
        playbin.set_property("audio-sink", &audiosink);

        Ok((
            playbin,
            pad,
            appsink.downcast::<gst_app::AppSink>().unwrap(),
        ))
    }

    pub(crate) fn new<S>(size: S) -> Result<Self, anyhow::Error>
    where
        S: Into<PhysicalSize<u32>>,
    {
        gst::init()?;

        let size = size.into();

        cfg_if! {
            if #[cfg(target_os = "macos")] {
                Self::enable_factory("vtdec", true);

            } else if #[cfg(target_os = "windows")] {
                Self::enable_factory("d3d11h264dec", true);
                Self::enable_factory("d3d11h265dec", true);
                Self::enable_factory("d3d11vp8dec", true);
                Self::enable_factory("d3d11vp9dec", true);
                Self::enable_factory("d3d11mpeg2dec", true);
                Self::enable_factory("d3d11av1dec", true);

            } else if #[cfg(target_os = "linux")] {
                Self::enable_factory("vaapih264dec", true);
                Self::enable_factory("vaapivp8dec", true);
                Self::enable_factory("vaapivp9dec", true);
                Self::enable_factory("vaapijpegdec", true);
                Self::enable_factory("vaapimpeg2dec", true);
                Self::enable_factory("vaapimpeg4dec", true);
                Self::enable_factory("vaapih265dec", true);
                Self::enable_factory("vaapivc1dec", true);
                Self::enable_factory("vaapiav1dec", true);
                Self::enable_factory("vaapipostproc", true);
            }
        }

        let (pipeline, pad, appsink) = Self::create_pipeline(
            "https://gstreamer.freedesktop.org/media/sintel_trailer-480p.webm",
            size,
        )
        .unwrap();
        let bus = pipeline.bus().unwrap();

        let frame = Arc::new(Mutex::new(Vec::new()));
        let frame_ref = frame.clone();

        let need_render = Arc::new(AtomicBool::new(false));
        let need_render_ref = need_render.clone();

        appsink.set_callbacks(
            AppSinkCallbacks::builder()
                .new_sample(move |appsink| {
                    let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Eos)?;

                    let _info = sample
                        .caps()
                        .and_then(|caps| gst_video::VideoInfo::from_caps(caps).ok())
                        .ok_or_else(|| {
                            element_error!(
                                appsink,
                                gst::ResourceError::Failed,
                                ("Failed to get video info from sample")
                            );

                            gst::FlowError::NotNegotiated
                        })?;

                    let mut f = frame_ref.lock().unwrap();
                    let buf = sample.buffer_owned().take().unwrap();

                    if f.len() != buf.size() {
                        f.resize(buf.size(), 0);
                    }
                    buf.copy_to_slice(0, &mut f).unwrap();

                    need_render_ref.store(true, Ordering::Release);

                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );

        pipeline.set_state(gst::State::Playing)?;
        pipeline.state(gst::ClockTime::from_seconds(5)).0?;

        let caps = pad.current_caps().unwrap();

        let s = caps.structure(0).unwrap();
        let framerate = s.get::<gst::Fraction>("framerate").unwrap();
        Ok(Self {
            pipeline,
            appsink,
            bus: Arc::new(bus),
            surface_size: size,
            frame,
            repeat: true,
            need_render,
            framerate: framerate.numer() as f64 / framerate.denom() as f64,
        })
    }

    pub(crate) fn rewind(&mut self) -> Result<(), anyhow::Error> {
        self.pipeline
            .seek_simple(gst::SeekFlags::FLUSH, gst::ClockTime::from_seconds(0))
            .map_err(anyhow::Error::from)
    }

    pub(crate) fn update(&mut self) -> Result<(), anyhow::Error> {
        use gst::MessageView::*;

        let bus = self.bus.clone();

        while let Some(msg) = bus.timed_pop_filtered(
            gst::ClockTime::from_mseconds(2),
            &[gst::MessageType::Eos, gst::MessageType::Error],
        ) {
            match msg.view() {
                Eos(_eos) => {
                    if self.repeat {
                        self.rewind()?;
                    }
                }
                // TODO(l3nemy): Handle error(Connection closed)
                Error(e) => Err(anyhow::anyhow!(
                    "Error from {:?}: {} ({:?})",
                    e.src().map(|s| s.path_string()),
                    e.error(),
                    e.debug()
                ))?,
                _ => {}
            }
        }
        Ok(())
    }

    pub(crate) fn render(&self, frame: &mut [u8]) -> bool {
        if self.need_render() {
            self.need_render.store(false, Ordering::Release);
            frame.swap_with_slice(self.frame.lock().unwrap().as_mut());
            true
        } else {
            false
        }
    }

    pub(crate) fn update_surface_size<S>(&mut self, size: S) -> Result<(), anyhow::Error>
    where
        S: Into<PhysicalSize<u32>>,
    {
        self.surface_size = size.into();
        self.update()
    }

    pub(crate) fn need_render(&self) -> bool {
        self.need_render.load(Ordering::Acquire)
    }
}
