use anyhow::anyhow;
use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;
use zip::ZipArchive;

#[derive(Error, Debug)]
pub enum CreateError {
    #[error("filesystem error: {0}")]
    IoError(anyhow::Error, std::io::Error),
    #[error("unknown template: {0}")]
    UnknownTemplate(String),
    #[error("Connection error: {0}")]
    ConnectionError(#[from] reqwest::Error),
    #[error("Zip error: {0}")]
    ZipError(#[from] zip::result::ZipError),
    #[error("A directory with this name already exists: {0}")]
    DirectoryAlreadyExists(String),
}

#[derive(Debug, Default)]
pub enum Template {
    #[default]
    LocalDefault,
    Default,
    Ft,
    Nft,
}

impl FromStr for Template {
    type Err = CreateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local_default" => Ok(Template::LocalDefault),
            "default" => Ok(Template::Default),
            "ft" => Ok(Template::Ft),
            "nft" => Ok(Template::Nft),
            _ => Err(CreateError::UnknownTemplate(s.to_string())),
        }
    }
}

impl Template {
    fn get_zip_template(&self) -> Result<ZipArchive<Cursor<Vec<u8>>>, CreateError> {
        match self {
            Template::LocalDefault => {
                let content =
                    include_bytes!(concat!(env!("OUT_DIR"), "/default_template.zip")).to_vec();
                let reader = std::io::Cursor::new(content);
                let zip = ZipArchive::new(reader)?;
                Ok(zip)
            }
            Template::Default => {
                let response_body = reqwest::blocking::get("https://github.com/L1X-Foundation/cargo-l1x-templates/archive/refs/heads/default.zip")?.bytes()?;
                let reader = Cursor::new(response_body.to_vec());
                let zip = ZipArchive::new(reader)?;
                Ok(zip)
            }
            Template::Ft => {
                let response_body = reqwest::blocking::get("https://github.com/L1X-Foundation/cargo-l1x-templates/archive/refs/heads/ft.zip")?.bytes()?;
                let reader = Cursor::new(response_body.to_vec());
                let zip = ZipArchive::new(reader)?;
                Ok(zip)
            }
            Template::Nft => {
                let response_body = reqwest::blocking::get("https://github.com/L1X-Foundation/cargo-l1x-templates/archive/refs/heads/nft.zip")?.bytes()?;
                let reader = Cursor::new(response_body.to_vec());
                let zip = ZipArchive::new(reader)?;
                Ok(zip)
            }
        }
    }

    fn unzip(
        archive: &mut ZipArchive<Cursor<Vec<u8>>>,
        destination_path: &PathBuf,
    ) -> Result<(), CreateError> {
        let mut top_level_dir_name = None;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let mut file_path = file.mangled_name();
            if let Some(top_level_dir_name) = top_level_dir_name.as_ref() {
                if let Ok(stripped_file_path) = file_path.strip_prefix(top_level_dir_name) {
                    file_path = stripped_file_path.to_owned();
                }
            }
            let mut path = destination_path.join(file_path);
            let file_name = file.name().to_owned();
            if file.is_dir() {
                if top_level_dir_name.is_none() {
                    top_level_dir_name = Some(file_name);
                    continue; // Skip the top-level directory
                }
                std::fs::create_dir_all(&path).map_err(|e| {
                    CreateError::IoError(
                        anyhow!("Couldn't create a directory: {}", path.display()),
                        e,
                    )
                })?;
            } else {
                let parent = path.parent().unwrap();
                if !parent.exists() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        CreateError::IoError(
                            anyhow!("Couldn't create a directory: {}", parent.display()),
                            e,
                        )
                    })?;
                }
                if path.file_name() == Some("Cargo.toml.template".as_ref()) {
                    path = path.with_file_name("Cargo.toml");
                }
                let mut outfile = File::create(&path).map_err(|e| {
                    CreateError::IoError(anyhow!("Couldn't create a file: {}", path.display()), e)
                })?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| {
                    CreateError::IoError(anyhow!("Couldn't copy file: {}", path.display()), e)
                })?;
            }
        }
        Ok(())
    }
}

pub fn create(name: String, from_template: String) -> Result<(), CreateError> {
    let template = Template::from_str(&from_template)?;

    let destination_path = PathBuf::from(&name);
    if destination_path.exists() {
        return Err(CreateError::DirectoryAlreadyExists(name));
    }

    fs::create_dir_all(name.clone())
        .map_err(|e| CreateError::IoError(anyhow!("Couldn't create a directory: {}", name), e))?;

    let mut archive = template.get_zip_template()?;

    Template::unzip(&mut archive, &destination_path)?;

    Ok(())
}
