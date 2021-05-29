use serde::{Deserialize, Serialize};

use std::{net::Ipv4Addr, path::Path};

use crate::wg::{wg_genkey, wg_pubkey};

#[derive(Serialize, Deserialize)]
pub struct Manager {
    private_key: String,
    public_key: String,

    ip: std::net::Ipv4Addr,
    subnet_bits: u8,

    clients: Vec<Client>,
}

impl Manager {
    pub fn new(ip: Ipv4Addr, subnet_bits: u8) -> Self {
        let private_key = wg_genkey();
        let public_key = wg_pubkey(&private_key);

        Manager {
            private_key,
            public_key,
            ip,
            subnet_bits,
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
    fn new_client(&mut self, familiar_name: String, ip: Ipv4Addr) -> String {
        let private_key = wg_genkey();
        let public_key = wg_pubkey(&private_key);

        // TODO: need to check name is unique
        let client = Client {
            familiar_name,
            public_key,
            ip,
        };

        self.clients.push(client);
        private_key
    }
}

#[derive(Serialize, Deserialize)]
struct Client {
    familiar_name: String,
    public_key: String,
    ip: Ipv4Addr,
}
