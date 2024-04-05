use std::{borrow::Cow, f32::consts, iter, mem, sync::Arc};

use crate::cube::{create_cube, create_plane, Vertex};
use bytemuck::{Pod, Zeroable};
use spark_gap::gpu_context::GpuContext;
use spark_gap::texture::DEPTH_FORMAT;
use wgpu::util::{align_to, DeviceExt};
use wgpu::{BindGroupLayout, Sampler, ShaderModule, TextureView};

use crate::light::{create_light_storage_buffer, Light, LightRaw};
use crate::render_passes::{create_forward_pass, create_shadow_pass, Pass, SHADOW_FORMAT, SHADOW_SIZE};

pub struct Entity {
    pub mx_world: glam::Mat4,
    pub rotation_speed: f32,
    pub color: wgpu::Color,
    pub vertex_buf: Arc<wgpu::Buffer>,
    pub index_buf: Arc<wgpu::Buffer>,
    pub index_format: wgpu::IndexFormat,
    pub index_count: usize,
    pub uniform_offset: wgpu::DynamicOffset,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GlobalUniforms {
    pub proj: [[f32; 4]; 4],
    pub num_lights: [u32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct EntityUniforms {
    pub model: [[f32; 4]; 4],
    pub color: [f32; 4],
}

pub struct World {
    pub entities: Vec<Entity>,
    pub lights: Vec<Light>,
    pub lights_are_dirty: bool,
    pub shader: ShaderModule,
    pub shadow_pass: Pass,
    pub shadow_view: TextureView,
    pub shadow_sampler: Sampler,
    pub forward_pass: Pass,
    pub forward_depth: wgpu::TextureView,
    pub entity_bind_group: wgpu::BindGroup,
    pub light_storage_buffer: wgpu::Buffer,
    pub entity_uniform_buf: wgpu::Buffer,
    pub local_bind_group_layout: BindGroupLayout,
}

impl World {
    pub fn new(gpu_context: &mut GpuContext) -> Self {

        let (cube_vertex_data, cube_index_data) = create_cube();

        let cube_vertex_buf = Arc::new(
            gpu_context.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                label: Some("Cubes Vertex Buffer"),
                contents: bytemuck::cast_slice(&cube_vertex_data),
                usage: wgpu::BufferUsages::VERTEX,
        }));

        let cube_index_buf = Arc::new(
            gpu_context.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                label: Some("Cubes Index Buffer"),
                contents: bytemuck::cast_slice(&cube_index_data),
                usage: wgpu::BufferUsages::INDEX,
        }));

        let (plane_vertex_data, plane_index_data) = create_plane(7);

