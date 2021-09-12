// Based on https://github.com/nannou-org/nannou/blob/master/examples/draw/draw_capture_hi_res.rs

use {
    nannou::{
        app::App,
        draw::Draw,
        draw::{Renderer, RendererBuilder},
        frame::Frame,
        prelude::WindowId,
        wgpu::{
            CommandEncoderDescriptor, Textue5cfe74reSnapshot, Texture, TextureBuilder,
            TextureCapturer, TextureCapturerAwaitWorkerTimeout, TextureFormat, TextureReshaper,
            TextureUsage,
        },
    },
    std::{future::Future, path::Path},
};

pub struct CaptureHelper {
    window_id: WindowId,
    texture: Texture,
    renderer: Renderer,
    texture_capturer: TextureCapturer,
    texture_reshaper: TextureReshaper,
    snapshot: Option<Textue5cfe74reSnapshot>,
}

impl CaptureHelper {
    pub fn from_main_window(app: &App, file_dimensions: [u32; 2]) -> Self {
        Self::from_window_id(app, app.main_window().id(), file_dimensions)
    }

    pub fn from_window_id(app: &App, window_id: WindowId, file_dimensions: [u32; 2]) -> Self {
        let window = app.window(window_id).expect("Invalid window ID");

        let wgpu_device = window.swap_chain_device();

        let sample_count = window.msaa_samples();
        let texture = TextureBuilder::new()
            .size(file_dimensions)
            .usage(TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED)
            .sample_count(sample_count)
            .format(TextureFormat::Rgba16Float)
            .build(wgpu_device);

        let renderer =
            RendererBuilder::new().build_from_texture_descriptor(wgpu_device, texture.descriptor());

        let texture_capturer = TextureCapturer::default();

        let texture_reshaper = TextureReshaper::new(
            wgpu_device,
            &texture.view().build(),
            sample_count,
            texture.sample_type(),
            sample_count,
            Frame::TEXTURE_FORMAT,
        );

        Self {
            window_id,
            texture,
            renderer,
            texture_capturer,
            texture_reshaper,
            snapshot: None,
        }
    }

    pub fn render_image(&self, app: &App, draw: &Draw) {
        // This should be safe, since Self can't be moved between threads. We
        // interpret it as mutable just for this function.
        let self_mut = unsafe { (self as *const Self as *mut Self).as_mut() }.unwrap();

        let window = app.window(self.window_id).expect("Invalid window ID");

        let wgpu_device = window.swap_chain_device();
        let ce_desc = CommandEncoderDescriptor {
            label: Some("texture renderer"),
        };
        let mut encoder = wgpu_device.create_command_encoder(&ce_desc);
        self_mut
            .renderer
            .render_to_texture(wgpu_device, &mut encoder, draw, &self.texture);

        self_mut.snapshot = Some(self.texture_capturer.capture(
            wgpu_device,
            &mut encoder,
            &self.texture,
        ));

        window.swap_chain_queue().submit(Some(encoder.finish()));
    }

    pub fn write_to_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(), TextureCapturerAwaitWorkerTimeout<impl Future<Output = ()>>> {
        // This should be safe, since Self can't be moved between threads. We
        // interpret it as mutable just for this function.
        let self_mut = unsafe { (self as *const Self as *mut Self).as_mut() }.unwrap();

        let path = path.as_ref().to_owned();

        let snapshot = self_mut
            .snapshot
            .take()
            .expect("render_image should be called before writing to a file.");
        snapshot.read(move |result| {
            let image = result.expect("failed to map texture memory").to_owned();
            image
                .save(&path)
                .expect("failed to save texture to png image");
        })?;

        Ok(())
    }

    pub fn display_in_window(&self, frame: &Frame) {
        let mut encoder = frame.command_encoder();
        self.texture_reshaper
            .encode_render_pass(frame.texture_view(), &mut *encoder);
    }

    pub fn close(&mut self, app: &App) -> Result<(), TextureCapturerAwaitWorkerTimeout<()>> {
        println!("Waiting for PNG writing to complete...");

        let window = app.window(self.window_id).expect("Invalid window ID");
        let wgpu_device = window.swap_chain_device();
        self.texture_capturer.await_active_snapshots(wgpu_device)?;

        println!("Done!");

        Ok(())
    }
}
