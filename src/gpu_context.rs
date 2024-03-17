use crate::hash_map::HashMap;
use log::debug;
use std::rc::Rc;
use std::sync::Arc;
use wgpu::BindGroupLayout;
use winit::window::Window;

pub struct GpuContext {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub bind_layout_cache: HashMap<String, Rc<BindGroupLayout>>,
}

impl Drop for GpuContext {
    fn drop(&mut self) {
        debug!("Context dropped")
    }
}

impl GpuContext {
    pub async fn new(window: Arc<Window>) -> GpuContext {
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let desired_max_bind_groups = 8;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // required_limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
                    required_limits: wgpu::Limits {
                        max_bind_groups: desired_max_bind_groups,
                        ..wgpu::Limits::default() // Fill in other limits with default values
                    },
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let surface_caps = surface.get_capabilities(&adapter);

        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        Self {
            window,
            surface,
            adapter,
            device,
            queue,
            config,
            size,
            bind_layout_cache: HashMap::new(),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size.width = new_size.width.max(1);
        self.size.height = new_size.height.max(1);
        self.config.width = self.size.width;
        self.config.height = self.size.height;
        self.surface.configure(&self.device, &self.config);
    }
}
