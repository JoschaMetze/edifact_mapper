use serde::{Deserialize, Serialize};

/// The canonical BO4E representation stored in `.mig.bo.json` files.
///
/// This is the version-independent business content. Each format version
/// renders this into a different EDIFACT wire format via TOML mappings.
///
/// Structure mirrors `mig_bo4e::model::MappedMessage` with added metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalBo4e {
    /// Metadata about the source fixture.
    pub meta: CanonicalMeta,
    /// Interchange-level data (sender/receiver, date, reference).
    pub nachrichtendaten: serde_json::Value,
    /// Message-level data (UNH reference, type, message-level entities).
    pub nachricht: NachrichtBo4e,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalMeta {
    /// PID that this fixture represents.
    pub pid: String,
    /// Message type (e.g., "UTILMD").
    pub message_type: String,
    /// Original format version this was derived from.
    pub source_format_version: String,
    /// Original fixture filename.
    pub source_fixture: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NachrichtBo4e {
    /// UNH reference number.
    pub unh_referenz: String,
    /// Message subtype (e.g., "UTILMD:D:11A:UN:S2.1").
    pub nachrichten_typ: String,
    /// Message-level entities (Marktteilnehmer, Ansprechpartner).
    pub stammdaten: serde_json::Value,
    /// Transaction-level data (one per SG4 repetition).
    pub transaktionen: Vec<TransaktionBo4e>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransaktionBo4e {
    /// Transaction-level entities (Marktlokation, Messlokation, etc.).
    pub stammdaten: serde_json::Value,
    /// Process data (Prozessdaten, Nachricht).
    pub transaktionsdaten: serde_json::Value,
}
