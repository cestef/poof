use crate::Result;
use facet::Facet;
use std::fs;
use std::path::PathBuf;

pub trait ConfigManager<'a, T>
where
    T: Facet<'a> + Default,
{
    fn config_path(&self) -> PathBuf;

    fn load(&self) -> Result<T> {
        let path = self.config_path();
        if !path.exists() {
            return Ok(T::default());
        }

        let content = fs::read_to_string(&path)?;
        let config = facet_toml::from_str(&content)
            .map_err(|e| crate::error!("Failed to parse config: {e}"))?;
        Ok(config)
    }

    fn save(&self, config: &'a T) -> Result<()> {
        let path = self.config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = facet_toml::to_string(config)?;
        fs::write(&path, content)?;
        Ok(())
    }

    fn exists(&self) -> bool {
        self.config_path().exists()
    }
}
