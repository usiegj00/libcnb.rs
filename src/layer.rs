use crate::{data::layer::Layer as ContentMetadata, Error};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// CNB Layer
pub struct Layer {
    pub name: String,
    path: PathBuf,
    content_metadata_path: PathBuf,
    content_metadata: ContentMetadata,
}

impl Layer {
    /// Layer Constructor that makes a ready to go layer:
    /// * create `/<layers_dir>/<layer> if it doesn't exist
    /// * `/<layers_dir>/<layer>.toml` will be read and parsed from disk if found. If not found an
    /// empty [`crate::data::layer::Layer`] will be constructed.
    ///
    /// # Errors
    /// This function will return an error when:
    /// * if it can not create the layer dir
    /// * if it can not deserialize Layer Content Metadata to [`crate::data::layer::Layer`]
    ///
    /// # Examples
    /// ```
    /// # use tempfile::tempdir;
    /// use libcnb::layer::Layer;
    ///
    /// # fn main() -> Result<(), libcnb::Error> {
    /// # let layers_dir = tempdir().unwrap().path().to_owned();
    /// let layer = Layer::new("foo", layers_dir)?;
    ///
    /// assert!(layer.as_path().exists());
    /// assert_eq!(layer.content_metadata().launch, false);
    /// assert_eq!(layer.content_metadata().build, false);
    /// assert_eq!(layer.content_metadata().cache, false);
    /// assert!(layer.content_metadata().metadata.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(name: impl Into<String>, layers_dir: impl AsRef<Path>) -> Result<Self, Error> {
        let name = name.into();
        let layers_dir = layers_dir.as_ref();
        let path = layers_dir.join(&name);

        fs::create_dir_all(&path)?;

        let content_metadata_path = layers_dir.join(format!("{}.toml", &name));
        let content_metadata = if let Ok(contents) = fs::read_to_string(&content_metadata_path) {
            toml::from_str(&contents)?
        } else {
            ContentMetadata::new()
        };

        Ok(Layer {
            name,
            path,
            content_metadata,
            content_metadata_path,
        })
    }

    /// Returns the path to the layer contents `/<layers_dir>/<layer>/`.
    pub fn as_path(&self) -> &Path {
        self.path.as_path()
    }

    /// Returns a reference to the [`crate::data::layer::Layer`]
    pub fn content_metadata(&self) -> &ContentMetadata {
        &self.content_metadata
    }

    /// Returns a mutable reference to the [`crate::data::layer::Layer`]
    pub fn mut_content_metadata(&mut self) -> &mut ContentMetadata {
        &mut self.content_metadata
    }

    /// Write [`crate::data::layer::Layer`] to `<layer>.toml`
    pub fn write_content_metadata(&self) -> Result<(), crate::Error> {
        fs::write(
            &self.content_metadata_path,
            toml::to_string(&self.content_metadata)?,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn new_reads_layer_toml_metadata() -> Result<(), anyhow::Error> {
        let layers_dir = tempdir()?.path().to_owned();
        fs::create_dir_all(&layers_dir)?;
        fs::write(
            layers_dir.join("foo.toml"),
            r#"
[metadata]
bar = "baz"
"#,
        )?;

        let layer = Layer::new("foo", &layers_dir)?;
        assert_eq!(
            layer
                .content_metadata()
                .metadata
                .get::<str>("bar")
                .unwrap()
                .as_str()
                .unwrap(),
            "baz"
        );

        Ok(())
    }
}
