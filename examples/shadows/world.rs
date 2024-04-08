use std::{borrow::Cow, f32::consts, iter, mem};
use glam::Mat4;

use wgpu::{Sampler, ShaderModule, TextureView};

use spark_gap::gpu_context::GpuContext;
use spark_gap::texture::DEPTH_FORMAT;

use crate::cube::Vertex;
use crate::entities::Entities;
use crate::lights::{Lights, LightUniform};
use crate::render_passes::{create_forward_pass, create_shadow_pass, ForwardPass, SHADOW_FORMAT, SHADOW_SIZE, ShadowPass};

pub struct World {
    pub entities: Entities,
    pub lights: Lights,
    pub shader: ShaderModule,
    pub shadow_pass: ShadowPass,
    pub shadow_view: TextureView,
    pub shadow_sampler: Sampler,
    pub forward_pass: ForwardPass,
    pub forward_depth: TextureView,
}

impl World {
    pub fn new(gpu_context: &mut GpuContext) -> Self {

        let entities = Entities::new(gpu_context);

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

        let lights = Lights::new(gpu_context, &shadow_texture);

        let shader = gpu_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let forward_depth = create_depth_texture(gpu_context);

        let shadow_pass = create_shadow_pass(gpu_context, &entities.entity_bind_group_layout, &shader);

        let forward_pass = create_forward_pass(
            gpu_context,
            &entities.entity_bind_group_layout,
            &lights,
            &shader,
            &shadow_view,
            &shadow_sampler,
        );

        World {
            entities,
            lights,
            shader,
            shadow_pass,
            shadow_view,
            shadow_sampler,
            forward_pass,
            forward_depth,
        }
    }

    pub fn resize(&mut self, gpu_context: &GpuContext) {

        let mx_total = get_projection_view_matrix(gpu_context.config.width as f32 / gpu_context.config.height as f32);
        let mx_ref: &[f32; 16] = mx_total.as_ref();

        gpu_context
            .queue
            .write_buffer(&self.forward_pass.projection_view_buffer, 0, bytemuck::cast_slice(mx_ref));

        self.forward_depth = create_depth_texture(gpu_context);
    }

    pub fn render(&mut self, context: &GpuContext) {

        self.entities.update(context);
        self.lights.update(context);

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        encoder.push_debug_group("shadow passes");

        for (i, light) in self.lights.lights.iter().enumerate()
        {
            encoder.push_debug_group(&format!("shadow pass {} (light at position {:?})", i, light.position));

            // this could also be done with instances

            // The light uniform buffer already has the projection,
            // so just copy it over to the shadow uniform buffer.
            //
            // This is a command that will occur in sync with the queue ensuring
            // the right data is in the buffer at the time of actual rendering
            encoder.copy_buffer_to_buffer(
                &self.lights.light_storage_buffer,
                (i * mem::size_of::<LightUniform>()) as wgpu::BufferAddress,
                &self.shadow_pass.projection_view_buffer,
                0,
                mem::size_of::<Mat4>() as wgpu::BufferAddress, // 64,
            );

            // Using write_buffer doesn't work here because updating the buffer will be out of sync with the
            // queue commands so that when the rendering actually occurs the buffer may not
            // have the right data at the moment of rendering.
            // context.queue.write_buffer(
            //     &self.shadow_pass.projection_view_buffer,
            //     0,
            //     bytemuck::bytes_of(&light.projection_view.to_cols_array()));

            encoder.insert_debug_marker("render entities");
            {
                let depth_stencil_attachment = wgpu::RenderPassDepthStencilAttachment {
                    view: &light.shadow_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                };

                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[],
                    depth_stencil_attachment: Some(depth_stencil_attachment),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                pass.set_pipeline(&self.shadow_pass.pipeline);
                pass.set_bind_group(0, &self.shadow_pass.bind_group, &[]);

                for entity in &self.entities.entities {
                    pass.set_bind_group(1, &self.entities.entity_bind_group, &[entity.uniform_offset]);
                    pass.set_vertex_buffer(0, entity.vertex_buf.slice(..));
                    pass.set_index_buffer(entity.index_buf.slice(..), entity.index_format);
                    pass.draw_indexed(0..entity.index_count as u32, 0, 0..1);
                }
            }

            encoder.pop_debug_group(); // render entities
        }

        encoder.pop_debug_group(); // shadow pass

        //
        // forward pass
        //
        encoder.push_debug_group("forward rendering pass");

        let frame = context
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let texture_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        {
            let color_attachment = wgpu::RenderPassColorAttachment {
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
            };

            let depth_stencil_attachment = wgpu::RenderPassDepthStencilAttachment {
                view: &self.forward_depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            };

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: Some(depth_stencil_attachment),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.forward_pass.pipeline);
            pass.set_bind_group(0, &self.forward_pass.bind_group, &[]);

            for entity in &self.entities.entities {
                pass.set_bind_group(1, &self.entities.entity_bind_group, &[entity.uniform_offset]);

                pass.set_vertex_buffer(0, entity.vertex_buf.slice(..));
                pass.set_index_buffer(entity.index_buf.slice(..), entity.index_format);

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

pub fn get_projection_view_matrix(aspect_ratio: f32) -> glam::Mat4 {
    let projection = glam::Mat4::perspective_rh(consts::FRAC_PI_4, aspect_ratio, 1.0, 20.0);
    let view = glam::Mat4::look_at_rh(glam::Vec3::new(3.0f32, -10.0, 6.0), glam::Vec3::new(0f32, 0.0, 0.0), glam::Vec3::Z);
    projection * view
}

fn create_depth_texture(gpu_context: &GpuContext) -> TextureView {
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
