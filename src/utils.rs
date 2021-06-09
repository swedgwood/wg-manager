use ipnet::Ipv4Net;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize_ipv4net<S>(ipv4net: &Ipv4Net, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ipv4net.to_string().serialize(serializer)
}

pub fn deserialize_ipv4net<'de, D>(deserializer: D) -> Result<Ipv4Net, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)
        .and_then(|x| x.parse::<Ipv4Net>().map_err(|e| D::Error::custom(e)))
}
