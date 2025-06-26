use crate::{
    Result,
    utils::constants::{CONFIG_DIRECTORY, DEFAULT_KEY_NAME, KEYS_FILE},
};
use facet::Facet;

#[derive(Facet)]
pub struct KeysManager {
    config: PoofKeysConfig,
}

impl KeysManager {
    pub async fn load() -> Result<Self> {
        let path = CONFIG_DIRECTORY.join(KEYS_FILE);
        let config = if path.exists() {
            let content = tokio::fs::read_to_string(&path).await?;
            facet_toml::from_str(&content)
                .map_err(|e| crate::error!("Failed to parse keys configuration: {}", e))?
        } else {
            PoofKeysConfig::default()
        };

        let mut self_ = Self { config };

        if self_.config.keys.is_empty() {
            self_.init().await?;
        }

        Ok(self_)
    }

    async fn init(&mut self) -> Result<()> {
        if !self.config.keys.iter().any(|k| k.name == DEFAULT_KEY_NAME) {
            let default_key = KeyEntry::default();
            default_key
                .save(&iroh::SecretKey::generate(&mut rand::rngs::OsRng))
                .await
                .map_err(|e| crate::error!("Failed to save default key: {}", e))?;
            self.config.keys.push(default_key);
        }

        if self.config.default.is_none() {
            self.config.default = Some(DEFAULT_KEY_NAME.to_string());
        }

        self.save().await?;
        tracing::info!(
            "Initialized keys configuration with default key: {}",
            DEFAULT_KEY_NAME
        );

        Ok(())
    }

    pub async fn save(&self) -> Result<()> {
        let path = CONFIG_DIRECTORY.join(KEYS_FILE);
        if !path.exists() {
            tokio::fs::create_dir_all(&*CONFIG_DIRECTORY).await?;
        }
        tokio::fs::write(&path, facet_toml::to_string(&self.config)?).await?;
        Ok(())
    }

    pub fn get_default_key(&self) -> Option<KeyEntry> {
        self.config.default.as_ref().and_then(|name| {
            self.config
                .keys
                .iter()
                .find(|key| key.name == *name)
                .cloned()
        })
    }

    pub fn get_key(&self, name: &str) -> Option<KeyEntry> {
        self.config
            .keys
            .iter()
            .find(|key| key.name == name)
            .cloned()
    }

    pub async fn add_key(&mut self, key: KeyEntry) -> Result<()> {
        if self.config.keys.iter().any(|k| k.name == key.name) {
            return Err(crate::error!("Key with name '{}' already exists", key.name));
        }
        self.config.keys.push(key);
        self.save().await?;
        Ok(())
    }

    pub async fn remove_key(&mut self, name: &str) -> Result<()> {
        if let Some(pos) = self.config.keys.iter().position(|k| k.name == name) {
            self.config.keys.remove(pos);
            self.save().await?;
            Ok(())
        } else {
            Err(crate::error!("Key with name '{}' not found", name))
        }
    }

    pub fn keys(&self) -> &[KeyEntry] {
        &self.config.keys
    }

    pub fn default_key_name(&self) -> Option<&str> {
        self.config.default.as_deref()
    }
}

#[derive(Debug, Clone, Facet)]
pub struct KeyEntry {
    pub name: String,
    pub path: String,
}

impl KeyEntry {
    pub fn new(name: String) -> Self {
        let path = CONFIG_DIRECTORY.join("keys").join(format!("{}.key", name));
        Self {
            name,
            path: path.to_string_lossy().to_string(),
        }
    }

    pub async fn load(&self) -> Result<iroh::SecretKey> {
        let content = tokio::fs::read_to_string(&self.path).await?;
        let secret_key: iroh::SecretKey = content.parse()?;
        Ok(secret_key)
    }

    pub async fn save(&self, sk: &iroh::SecretKey) -> Result<()> {
        let content = sk.to_string();
        if let Some(parent) = std::path::Path::new(&self.path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&self.path, content).await?;
        Ok(())
    }
}

impl Default for KeyEntry {
    fn default() -> Self {
        let path = CONFIG_DIRECTORY
            .join("keys")
            .join(format!("{}.key", DEFAULT_KEY_NAME));
        Self {
            name: DEFAULT_KEY_NAME.to_string(),
            path: path.to_string_lossy().to_string(),
        }
    }
}

#[derive(Debug, Clone, Facet, Default)]
pub struct PoofKeysConfig {
    keys: Vec<KeyEntry>,
    default: Option<String>,
}
