use super::config::ConfigManager;
use crate::utils::constants::{CONFIG_DIRECTORY, KEYS_FILE};
use crate::{Result, error};
use facet::Facet;
use iroh::{PublicKey, SecretKey};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::SystemTime;

#[derive(Debug, Clone, Facet)]
pub struct Host {
    pub alias: String,
    pub public_key: String,
    pub description: Option<String>,
    pub added_at: u64,
    pub last_seen: Option<u64>,
    pub metadata: HashMap<String, String>,
}

impl Host {
    pub fn new(alias: String, public_key: PublicKey, description: Option<String>) -> Self {
        Self {
            alias,
            public_key: public_key.to_string(),
            description,
            added_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            last_seen: None,
            metadata: HashMap::new(),
        }
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = Some(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey::from_str(&self.public_key).expect("Invalid public key format")
    }

    pub fn added_at(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(self.added_at)
    }
    pub fn last_seen(&self) -> Option<SystemTime> {
        self.last_seen
            .map(|ts| SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(ts))
    }
}

#[derive(Debug, Clone, Facet, Default)]
pub struct HostConfig {
    // alias -> host
    pub hosts: HashMap<String, Host>,
}

impl HostConfig {
    pub fn add_host(&mut self, host: Host) -> Result<()> {
        if self.hosts.contains_key(&host.alias) {
            return Err(error!("Host with alias '{}' already exists", host.alias));
        }

        // Check for duplicate public keys
        for existing_host in self.hosts.values() {
            if existing_host.public_key == host.public_key {
                return Err(error!(
                    "Host with public key '{}' already exists with alias '{}'",
                    host.public_key, existing_host.alias
                ));
            }
        }

        self.hosts.insert(host.alias.clone(), host);
        Ok(())
    }

    pub fn remove_host(&mut self, alias: &str) -> Result<Host> {
        self.hosts
            .remove(alias)
            .ok_or_else(|| error!("Host with alias '{}' not found", alias))
    }

    pub fn get_host(&self, alias: &str) -> Option<&Host> {
        self.hosts.get(alias)
    }

    pub fn get_host_mut(&mut self, alias: &str) -> Option<&mut Host> {
        self.hosts.get_mut(alias)
    }

    pub fn find_by_public_key(&self, public_key: &PublicKey) -> Option<&Host> {
        self.hosts
            .values()
            .find(|host| host.public_key == public_key.to_string())
    }

    pub fn list_hosts(&self) -> Vec<&Host> {
        self.hosts.values().collect()
    }

    pub fn update_host_alias(&mut self, old_alias: &str, new_alias: String) -> Result<()> {
        if self.hosts.contains_key(&new_alias) {
            return Err(error!("Host with alias '{}' already exists", new_alias));
        }

        let mut host = self.remove_host(old_alias)?;
        host.alias = new_alias;
        self.add_host(host)
    }
}

#[derive(Debug, Clone, Facet)]
pub struct HostKey {
    pub name: String,
    pub secret_key: String,
    pub created_at: u64,
    pub description: Option<String>,
}

impl HostKey {
    pub fn new(name: String, secret_key: SecretKey, description: Option<String>) -> Self {
        Self {
            name,
            secret_key: secret_key.to_string(),
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            description,
        }
    }

    pub fn public_key(&self) -> PublicKey {
        self.secret_key().public()
    }

    pub fn secret_key(&self) -> SecretKey {
        SecretKey::from_str(&self.secret_key).expect("Invalid secret key format")
    }

    pub fn created_at(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(self.created_at)
    }
}

#[derive(Debug, Clone, Facet, Default)]
pub struct KeyConfig {
    /// Map of key name -> HostKey
    pub keys: HashMap<String, HostKey>,
    /// Name of the default key to use
    pub default_key: Option<String>,
}

impl KeyConfig {
    pub fn add_key(&mut self, key: HostKey) -> Result<()> {
        if self.keys.contains_key(&key.name) {
            return Err(error!("Key with name '{}' already exists", key.name));
        }

        self.keys.insert(key.name.clone(), key.clone());

        // Set as default if it's the first key
        if self.default_key.is_none() {
            self.default_key = Some(key.name);
        }

        Ok(())
    }

