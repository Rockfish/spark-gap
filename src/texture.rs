use crate::context::Context;
use crate::error::Error;
use crate::error::Error::ImageError;
use crate::texture_config::{TextureConfig, TextureFilter, TextureType, TextureWrap};
use image::GenericImageView;
use std::ffi::OsString;
use std::fmt::Display;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Texture {
    pub texture: Rc<wgpu::Texture>,
    pub view: Rc<wgpu::TextureView>,
    pub sampler: Rc<wgpu::Sampler>,
    pub texture_path: OsString,
    pub texture_type: TextureType,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn new(
        context: &Context,
        file_path: impl Into<PathBuf>,
        texture_config: &TextureConfig,
    ) -> Result<Texture, Error> {
        let file_path = file_path.into();
        load_texture(context, &file_path, texture_config)
    }
}

pub fn load_texture(
    context: &Context,
    texture_path: &PathBuf,
    texture_config: &TextureConfig,
) -> Result<Texture, Error> {
    let img = match image::open(texture_path) {
        Ok(img) => img,
        Err(e) => {
            return Err(ImageError(format!(
                "image error: {:?}  file: {:?}",
                e, texture_path
            )))
        }
    };

    let (width, height) = img.dimensions();

    let img = if texture_config.flip_v {
        img.flipv()
    } else {
        img
    };
    let img = if texture_config.flip_h {
        img.fliph()
    } else {
        img
    };

    let img_rgba = img.to_rgba8();

    let texture_size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let wgpu_texture = context.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("diffuse_texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    context.queue.write_texture(
        // Tells wgpu where to copy the pixel data
        wgpu::ImageCopyTexture {
            texture: &wgpu_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        // The actual pixel data
        &img_rgba,
        // The layout of the texture
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: Some(height),
        },
        texture_size,
    );

    let texture_view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let wrap_param = match texture_config.wrap {
        TextureWrap::Clamp => wgpu::AddressMode::ClampToEdge,
        TextureWrap::Repeat => wgpu::AddressMode::Repeat,
    };

    let filter_mode = match texture_config.filter {
        TextureFilter::Linear => wgpu::FilterMode::Linear,
        TextureFilter::Nearest => wgpu::FilterMode::Nearest,
    };

    let texture_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wrap_param,
        address_mode_v: wrap_param,
        address_mode_w: wrap_param,
        mag_filter: filter_mode,
        min_filter: filter_mode,
        mipmap_filter: filter_mode,
        ..Default::default()
    });

    let texture = Texture {
        texture: wgpu_texture.into(),
        view: texture_view.into(),
        sampler: texture_sampler.into(),
        texture_path: texture_path.into(),
        texture_type: texture_config.texture_type,
        width,
        height,
    };

    Ok(texture)
}