        let plane_vertex_buf = gpu_context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Plane Vertex Buffer"),
                contents: bytemuck::cast_slice(&plane_vertex_data),
                usage: wgpu::BufferUsages::VERTEX,
        });

        let plane_index_buf = gpu_context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Plane Index Buffer"),
                contents: bytemuck::cast_slice(&plane_index_data),
                usage: wgpu::BufferUsages::INDEX,
        });

        struct CubeDesc {
            offset: glam::Vec3,
            angle: f32,
            scale: f32,
            rotation: f32,
        }

        let cube_descriptions = [
            CubeDesc {
                offset: glam::Vec3::new(-2.0, -2.0, 2.0),
                angle: 10.0,
                scale: 0.7,
                rotation: 0.1,
            },
            CubeDesc {
                offset: glam::Vec3::new(2.0, -2.0, 2.0),
                angle: 50.0,
                scale: 1.3,
                rotation: 0.2,
            },
            CubeDesc {
                offset: glam::Vec3::new(-2.0, 2.0, 2.0),
                angle: 140.0,
                scale: 1.1,
                rotation: 0.3,
            },
            CubeDesc {
                offset: glam::Vec3::new(2.0, 2.0, 2.0),
                angle: 210.0,
                scale: 0.9,
                rotation: 0.4,
            },
        ];

        let entity_uniform_size = mem::size_of::<EntityUniforms>() as wgpu::BufferAddress;

        let num_entities = 1 + cube_descriptions.len() as wgpu::BufferAddress;

        // Make the `uniform_alignment` >= `entity_uniform_size` and aligned to `min_uniform_buffer_offset_alignment`.
        let uniform_alignment = {
            let alignment = gpu_context.device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
            align_to(entity_uniform_size, alignment)
        };

        // Note: dynamic uniform offsets also have to be aligned to `Limits::min_uniform_buffer_offset_alignment`.
        let entity_uniform_buf = gpu_context.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: num_entities * uniform_alignment,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_format = wgpu::IndexFormat::Uint16;

        let mut entities = vec![{
            Entity {
                mx_world: glam::Mat4::IDENTITY,
                rotation_speed: 0.0,
                color: wgpu::Color::WHITE,
                vertex_buf: Arc::new(plane_vertex_buf),
                index_buf: Arc::new(plane_index_buf),
                index_format,
                index_count: plane_index_data.len(),
                uniform_offset: 0,
            }
        }];

        for (i, cube) in cube_descriptions.iter().enumerate() {
            let mx_world = glam::Mat4::from_scale_rotation_translation(
                glam::Vec3::splat(cube.scale),
                glam::Quat::from_axis_angle(cube.offset.normalize(), cube.angle * consts::PI / 180.),
                cube.offset,
            );

            entities.push(Entity {
                mx_world,
                rotation_speed: cube.rotation,
                color: wgpu::Color::GREEN,
                vertex_buf: Arc::clone(&cube_vertex_buf),
                index_buf: Arc::clone(&cube_index_buf),
                index_format,
                index_count: cube_index_data.len(),
                uniform_offset: ((i + 1) * uniform_alignment as usize) as _,
            });
        }

        let local_bind_group_layout = gpu_context.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(entity_uniform_size),
                    },
                    count: None,
                }],
                label: None,
        });

        let entity_bind_group = gpu_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &local_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &entity_uniform_buf,
                    offset: 0,
                    size: wgpu::BufferSize::new(entity_uniform_size),
                }),
            }],
            label: None,
        });

        // Create other resources
        let shadow_sampler = gpu_context.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let shadow_texture = gpu_context.device.create_texture(&wgpu::TextureDescriptor {
            size: SHADOW_SIZE,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: SHADOW_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            label: None,
            view_formats: &[],
        });

        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut shadow_target_views = (0..2)
            .map(|i| {
                Some(shadow_texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("shadow"),
                    format: None,
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: i as u32,
                    array_layer_count: Some(1),
                }))
            })
            .collect::<Vec<_>>();

        let lights = vec![
            Light {
                pos: glam::Vec3::new(7.0, -5.0, 10.0),
                color: wgpu::Color {
                    r: 0.5,
                    g: 1.0,
                    b: 0.5,
                    a: 1.0,
                },
                fov: 60.0,
                depth: 1.0..20.0,
                target_view: shadow_target_views[0].take().unwrap(),
            },
            Light {
                pos: glam::Vec3::new(-5.0, 7.0, 10.0),
                color: wgpu::Color {
                    r: 1.0,
                    g: 0.5,
                    b: 0.5,
                    a: 1.0,
                },
                fov: 45.0,
                depth: 1.0..20.0,
                target_view: shadow_target_views[1].take().unwrap(),
            },
        ];

        let light_storage_buffer = create_light_storage_buffer(gpu_context);

        let shader = gpu_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let forward_depth = create_depth_texture(gpu_context);

        let shadow_pass = create_shadow_pass(gpu_context, &local_bind_group_layout, &shader);

        let forward_pass = create_forward_pass(
            gpu_context,
            &local_bind_group_layout,
            &lights,
            &light_storage_buffer,
            &shader,
            &shadow_view,
            &shadow_sampler,
        );

        World {
            entities,
            lights,
            lights_are_dirty: true,
            shader,
            shadow_pass,
            shadow_view,
            shadow_sampler,
            forward_pass,
            forward_depth,
            light_storage_buffer,
            entity_uniform_buf,
            entity_bind_group,
            local_bind_group_layout,
        }
    }

    fn update(&mut self, _event: winit::event::WindowEvent) {
        //empty
    }

    fn resize(&mut self, gpu_context: &GpuContext) {
        // update view-projection matrix
        let mx_total = generate_matrix(gpu_context.config.width as f32 / gpu_context.config.height as f32);
        let mx_ref: &[f32; 16] = mx_total.as_ref();
        gpu_context
            .queue
            .write_buffer(&self.forward_pass.uniform_buf, 0, bytemuck::cast_slice(mx_ref));

        self.forward_depth = create_depth_texture(gpu_context);
    }

    pub fn render(&mut self, context: &GpuContext) {
        // update uniforms
        for entity in self.entities.iter_mut() {
            if entity.rotation_speed != 0.0 {
                let rotation = glam::Mat4::from_rotation_x(entity.rotation_speed * consts::PI / 180.);
                entity.mx_world *= rotation;
            }
            let data = EntityUniforms {
                model: entity.mx_world.to_cols_array_2d(),
                color: [
                    entity.color.r as f32,
                    entity.color.g as f32,
                    entity.color.b as f32,
                    entity.color.a as f32,
                ],
            };
            context.queue.write_buffer(
                &self.entity_uniform_buf,
                entity.uniform_offset as wgpu::BufferAddress,
                bytemuck::bytes_of(&data),
            );
        }

        if self.lights_are_dirty {
            self.lights_are_dirty = false;
            for (i, light) in self.lights.iter().enumerate() {
                context.queue.write_buffer(
                    &self.light_storage_buffer,
                    (i * mem::size_of::<LightRaw>()) as wgpu::BufferAddress,
                    bytemuck::bytes_of(&light.to_raw()),
                );
            }
        }

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        encoder.push_debug_group("shadow passes");

        for (i, light) in self.lights.iter().enumerate() {
            encoder.push_debug_group(&format!("shadow pass {} (light at position {:?})", i, light.pos));

            // The light uniform buffer already has the projection,
            // let's just copy it over to the shadow uniform buffer.
            encoder.copy_buffer_to_buffer(
                &self.light_storage_buffer,
                (i * mem::size_of::<LightRaw>()) as wgpu::BufferAddress,
                &self.shadow_pass.uniform_buf,
                0,
                64,
            );

            encoder.insert_debug_marker("render entities");
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &light.target_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&self.shadow_pass.pipeline);
                pass.set_bind_group(0, &self.shadow_pass.bind_group, &[]);

                for entity in &self.entities {
                    pass.set_bind_group(1, &self.entity_bind_group, &[entity.uniform_offset]);
                    pass.set_index_buffer(entity.index_buf.slice(..), entity.index_format);
                    pass.set_vertex_buffer(0, entity.vertex_buf.slice(..));
                    pass.draw_indexed(0..entity.index_count as u32, 0, 0..1);
                }
            }

            encoder.pop_debug_group(); // render entities
        }

        encoder.pop_debug_group(); // shadow pass

        // forward pass
        encoder.push_debug_group("forward rendering pass");

        let frame = context
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let texture_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.forward_depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.forward_pass.pipeline);
            pass.set_bind_group(0, &self.forward_pass.bind_group, &[]);

            for entity in &self.entities {
                pass.set_bind_group(1, &self.entity_bind_group, &[entity.uniform_offset]);
                pass.set_index_buffer(entity.index_buf.slice(..), entity.index_format);
                pass.set_vertex_buffer(0, entity.vertex_buf.slice(..));
                pass.draw_indexed(0..entity.index_count as u32, 0, 0..1);
            }
        }
        encoder.pop_debug_group();

        context.queue.submit(iter::once(encoder.finish()));
        frame.present();
    }
}

