use crate::error::Error;
use russimp::sys::aiTextureType;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Debug, Copy, Clone)]
pub enum TextureFilter {
    Linear,
    Nearest,
}

#[derive(Debug, Copy, Clone)]
pub enum TextureWrap {
    Clamp,
    Repeat,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum TextureType {
    None,
    Diffuse,
    Specular,
    Ambient,
    Emissive,
    Height,
    Normals,
    Shininess,
    Opacity,
    Displacement,
    Lightmap,
    Reflection,
    BaseColor,
    Unknown,
    NormalCamera,
    EmissionColor,
    Metalness,
    Roughness,
    AmbientOcclusion,
    Sheen,
    ClearCoat,
    Transmission,
    Force32bit,
}

impl TextureType {
    pub fn convert_from(r_texture_type: &russimp::material::TextureType) -> Self {
        match r_texture_type {
            russimp::material::TextureType::None => TextureType::None,
            russimp::material::TextureType::Diffuse => TextureType::Diffuse,
            russimp::material::TextureType::Specular => TextureType::Specular,
            russimp::material::TextureType::Ambient => TextureType::Ambient,
            russimp::material::TextureType::Emissive => TextureType::Emissive,
            russimp::material::TextureType::Height => TextureType::Height,
            russimp::material::TextureType::Normals => TextureType::Normals,
            russimp::material::TextureType::Shininess => TextureType::Shininess,
            russimp::material::TextureType::Opacity => TextureType::Opacity,
            russimp::material::TextureType::Displacement => TextureType::Displacement,
            russimp::material::TextureType::LightMap => TextureType::Lightmap,
            russimp::material::TextureType::Reflection => TextureType::Reflection,
            russimp::material::TextureType::BaseColor => TextureType::BaseColor,
            russimp::material::TextureType::NormalCamera => TextureType::NormalCamera,
            russimp::material::TextureType::EmissionColor => TextureType::EmissionColor,
            russimp::material::TextureType::Metalness => TextureType::Metalness,
            russimp::material::TextureType::Roughness => TextureType::Roughness,
            russimp::material::TextureType::AmbientOcclusion => TextureType::AmbientOcclusion,
            russimp::material::TextureType::Unknown => TextureType::Unknown,
            russimp::material::TextureType::Sheen => TextureType::Sheen,
            russimp::material::TextureType::ClearCoat => TextureType::ClearCoat,
            russimp::material::TextureType::Transmission => TextureType::Transmission,
            russimp::material::TextureType::Force32bit => TextureType::Force32bit,
        }
    }
}

impl Display for TextureType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TextureType::Diffuse => write!(f, "texture_diffuse"),
            TextureType::Specular => write!(f, "texture_specular"),
            TextureType::Ambient => write!(f, "texture_ambient"),
            TextureType::Emissive => write!(f, "texture_emissive"),
            TextureType::Normals => write!(f, "texture_normal"),
            TextureType::Height => write!(f, "texture_height"),
            TextureType::Shininess => write!(f, "texture_shininess"),
            TextureType::Opacity => write!(f, "texture_opacity"),
            TextureType::Displacement => write!(f, "texture_displacement"),
            TextureType::Lightmap => write!(f, "texture_lightmap"),
            TextureType::Reflection => write!(f, "texture_reflection"),
            TextureType::BaseColor => write!(f, "texture_basecolor"),
            TextureType::Unknown => write!(f, "texture_unknown"),
            TextureType::None => write!(f, "texture_none"),
            TextureType::NormalCamera => write!(f, "texture_normalcamera"),
            TextureType::EmissionColor => write!(f, "texture_emissioncolor"),
            TextureType::Metalness => write!(f, "texture_metalness"),
            TextureType::Roughness => write!(f, "texture_roughness"),
            TextureType::AmbientOcclusion => write!(f, "texture_ambientocclusion"),
            TextureType::Sheen => write!(f, "texture_sheen"),
            TextureType::ClearCoat => write!(f, "texture_clearcoat"),
            TextureType::Transmission => write!(f, "texture_transmission"),
            TextureType::Force32bit => write!(f, "texture_force32bit"),
        }
    }
}

