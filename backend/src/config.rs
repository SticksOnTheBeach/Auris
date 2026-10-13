use std::env;
use std::net::{AddrParseError, IpAddr, Ipv4Addr, SocketAddr};
use std::num::ParseIntError;

use thiserror::Error;

/// Variable d'environnement portant l'adresse d'écoute.
const BIND_VAR: &str = "AURIS_BIND";
/// Variable d'environnement portant le port d'écoute.
const PORT_VAR: &str = "AURIS_PORT";

/// Configuration invalide. On refuse de démarrer plutôt que de retomber
/// silencieusement sur une valeur par défaut : sur un réseau air-gapped, un
/// serveur qui écoute au mauvais endroit est indétectable.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("{BIND_VAR} : adresse IP invalide ({0})")]
    Address(#[from] AddrParseError),

    #[error("{PORT_VAR} : port invalide ({0})")]
    Port(#[from] ParseIntError),
}

/// Paramètres réseau du serveur.
pub struct ServerConfig {
    pub address: SocketAddr,
}

impl ServerConfig {
    /// Toutes les interfaces : la borne arrive par le LAN, pas par localhost.
    pub const DEFAULT_IP: IpAddr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
    pub const DEFAULT_PORT: u16 = 8080;

    /// Lit `AURIS_BIND` et `AURIS_PORT`, sinon `0.0.0.0:8080`.
    pub fn from_env() -> Result<Self, ConfigError> {
        let ip = match env::var(BIND_VAR) {
            Ok(value) => value.trim().parse::<IpAddr>()?,
            Err(_) => Self::DEFAULT_IP,
        };

        let port = match env::var(PORT_VAR) {
            Ok(value) => value.trim().parse::<u16>()?,
            Err(_) => Self::DEFAULT_PORT,
        };

        Ok(Self {
            address: SocketAddr::new(ip, port),
        })
    }
}
