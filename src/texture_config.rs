use russimp::sys::aiTextureType;
use std::fmt::{Display, Formatter};

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
