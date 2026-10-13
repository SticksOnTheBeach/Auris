//! Auris : serveur de pointage d'évacuation par badges RFID.
//!
//! Le code est découpé en quatre couches, de la plus abstraite à la plus
//! concrète. Chaque couche ne dépend que de la précédente :
//!
//! - [`domain`] : les notions métier (badge, borne, statut de scan, rapport
//!   d'évacuation). Ne dépend de rien.
//! - [`repository`] : les interfaces d'accès aux données et, pour l'instant,
//!   leur implémentation en mémoire.
//! - [`service`] : les règles métier. Dépend des interfaces de dépôt, jamais
//!   de leur implémentation.
//! - [`api`] : le routage HTTP, les formats d'échange et la traduction des
//!   erreurs. Seule couche à connaître axum.
//!
//! [`config`] est transverse et ne contient que la lecture des paramètres
//! réseau. L'assemblage de tout cela vit dans `main.rs`.

pub mod api;
pub mod config;
pub mod domain;
pub mod repository;
pub mod service;
