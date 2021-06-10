use ipnet::Ipv4Net;
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, net::{Ipv4Addr, SocketAddrV4}, path::Path};

use crate::utils::{deserialize_ipv4net, serialize_ipv4net};
use crate::wg::{wg_genkey, wg_pubkey};

#[derive(Debug)]
pub enum ManagerError {
    IOError(std::io::Error),
    SerializationError(serde_json::Error),
    ClientNameExistsError(String),
}

impl From<std::io::Error> for ManagerError {
    fn from(e: std::io::Error) -> Self {
        ManagerError::IOError(e)
    }
}

impl From<serde_json::Error> for ManagerError {
    fn from(e: serde_json::Error) -> Self {
        ManagerError::SerializationError(e)
    }
}

impl ToString for ManagerError {
    fn to_string(&self) -> String {
        match &self {
            &ManagerError::IOError(e) => e.to_string(),
            &ManagerError::SerializationError(e) => e.to_string(),
            &ManagerError::ClientNameExistsError(name) => format!("client with name '{}' already exists", name),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Manager {
    private_key: String,
    public_key: String,
    endpoint: SocketAddrV4,

    #[serde(
        serialize_with = "serialize_ipv4net",
        deserialize_with = "deserialize_ipv4net"
    )]
    ip_range: Ipv4Net,

    clients: HashMap<String, Client>,
}

impl Manager {
    pub fn new(endpoint: SocketAddrV4, ip_range: Ipv4Net) -> Self {
        let private_key = wg_genkey();
        let public_key = wg_pubkey(&private_key);

        Manager {
            private_key,
            public_key,
            endpoint,
            ip_range,
            clients: HashMap::new(),
        }
    }

    pub fn from_config(path: &Path) -> Result<Self, ManagerError> {
        let data = std::fs::read(path)?;
        let manager: Manager = serde_json::from_slice(&data)?;
        Ok(manager)
    }

    pub fn save_config(&self, path: &Path) -> Result<(), ManagerError> {
        let data = serde_json::to_vec(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    // Creates new client and returns private key
    pub fn new_client(&mut self, name: String, ip: Ipv4Addr) -> Result<(&Client, String), ManagerError> {
        if self.clients.contains_key(&name) {
            Err(ManagerError::ClientNameExistsError(name))
        } else {
            let private_key = wg_genkey();
            let public_key = wg_pubkey(&private_key);

            let client = Client {
                name: name.clone(),
                public_key,
                ip,
            };

            self.clients.insert(name.clone(), client);
            Ok((self.clients.get(&name).unwrap(), private_key))
        }
    }

    pub fn clients(&self) -> Vec<&Client> {
        self.clients.values().into_iter().collect()
    }

    pub fn endpoint(&self) -> SocketAddrV4 {
        self.endpoint
    }
}

#[derive(Serialize, Deserialize)]
pub struct Client {
    name: String,
    public_key: String,
    ip: Ipv4Addr,
}

impl Client {
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn public_key(&self) -> &String {
        &self.public_key
    }
}
