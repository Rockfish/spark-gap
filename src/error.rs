#[derive(Debug)]
pub enum Error {
    PathError(String),
    FileError(std::io::Error),
    ShaderError(String),
    ImageError(String),
    ModelError(russimp::RussimpError),
    SceneError(String),
    MeshError(String),
    TextureError(String),
    UnknownError(&'static str),
}

impl From<std::io::Error> for Error {
    fn from(s: std::io::Error) -> Self {
        Error::FileError(s)
    }
}

impl From<image::ImageError> for Error {
    fn from(s: image::ImageError) -> Self {
        Error::ImageError(format!("{:?}", s))
    }
}

impl From<russimp::RussimpError> for Error {
    fn from(s: russimp::RussimpError) -> Self {
        Error::ModelError(s)
    }
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error::UnknownError(s)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {:?}", self)
    }
}
