use ipnet::Ipv4Net;
use serde::{Deserialize, Serialize};

use std::{
    net::{Ipv4Addr, SocketAddrV4},
    path::Path,
};

use crate::utils::{deserialize_ipv4net, serialize_ipv4net};
use crate::wg::{wg_genkey, wg_pubkey};

#[derive(Serialize, Deserialize)]
pub struct Manager {
    private_key: String,
    public_key: String,
    endpoint: SocketAddrV4,

    // (ip_range, subnet_bits), i.e. 10.33.7.0/24 => (10.33.7.0, 24)
    #[serde(
        serialize_with = "serialize_ipv4net",
        deserialize_with = "deserialize_ipv4net"
    )]
    ip_range: Ipv4Net,

    clients: Vec<Client>,
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
            clients: Vec::new(),
        }
    }

    pub fn from_config(path: &Path) -> std::io::Result<Self> {
        let data = std::fs::read(path)?;
        let manager: Manager = serde_json::from_slice(&data)?;
        Ok(manager)
    }

    pub fn save_config(&self, path: &Path) -> std::io::Result<()> {
        let data = serde_json::to_vec(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    // Creates new client and returns private key
    pub fn new_client(&mut self, name: String, ip: Ipv4Addr) -> (&Client, String) {
        let private_key = wg_genkey();
        let public_key = wg_pubkey(&private_key);

        // TODO: need to check name is unique
        let client = Client {
            name,
            public_key,
            ip,
        };

        self.clients.push(client);
        let client = self.clients.last().unwrap();
        (client, private_key)
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
    pub fn public_key(&self) -> &String {
        &self.public_key
    }
}
