//! Entity-specific mappers for UTILMD messages.
//!
//! Each mapper implements `SegmentHandler` + `Builder<T>` for one entity type.
//! The coordinator registers all mappers and routes segments to them.

pub mod geschaeftspartner;
pub mod mabis_zaehlpunkt;
pub mod marktlokation;
pub mod messlokation;
pub mod netzlokation;
pub mod prozessdaten;
pub mod steuerbare_ressource;
pub mod technische_ressource;
pub mod tranche;
pub mod vertrag;
pub mod zaehler;
pub mod zeitscheibe;

pub mod produktpaket;

pub use geschaeftspartner::GeschaeftspartnerMapper;
pub use mabis_zaehlpunkt::MabisZaehlpunktMapper;
pub use marktlokation::MarktlokationMapper;
pub use messlokation::MesslokationMapper;
pub use netzlokation::NetzlokationMapper;
pub use prozessdaten::ProzessdatenMapper;
pub use steuerbare_ressource::SteuerbareRessourceMapper;
pub use technische_ressource::TechnischeRessourceMapper;
pub use tranche::TrancheMapper;
pub use vertrag::VertragMapper;
pub use zaehler::ZaehlerMapper;
pub use zeitscheibe::ZeitscheibeMapper;

pub use produktpaket::ProduktpaketMapper;
