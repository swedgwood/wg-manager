use ipnet::Ipv4Net;
use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
    io::Write,
    net::{Ipv4Addr, SocketAddrV4},
    path::Path,
};

use crate::utils::{deserialize_ipv4net, serialize_ipv4net};
use crate::wg::Wg;

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
            &ManagerError::ClientNameExistsError(name) => {
                format!("client with name '{}' already exists", name)
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Manager {
    interface_name: String,
    private_key: String,
    public_key: String,
    endpoint: SocketAddrV4,

    #[serde(
        serialize_with = "serialize_ipv4net",
        deserialize_with = "deserialize_ipv4net"
    )]
    ip_range: Ipv4Net,

    clients: HashMap<String, Client>,
    wg: Wg,
}

impl Manager {
    pub fn new(endpoint: SocketAddrV4, ip_range: Ipv4Net, interface_name: String) -> Self {
        let wg = Wg::new("wg".into());
        let private_key = wg.genkey();
        let public_key = wg.pubkey(&private_key);

        Manager {
            interface_name,
            private_key,
            public_key,
            endpoint,
            ip_range,
            clients: HashMap::new(),
            wg,
        }
    }

    /// Produces `Manager` struct from the contents of a config file
    ///
    /// NOTE: `from_config` and `save_config` do not handle file locking
    pub fn from_config(path: &Path) -> Result<Self, ManagerError> {
        let data = std::fs::read(path)?;
        let manager: Manager = serde_json::from_slice(&data)?;
        Ok(manager)
    }

    /// Commits changes to WireGuard interface
    pub fn commit(&self) {
        // Check values for server
        let private_key = self.wg.show_private_key(&self.interface_name);
        let listen_port = self.wg.show_listen_port(&self.interface_name);

        // Check values for peers (clients)
        // TODO: Ipv4Net parsing may fail, since there may be comma-separate ip ranges, but I haven't checked this yet
        let mut peers_allowed_ips: HashMap<String, Ipv4Net> = self
            .wg
            .show_allowed_ips(&self.interface_name)
            .into_iter()
            .map(|mut row| (row.pop().unwrap(), row.pop().unwrap().parse().unwrap()))
            .collect();

        // Check for differences and apply
        // Note that checking for public key is not needed, as this is derived from private key
        if private_key != self.private_key {
            // TODO: error handling
            let mut temp_file = tempfile::NamedTempFile::new().unwrap();
            writeln!(temp_file, "{}", self.private_key).unwrap();
            self.wg
                .set_private_key(&self.interface_name, temp_file.path());
            temp_file.close().unwrap();
        }
        if listen_port != self.endpoint().port() {
            self.wg
                .set_listen_port(&self.interface_name, self.endpoint().port())
        }
        // TODO: check/update listen ip????

        for (_name, client) in &self.clients {
            let pubkey = client.public_key();
            if let Some(current_allowed_ips) = peers_allowed_ips.remove(pubkey) {
                todo!("compare allowed ips, if not the same, update them");
            } else {
                todo!("add client")
            }
        }

        for (public_key, _allowed_ips) in peers_allowed_ips {
            todo!("remove public key")
        }

        todo!();
    }

    /// Save `Manager` struct to the contents of a config file
    ///
    /// NOTE: `from_config` and `save_config` do not handle file locking
    pub fn save_config(self, path: &Path) -> Result<(), ManagerError> {
        let data = serde_json::to_vec(&self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    // Creates new client and returns private key
    pub fn new_client(
        &mut self,
        name: String,
        ip: Ipv4Addr,
    ) -> Result<(&Client, String), ManagerError> {
        if self.clients.contains_key(&name) {
            Err(ManagerError::ClientNameExistsError(name))
        } else {
            let private_key = self.wg.genkey();
            let public_key = self.wg.pubkey(&private_key);

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
