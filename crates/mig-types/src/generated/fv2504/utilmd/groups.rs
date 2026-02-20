//! Auto-generated segment group structs from MIG XML.
//! Do not edit manually.

use super::segments::*;
use serde::{Deserialize, Serialize};

/// SG1 — Referenz auf eine vorangegangene Anfrage
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sg1 {
    pub rff: SegRff,
}

/// SG2 — MP-ID Absender
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sg2 {
    pub nad: SegNad,
    pub sg3: Vec<Sg3>,
}

/// SG3 — Kontaktinformationen
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sg3 {
    pub cta: SegCta,
    pub com: Vec<SegCom>,
}

/// SG4 — Identifikation einer Liste
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sg4 {
    pub ide: SegIde,
    pub sts: Vec<SegSts>,
    pub sg5: Vec<Sg5>,
    pub sg6: Vec<Sg6>,
    pub sg8: Vec<Sg8>,
}

/// SG5 — MaBiS-Zählpunkt
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sg5 {
    pub loc: SegLoc,
}

/// SG6 — Prüfidentifikator
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sg6 {
    pub rff: SegRff,
}

/// SG8 — Daten der Summenzeitreihe
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sg8 {
    pub seq: SegSeq,
    pub rff: Vec<SegRff>,
    pub sg10: Vec<Sg10>,
}

/// SG10 — Bilanzkreis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sg10 {
    pub cci: SegCci,
}