    pub fn remove_key(&mut self, name: &str) -> Result<HostKey> {
        let key = self
            .keys
            .remove(name)
            .ok_or_else(|| error!("Key with name '{}' not found", name))?;

        // Update default if we removed it
        if self.default_key.as_ref() == Some(&key.name) {
            self.default_key = self.keys.keys().next().map(|k| k.clone());
        }

        Ok(key)
    }

    pub fn get_key(&self, name: &str) -> Option<&HostKey> {
        self.keys.get(name)
    }

    pub fn get_default_key(&self) -> Option<&HostKey> {
        self.default_key
            .as_ref()
            .and_then(|name| self.keys.get(name))
    }

    pub fn set_default_key(&mut self, name: String) -> Result<()> {
        if !self.keys.contains_key(&name) {
            return Err(error!("Key with name '{}' not found", name));
        }
        self.default_key = Some(name);
        Ok(())
    }

    pub fn list_keys(&self) -> Vec<&HostKey> {
        self.keys.values().collect()
    }
}

pub struct HostManager;

impl ConfigManager<'_, HostConfig> for HostManager {
    fn config_path(&self) -> PathBuf {
        CONFIG_DIRECTORY.join("hosts.toml")
    }
}

impl HostManager {
    pub fn new() -> Self {
        Self
    }

    pub fn add_host(
        &self,
        alias: String,
        public_key: PublicKey,
        description: Option<String>,
    ) -> Result<()> {
        let mut config = self.load()?;
        let host = Host::new(alias, public_key, description);
        config.add_host(host)?;
        self.save(&config)
    }

    pub fn remove_host(&self, alias: &str) -> Result<Host> {
        let mut config = self.load()?;
        let host = config.remove_host(alias)?;
        self.save(&config)?;
        Ok(host)
    }

    pub fn list_hosts(&self) -> Result<Vec<Host>> {
        let config = self.load()?;
        Ok(config.list_hosts().into_iter().cloned().collect())
    }

    pub fn get_host(&self, alias: &str) -> Result<Option<Host>> {
        let config = self.load()?;
        Ok(config.get_host(alias).cloned())
    }

    pub fn find_by_public_key(&self, public_key: &PublicKey) -> Result<Option<Host>> {
        let config = self.load()?;
        Ok(config.find_by_public_key(public_key).cloned())
    }

    pub fn update_last_seen(&self, alias: &str) -> Result<()> {
        let mut config = self.load()?;
        if let Some(host) = config.get_host_mut(alias) {
            host.update_last_seen();
            self.save(&config)?;
        }
        Ok(())
    }

    pub fn rename_host(&self, old_alias: &str, new_alias: String) -> Result<()> {
        let mut config = self.load()?;
        config.update_host_alias(old_alias, new_alias)?;
        self.save(&config)
    }
}

pub struct KeyManager;

impl ConfigManager<'_, KeyConfig> for KeyManager {
    fn config_path(&self) -> PathBuf {
        CONFIG_DIRECTORY.join(KEYS_FILE)
    }
}

impl KeyManager {
    pub fn new() -> Self {
        Self
    }

    pub fn add_key(
        &self,
        name: String,
        secret_key: SecretKey,
        description: Option<String>,
    ) -> Result<()> {
        let mut config = self.load()?;
        let key = HostKey::new(name, secret_key, description);
        config.add_key(key)?;
        self.save(&config)
    }

    pub fn remove_key(&self, name: &str) -> Result<HostKey> {
        let mut config = self.load()?;
        let key = config.remove_key(name)?;
        self.save(&config)?;
        Ok(key)
    }

    pub fn get_key(&self, name: &str) -> Result<Option<HostKey>> {
        let config = self.load()?;
        Ok(config.get_key(name).cloned())
    }

    pub fn get_default_key(&self) -> Result<Option<HostKey>> {
        let config = self.load()?;
        Ok(config.get_default_key().cloned())
    }

    pub fn set_default_key(&self, name: &str) -> Result<()> {
        let mut config = self.load()?;
        config.set_default_key(name.to_string())?;
        self.save(&config)
    }

    pub fn list_keys(&self) -> Result<Vec<HostKey>> {
        let config = self.load()?;
        Ok(config.list_keys().into_iter().cloned().collect())
    }

    pub fn generate_key(&self, name: String, description: Option<String>) -> Result<HostKey> {
        use rand::rngs::OsRng;
        let secret_key = SecretKey::generate(&mut OsRng);
        self.add_key(name.clone(), secret_key.clone(), description)?;
        Ok(HostKey::new(name, secret_key, None))
    }
}
