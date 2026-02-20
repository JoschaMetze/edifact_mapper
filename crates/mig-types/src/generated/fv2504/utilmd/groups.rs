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
    pub com: Vec<SegCom>,
    pub cta: SegCta,
}

/// SG4 — Identifikation einer Liste
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sg4 {
    pub agr: Vec<SegAgr>,
    pub dtm: Vec<SegDtm>,
    pub ftx: Vec<SegFtx>,
    pub ide: SegIde,
    pub sts: Vec<SegSts>,
    pub sg12: Vec<Sg12>,
    pub sg5: Vec<Sg5>,
    pub sg6: Vec<Sg6>,
    pub sg8: Vec<Sg8>,
}

/// SG12 — Kunde des Lieferanten
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sg12 {
    pub nad: SegNad,
    pub rff: Vec<SegRff>,
}

/// SG5 — MaBiS-Zählpunkt
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sg5 {
    pub loc: SegLoc,
}

/// SG6 — Prüfidentifikator
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sg6 {
    pub dtm: Vec<SegDtm>,
    pub rff: SegRff,
}

/// SG8 — Daten der Summenzeitreihe
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sg8 {
    pub pia: Vec<SegPia>,
    pub rff: Vec<SegRff>,
    pub seq: SegSeq,
    pub sg10: Vec<Sg10>,
    pub sg9: Vec<Sg9>,
}

/// SG10 — Bilanzkreis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sg10 {
    pub cav: Vec<SegCav>,
    pub cci: SegCci,
}

/// SG9 — Arbeit / Leistung für tagesparameterabhängige Marktlokation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sg9 {
    pub dtm: Vec<SegDtm>,
    pub qty: SegQty,
}
