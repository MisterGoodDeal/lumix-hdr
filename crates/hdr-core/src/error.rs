use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum HdrError {
    #[error("OpenCV: {0}")]
    OpenCv(String),
    #[error("lecture impossible ({path}): {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("aucune image JPEG dans le dossier")]
    NoImages,
    #[error("au moins deux images sont nécessaires (trouvé : {0})")]
    TooFewImages(usize),
    #[error("images incompatibles: {0}")]
    Inconsistent(String),
    #[error("fusion annulée")]
    Cancelled,
    #[error("impossible d'écrire {path}: {detail}")]
    Write { path: PathBuf, detail: String },
    #[error("EXIF: {0}")]
    Exif(String),
}

impl From<opencv::Error> for HdrError {
    fn from(error: opencv::Error) -> Self {
        Self::OpenCv(error.to_string())
    }
}
