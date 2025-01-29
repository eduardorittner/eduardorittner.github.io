use crate::link::Link;

pub struct InvalidLinks(pub Vec<Link>);

pub enum BuildError {
    InvalidLinks(InvalidLinks),
    IoError(std::io::Error),
}

impl std::fmt::Debug for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::InvalidLinks(links) => {
                for link in links.0.iter() {
                    write!(
                        f,
                        "\nInvalid link: {} from file: {:?}",
                        link.link, link.file
                    )?;
                }
                Ok(())
            }
            BuildError::IoError(e) => write!(f, "{:?}", e),
        }
    }
}