pub fn get_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    use std::mem;
    wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            // vertices
            wgpu::VertexAttribute {
                shader_location: 0,
                format: wgpu::VertexFormat::Sint8x4,
                offset: 0,
            },
            // tex coords
            wgpu::VertexAttribute {
                shader_location: 1,
                format: wgpu::VertexFormat::Sint8x4,
                // Sint8x4 is four signed bytes (i8). vec4<i32> in shaders
                offset: mem::size_of::<[i8; 4]>() as wgpu::BufferAddress,
            },
        ],
    }
}

pub fn generate_matrix(aspect_ratio: f32) -> glam::Mat4 {
    let projection = glam::Mat4::perspective_rh(consts::FRAC_PI_4, aspect_ratio, 1.0, 20.0);
    let view = glam::Mat4::look_at_rh(glam::Vec3::new(3.0f32, -10.0, 6.0), glam::Vec3::new(0f32, 0.0, 0.0), glam::Vec3::Z);
    projection * view
}

fn create_depth_texture(gpu_context: &GpuContext) -> wgpu::TextureView {
    let depth_texture = gpu_context.device.create_texture(&wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: gpu_context.config.width,
            height: gpu_context.config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: None,
        view_formats: &[],
    });

    depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
}

fn optional_features() -> wgpu::Features {
    wgpu::Features::DEPTH_CLIP_CONTROL
}
