use std::ops::{Range, Rem};
use std::sync::Arc;
use std::time::{Duration, Instant};

use lyon::extra::rust_logo::build_logo_path;
use lyon::path::{Path, Polygon, NO_ATTRIBUTES};
use lyon::tessellation;
use lyon::tessellation::geometry_builder::*;
use lyon::tessellation::{FillOptions, FillTessellator};
use lyon::tessellation::{StrokeOptions, StrokeTessellator};

use lyon::algorithms::{rounded_polygon, walk};

use renderer::geometry::{Geom, Point2D, Primitive, TessellationOptions, Vector2};
use renderer::material::{Colour, Material, TextureStretchMode};
use renderer::Renderer;
use wgpu::core::device::queue;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

pub mod renderer;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    // create a transfomation matrix that maps from -1..1 to the window size
    let window_size = PhysicalSize::new(2200, 2200);

    let transform = -nalgebra::Matrix3::from([
        [2.0 / window_size.width as f32, 0.0, 0.0],
        [0.0, 2.0 / window_size.height as f32, 0.0],
        [-1.0, -1.0, 1.0],
    ]);

    // create a texture
    let texture1 = renderer::texture::Texture::new_from_image(
        image::load_from_memory(include_bytes!("test.png")).unwrap(),
    );
    let texture2 = renderer::texture::Texture::new_from_image(
        image::load_from_memory(include_bytes!("einstein.jpg")).unwrap(),
    );

    // create a blue circle
    let g1 = Geom::new(
        Primitive::Circle {
            center: Point2D::new(0.0, 0.0),
            radius: 100.0,
        },
        Material::Color {
            color: Colour::new(0.0, 1.0, 0.0, 1.0),
        },
        Some(transform.into()),
        vec![],
        TessellationOptions::Fill,
    );

    let g2 = Geom::new(
        Primitive::Rectangle {
            a: Point2D::new(-100.0, -100.0),
            b: Point2D::new(800.0, 700.0),
        },
        Material::Texture {
            texture: texture1,
            stretch: TextureStretchMode::Stretch,
        },
        Some(transform.into()),
        vec![],
        TessellationOptions::Fill,
    );

    let g2b = Geom::new(
        Primitive::Rectangle {
            a: Point2D::new(0.0, 0.0),
            b: Point2D::new(300.0, 300.0),
        },
        Material::Color {
            color: Colour::new(1.0, 0.0, 0.0, 1.0),
        },
        Some(transform.into()),
        vec![],
        TessellationOptions::simple_line(15.0),
    );

    let g3 = Geom::new(
        Primitive::Ellipse {
            center: Point2D::new(-200.0, -200.0),
            radii: Vector2::new(300.0, 400.0),
            rotation: 25.0,
        },
        Material::Texture {
            texture: texture2,
            stretch: TextureStretchMode::Stretch,
        },
        Some(transform.into()),
        vec![],
        TessellationOptions::Fill,
    );

    let g4 = Geom::new(
        Primitive::Line {
            a: Point2D::new(0.0, -50.0),
            b: Point2D::new(0.0, 50.0),
        },
        Material::Color {
            color: Colour::new(1.0, 1.0, 1.0, 1.0),
        },
        Some(transform.into()),
        vec![],
        TessellationOptions::simple_line(15.0),
    );

    let g5 = Geom::new(
        Primitive::Line {
            a: Point2D::new(-50.0, 0.0),
            b: Point2D::new(50.0, 0.0),
        },
        Material::Color {
            color: Colour::new(1.0, 1.0, 1.0, 1.0),
        },
        Some(transform.into()),
        vec![],
        TessellationOptions::simple_line(15.0),
    );

    let mut app = App {
        window: None,
        gfx_state: None,
        geoms: vec![g1, g2, g2b, g3, g4, g5],
        window_size,
    };

    event_loop.run_app(&mut app).unwrap();
}

/// The application itself
struct App {
    /// We need to store an `Arc` because both `App` (for `ApplicationHandler`) and `GfxState`
    /// (for the surface) require references to the `Window`.
    window: Option<Arc<Window>>,
    gfx_state: Option<GPUState>,
    geoms: Vec<Geom>,
    window_size: PhysicalSize<u32>,
}

impl ApplicationHandler for App {
    /// Create a new window
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let win_attrs = Window::default_attributes()
            .with_title("Lyon tessellation example")
            .with_inner_size(self.window_size);
        let window = Arc::new(event_loop.create_window(win_attrs).unwrap());

        let gfx_state = GPUState::new(Arc::clone(&window));
        window.request_redraw();

        self.window = Some(window);
        self.gfx_state = Some(gfx_state);
    }

    /// Handle redraw requests and other state changes
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Destroyed | WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                // todo!("resize");
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match key_code {
                _key => {}
            },
            _evt => {}
        };

        if event_loop.exiting() {
            return;
        }

        self.gfx_state.as_mut().unwrap().paint(&self.geoms);
    }
}

/// Everything needed for wgpu graphics
struct GPUState {
    device: wgpu::Device,
    surface_desc: wgpu::SurfaceConfiguration,
    /// Drawable surface, which contains an `Arc<Window>`
    surface: wgpu::Surface<'static>,
    queue: wgpu::Queue,
    renderer: Renderer,
}

impl GPUState {
    // impl<'win> GfxState<'win> {
    fn new(window: Arc<Window>) -> Self {
        // Create an instance
        let instance = wgpu::Instance::default();
        let size = window.inner_size();

        // Create a surface
        let surface = instance.create_surface(window).unwrap();

        let (device, queue) = pollster::block_on(async {
            // Create an adapter
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .unwrap();

            // Create a device and a queue
            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::default(),
                    },
                    None,
                )
                .await
                .unwrap()
        });

        let surface_desc = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            // defaults from `surface.get_default_config(...)``
            desired_maximum_frame_latency: 1,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        surface.configure(&device, &surface_desc);

        let renderer = Renderer::new(&device);

        Self {
            device,
            surface_desc,
            surface,
            queue,
            renderer,
        }
    }

    fn paint(&mut self, geoms: &[Geom]) {
        let frame = match self.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(e) => {
                println!("Swap-chain error: {e:?}");
                return;
            }
        };

        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Encoder"),
            });

        // draw the primitives
        let t0 = Instant::now();
        let rd = self
            .renderer
            .prepare(&mut self.device, &mut self.queue, geoms);

        let t1 = Instant::now();
        {
            // A resolve target is only supported if the attachment actually uses anti-aliasing
            // So if sample_count == 1 then we must render directly to the surface's buffer
            let color_attachment = wgpu::RenderPassColorAttachment {
                view: &frame_view,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
                resolve_target: None,
            };

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.renderer.render(&mut pass, &self.device, geoms, &rd);
        }

        // submit the commands
        self.queue.submit(std::iter::once(encoder.finish()));

        let t2 = Instant::now();

        println!(
            "Frame time: {:?} (prepare: {:?}, render: {:?})",
            t2 - t0,
            t1 - t0,
            t2 - t1
        );

        // present the frame
        frame.present();
    }
}