impl From<TextureType> for aiTextureType {
    fn from(value: TextureType) -> Self {
        match value {
            TextureType::None => 0,
            TextureType::Diffuse => 1,
            TextureType::Specular => 2,
            TextureType::Ambient => 3,
            TextureType::Emissive => 4,
            TextureType::Height => 5,
            TextureType::Normals => 6,
            TextureType::Shininess => 7,
            TextureType::Opacity => 8,
            TextureType::Displacement => 9,
            TextureType::Lightmap => 10,
            TextureType::Reflection => 11,
            TextureType::BaseColor => 12,
            TextureType::NormalCamera => 13,
            TextureType::EmissionColor => 14,
            TextureType::Metalness => 15,
            TextureType::Roughness => 16,
            TextureType::AmbientOcclusion => 17,
            TextureType::Unknown => 18,
            TextureType::Sheen => 19,
            TextureType::ClearCoat => 20,
            TextureType::Transmission => 21,
            TextureType::Force32bit => 2147483647,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextureConfig {
    pub texture_type: TextureType,
    pub filter: TextureFilter,
    pub wrap: TextureWrap,
    pub flip_v: bool,
    pub flip_h: bool,
    pub gamma_correction: bool,
}

impl Default for TextureConfig {
    fn default() -> Self {
        TextureConfig::new()
    }
}

impl TextureConfig {
    pub fn new() -> Self {
        TextureConfig {
            texture_type: TextureType::Diffuse,
            filter: TextureFilter::Linear,
            wrap: TextureWrap::Clamp,
            flip_v: false,
            flip_h: false,
            gamma_correction: false,
        }
    }

    pub fn set_type(mut self, texture_type: TextureType) -> Self {
        self.texture_type = texture_type;
        self
    }

    pub fn set_filter(mut self, filter_type: TextureFilter) -> Self {
        self.filter = filter_type;
        self
    }

    pub fn set_wrap(mut self, wrap_type: TextureWrap) -> Self {
        self.wrap = wrap_type;
        self
    }

    pub fn set_flipv(mut self, flip_v: bool) -> Self {
        self.flip_v = flip_v;
        self
    }

    pub fn set_fliph(mut self, flip_h: bool) -> Self {
        self.flip_h = flip_h;
        self
    }

    pub fn set_gamma_correction(mut self, correct_gamma: bool) -> Self {
        self.gamma_correction = correct_gamma;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Texture {
    pub id: u32,
    pub texture_path: OsString,
    pub texture_type: TextureType,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn new(
        file_path: impl Into<PathBuf>,
        texture_config: &TextureConfig,
    ) -> Result<Texture, Error> {
        let file_path = file_path.into();
        // let (id, width, height) = load_texture(&file_path, texture_config)?;
        // let texture = Texture {
        //     id,
        //     texture_path: file_path.into(),
        //     texture_type: texture_config.texture_type,
        //     width,
        //     height,
        // };
        let texture = Texture {
            id: 0,
            texture_path: file_path.into(),
            texture_type: texture_config.texture_type,
            width: 0,
            height: 0,
        };
        Ok(texture)
    }
}

// pub fn bind_texture(shader: &Shader, texture_unit: i32, uniform_name: &str, texture: &Texture) {
//     unsafe {
//         gl::ActiveTexture(gl::TEXTURE0 + texture_unit as u32);
//         gl::BindTexture(gl::TEXTURE_2D, texture.id);
//         shader.set_int(uniform_name, texture_unit);
//     }
// }

/* opengl texture loading
pub fn load_texture(
    texture_path: &PathBuf,
    texture_config: &TextureConfig,
) -> Result<(GLuint, u32, u32), Error> {
    let mut texture_id: GLuint = 0;

    let img = match image::open(texture_path) {
        Ok(img) => img,
        Err(e) => {
            return Err(ImageError(format!(
                "image error: {:?}  file: {:?}",
                e, texture_path
            )))
        }
    };

    let (width, height) = (img.width() as GLsizei, img.height() as GLsizei);

    let color_type = img.color();

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

    unsafe {
        let internal_format: c_uint;
        let data_format: c_uint;
        match color_type {
            ColorType::L8 => {
                internal_format = gl::RED;
                data_format = gl::RED;
            }
            ColorType::Rgb8 => {
                internal_format = if texture_config.gamma_correction {
                    gl::SRGB
                } else {
                    gl::RGB
                };
                data_format = gl::RGB;
            }
            ColorType::Rgba8 => {
                internal_format = if texture_config.gamma_correction {
                    gl::SRGB_ALPHA
                } else {
                    gl::RGBA
                };
                data_format = gl::RGBA;
            }
            _ => panic!("no mapping for color type"),
        };

        let data = match color_type {
            ColorType::L8 => img.into_rgb8().into_raw(),
            ColorType::Rgb8 => img.into_rgb8().into_raw(),
            ColorType::Rgba8 => img.into_rgba8().into_raw(),
            _ => panic!("no mapping for color type"),
        };

        gl::GenTextures(1, &mut texture_id);
        gl::BindTexture(gl::TEXTURE_2D, texture_id);

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            internal_format as GLint,
            width,
            height,
            0,
            data_format,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const GLvoid,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);

        let wrap_param = match texture_config.wrap {
            TextureWrap::Clamp => gl::CLAMP_TO_EDGE,
            TextureWrap::Repeat => gl::REPEAT,
        };

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap_param as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap_param as GLint);

        match texture_config.filter {
            TextureFilter::Linear => {
                gl::TexParameteri(
                    gl::TEXTURE_2D,
                    gl::TEXTURE_MIN_FILTER,
                    gl::LINEAR_MIPMAP_LINEAR as GLint,
                );
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            }
            TextureFilter::Nearest => {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            }
        }
    }
    Ok((texture_id, width as u32, height as u32))
}
*/
