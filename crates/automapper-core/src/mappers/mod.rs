//! Entity-specific mappers for UTILMD messages.
//!
//! Each mapper implements `SegmentHandler` + `Builder<T>` for one entity type.
//! The coordinator registers all mappers and routes segments to them.

pub mod geschaeftspartner;
pub mod marktlokation;
pub mod messlokation;
pub mod netzlokation;
pub mod prozessdaten;
pub mod vertrag;
pub mod zaehler;
pub mod zeitscheibe;

pub use geschaeftspartner::GeschaeftspartnerMapper;
pub use marktlokation::MarktlokationMapper;
pub use messlokation::MesslokationMapper;
pub use netzlokation::NetzlokationMapper;
pub use prozessdaten::ProzessdatenMapper;
pub use vertrag::VertragMapper;
pub use zaehler::ZaehlerMapper;
pub use zeitscheibe::ZeitscheibeMapper;
