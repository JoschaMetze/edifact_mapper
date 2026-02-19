//! UTILMD-specific coordinator that orchestrates all mappers.
//!
//! Implements `EdifactHandler` and `Coordinator`. Routes segments to
//! registered mappers and collects built objects into `UtilmdTransaktion`.
//!
//! Mirrors C# `UtilmdCoordinator.cs`.

use std::marker::PhantomData;

use bo4e_extensions::{
    LinkRegistry, Marktteilnehmer, Nachrichtendaten, UtilmdNachricht, UtilmdTransaktion,
};
use edifact_parser::EdifactHandler;
use edifact_types::{Control, EdifactDelimiters, RawSegment};

use crate::context::TransactionContext;
use crate::coordinator::Coordinator;
use crate::error::AutomapperError;
use crate::mappers::prozessdaten::parse_edifact_dtm;
use crate::mappers::*;
use crate::traits::{Builder, FormatVersion, SegmentHandler};
use crate::version::VersionConfig;
use crate::writer::{
    BilanzierungWriter, EdifactDocumentWriter, GeschaeftspartnerWriter, LokationszuordnungWriter,
    MabisZaehlpunktWriter, MarktlokationWriter, MesslokationWriter, NetzlokationWriter,
    ProduktpaketWriter, ProzessdatenWriter, SteuerbareRessourceWriter, TechnischeRessourceWriter,
    TrancheWriter, VertragWriter, ZaehlerWriter, ZeitscheibeWriter,
};

/// UTILMD coordinator that orchestrates all entity mappers.
///
/// Generic over `V: VersionConfig` for compile-time mapper dispatch.
/// Implements `EdifactHandler` for the streaming parser and `Coordinator`
/// for the high-level parse/generate API.
pub struct UtilmdCoordinator<V: VersionConfig> {
    context: TransactionContext,
    #[allow(dead_code)]
    link_registry: LinkRegistry,

    // Mappers
    prozessdaten_mapper: ProzessdatenMapper,
    zeitscheibe_mapper: ZeitscheibeMapper,
    marktlokation_mapper: MarktlokationMapper,
    messlokation_mapper: MesslokationMapper,
    netzlokation_mapper: NetzlokationMapper,
    geschaeftspartner_mapper: GeschaeftspartnerMapper,
    vertrag_mapper: VertragMapper,
    zaehler_mapper: ZaehlerMapper,
    steuerbare_ressource_mapper: SteuerbareRessourceMapper,
    technische_ressource_mapper: TechnischeRessourceMapper,
    tranche_mapper: TrancheMapper,
    mabis_zaehlpunkt_mapper: MabisZaehlpunktMapper,
    bilanzierung_mapper: BilanzierungMapper,
    produktpaket_mapper: ProduktpaketMapper,
    lokationszuordnung_mapper: LokationszuordnungMapper,

    // Collected transactions
    transactions: Vec<UtilmdTransaktion>,

    // Nachrichtendaten from service segments
    nachrichtendaten: Nachrichtendaten,
    absender: Marktteilnehmer,
    empfaenger: Marktteilnehmer,

    // Current transaction state
    in_transaction: bool,
    current_transaction_id: Option<String>,

    _version: PhantomData<V>,
}

impl<V: VersionConfig> UtilmdCoordinator<V> {
    /// Creates a new UtilmdCoordinator.
    pub fn new() -> Self {
        Self {
            context: TransactionContext::new(V::VERSION.as_str()),
            link_registry: LinkRegistry::new(),
            prozessdaten_mapper: ProzessdatenMapper::new(),
            zeitscheibe_mapper: ZeitscheibeMapper::new(),
            marktlokation_mapper: MarktlokationMapper::new(),
            messlokation_mapper: MesslokationMapper::new(),
            netzlokation_mapper: NetzlokationMapper::new(),
            geschaeftspartner_mapper: GeschaeftspartnerMapper::new(),
            vertrag_mapper: VertragMapper::new(),
            zaehler_mapper: ZaehlerMapper::new(),
            steuerbare_ressource_mapper: SteuerbareRessourceMapper::new(),
            technische_ressource_mapper: TechnischeRessourceMapper::new(),
            tranche_mapper: TrancheMapper::new(),
            mabis_zaehlpunkt_mapper: MabisZaehlpunktMapper::new(),
            bilanzierung_mapper: BilanzierungMapper::new(),
            produktpaket_mapper: ProduktpaketMapper::new(),
            lokationszuordnung_mapper: LokationszuordnungMapper::new(),
            transactions: Vec::new(),
            nachrichtendaten: Nachrichtendaten::default(),
            absender: Marktteilnehmer::default(),
            empfaenger: Marktteilnehmer::default(),
            in_transaction: false,
            current_transaction_id: None,
            _version: PhantomData,
        }
    }

    /// Routes a segment to all mappers that can handle it.
    fn route_to_mappers(&mut self, segment: &RawSegment) {
        if self.prozessdaten_mapper.can_handle(segment) {
            self.prozessdaten_mapper.handle(segment, &mut self.context);
        }
        if self.zeitscheibe_mapper.can_handle(segment) {
            self.zeitscheibe_mapper.handle(segment, &mut self.context);
        }
        if self.marktlokation_mapper.can_handle(segment) {
            self.marktlokation_mapper.handle(segment, &mut self.context);
        }
        if self.messlokation_mapper.can_handle(segment) {
            self.messlokation_mapper.handle(segment, &mut self.context);
        }
        if self.netzlokation_mapper.can_handle(segment) {
            self.netzlokation_mapper.handle(segment, &mut self.context);
        }
        if self.geschaeftspartner_mapper.can_handle(segment) {
            self.geschaeftspartner_mapper
                .handle(segment, &mut self.context);
        }
        if self.vertrag_mapper.can_handle(segment) {
            self.vertrag_mapper.handle(segment, &mut self.context);
        }
        if self.zaehler_mapper.can_handle(segment) {
            self.zaehler_mapper.handle(segment, &mut self.context);
        }
        if self.steuerbare_ressource_mapper.can_handle(segment) {
            self.steuerbare_ressource_mapper
                .handle(segment, &mut self.context);
        }
        if self.technische_ressource_mapper.can_handle(segment) {
            self.technische_ressource_mapper
                .handle(segment, &mut self.context);
        }
        if self.tranche_mapper.can_handle(segment) {
            self.tranche_mapper.handle(segment, &mut self.context);
        }
        if self.mabis_zaehlpunkt_mapper.can_handle(segment) {
            self.mabis_zaehlpunkt_mapper
                .handle(segment, &mut self.context);
        }
        if self.bilanzierung_mapper.can_handle(segment) {
            self.bilanzierung_mapper.handle(segment, &mut self.context);
        }
        if self.produktpaket_mapper.can_handle(segment) {
            self.produktpaket_mapper.handle(segment, &mut self.context);
        }
        if self.lokationszuordnung_mapper.can_handle(segment) {
            self.lokationszuordnung_mapper
                .handle(segment, &mut self.context);
        }
    }

    /// Handles IDE segment (transaction identifier).
    ///
    /// When a new IDE+24 is encountered while already processing a transaction,
    /// the current transaction is finalized first (multi-transaction support).
    fn handle_ide(&mut self, segment: &RawSegment) {
        // IDE+24+transactionId'
        let qualifier = segment.get_element(0);
        if qualifier == "24" {
            // Finalize previous transaction if we're already in one
            if self.in_transaction {
                let tx = self.collect_transaction();
                self.transactions.push(tx);
                self.reset_mappers();
            }

            let tx_id = segment.get_element(1);
            if !tx_id.is_empty() {
                self.current_transaction_id = Some(tx_id.to_string());
                self.in_transaction = true;
            }
        }
    }

    /// Handles NAD+MS (sender) and NAD+MR (recipient) at message level.
    fn handle_message_level_nad(&mut self, segment: &RawSegment) {
        let qualifier = segment.get_element(0);
        let mp_id = segment.get_component(1, 0);
        let code_qualifier = segment.get_component(1, 2);

        match qualifier {
            "MS" => {
                if !mp_id.is_empty() {
                    self.absender.mp_id = Some(mp_id.to_string());
                    self.context.set_sender_mp_id(mp_id);
                }
                if !code_qualifier.is_empty() {
                    self.nachrichtendaten.absender_code_qualifier =
                        Some(code_qualifier.to_string());
                }
            }
            "MR" => {
                if !mp_id.is_empty() {
                    self.empfaenger.mp_id = Some(mp_id.to_string());
                    self.context.set_recipient_mp_id(mp_id);
                }
                if !code_qualifier.is_empty() {
                    self.nachrichtendaten.empfaenger_code_qualifier =
                        Some(code_qualifier.to_string());
                }
            }
            _ => {}
        }
    }

    /// Handles BGM segment (document type).
    fn handle_bgm(&mut self, segment: &RawSegment) {
        // BGM+E03+documentNumber'
        let kategorie = segment.get_element(0);
        if !kategorie.is_empty() {
            self.nachrichtendaten.kategorie = Some(kategorie.to_string());
        }
        let doc_nr = segment.get_element(1);
        if !doc_nr.is_empty() {
            self.nachrichtendaten.dokumentennummer = Some(doc_nr.to_string());
        }
    }

    /// Collects all built objects into a UtilmdTransaktion.
    fn collect_transaction(&mut self) -> UtilmdTransaktion {
        let prozessdaten = self.prozessdaten_mapper.build();
        let zeitscheiben = self.zeitscheibe_mapper.build();
        let marktlokationen = self.marktlokation_mapper.build().into_iter().collect();
        let messlokationen = self.messlokation_mapper.build().into_iter().collect();
        let netzlokationen = self.netzlokation_mapper.build().into_iter().collect();
        let parteien = self.geschaeftspartner_mapper.build();
        let vertrag = self.vertrag_mapper.build();
        let zaehler = self.zaehler_mapper.build();

        UtilmdTransaktion {
            transaktions_id: self.current_transaction_id.take().unwrap_or_default(),
            referenz_transaktions_id: None,
            absender: self.absender.clone(),
            empfaenger: self.empfaenger.clone(),
            prozessdaten,
            antwortstatus: None,
            zeitscheiben,
            marktlokationen,
            messlokationen,
            netzlokationen,
            steuerbare_ressourcen: self
                .steuerbare_ressource_mapper
                .build()
                .into_iter()
                .collect(),
            technische_ressourcen: self
                .technische_ressource_mapper
                .build()
                .into_iter()
                .collect(),
            tranchen: self.tranche_mapper.build().into_iter().collect(),
            mabis_zaehlpunkte: self.mabis_zaehlpunkt_mapper.build().into_iter().collect(),
            parteien,
            vertrag,
            bilanzierung: self.bilanzierung_mapper.build(),
            zaehler,
            produktpakete: self.produktpaket_mapper.build(),
            lokationszuordnungen: self.lokationszuordnung_mapper.build(),
        }
    }

    /// Resets all mappers for a new transaction.
    fn reset_mappers(&mut self) {
        self.prozessdaten_mapper.reset();
        self.zeitscheibe_mapper.reset();
        self.marktlokation_mapper.reset();
        self.messlokation_mapper.reset();
        self.netzlokation_mapper.reset();
        self.geschaeftspartner_mapper.reset();
        self.vertrag_mapper.reset();
        self.zaehler_mapper.reset();
        self.steuerbare_ressource_mapper.reset();
        self.technische_ressource_mapper.reset();
        self.tranche_mapper.reset();
        self.mabis_zaehlpunkt_mapper.reset();
        self.bilanzierung_mapper.reset();
        self.produktpaket_mapper.reset();
        self.lokationszuordnung_mapper.reset();
        self.in_transaction = false;
    }

    /// Builds a UtilmdNachricht from the current coordinator state.
    fn build_nachricht(&mut self) -> UtilmdNachricht {
        let transactions = std::mem::take(&mut self.transactions);
        let nachrichtendaten = self.nachrichtendaten.clone();

        UtilmdNachricht {
            dokumentennummer: nachrichtendaten
                .dokumentennummer
                .clone()
                .unwrap_or_default(),
            kategorie: nachrichtendaten.kategorie.clone(),
            nachrichtendaten,
            transaktionen: transactions,
        }
    }

    /// Generates EDIFACT bytes from a UtilmdNachricht.
    ///
    /// Segment ordering follows the MIG XML Counter attributes from
    /// `UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml`.
    /// See `docs/mig-segment-ordering.md` for the full derivation.
    fn generate_impl(
        nachricht: &UtilmdNachricht,
        format_version: FormatVersion,
    ) -> Result<Vec<u8>, AutomapperError> {
        let nd = &nachricht.nachrichtendaten;

        // Determine date/time for UNB: prefer raw UNB values, fallback to erstellungsdatum
        let (date_str, time_str) = if nd.unb_datum.is_some() || nd.unb_zeit.is_some() {
            (
                nd.unb_datum.clone().unwrap_or_default(),
                nd.unb_zeit.clone().unwrap_or_default(),
            )
        } else if let Some(ref dt) = nd.erstellungsdatum {
            (
                dt.format("%y%m%d").to_string(),
                dt.format("%H%M").to_string(),
            )
        } else {
            (String::new(), String::new())
        };

        let mut doc = EdifactDocumentWriter::new();

        // UNA + UNB (Counter=0000)
        doc.begin_interchange(
            nd.absender_mp_id.as_deref().unwrap_or(""),
            nd.absender_unb_qualifier.as_deref(),
            nd.empfaenger_mp_id.as_deref().unwrap_or(""),
            nd.empfaenger_unb_qualifier.as_deref(),
            nd.datenaustauschreferenz.as_deref().unwrap_or(""),
            &date_str,
            &time_str,
            nd.explicit_una,
        );

        // UNH (Counter=0010) — use original message type if available, else derive from version
        let message_type = nd
            .nachrichtentyp
            .as_deref()
            .unwrap_or_else(|| format_version.message_type_string());
        doc.begin_message(
            nd.nachrichtenreferenz.as_deref().unwrap_or(""),
            message_type,
        );

        // BGM (Counter=0020)
        doc.write_segment(
            "BGM",
            &[
                nachricht.kategorie.as_deref().unwrap_or(""),
                &nachricht.dokumentennummer,
            ],
        );

        // DTM+137 Nachrichtendatum (Counter=0030, Nr 00005)
        if let Some(ref dt) = nd.erstellungsdatum {
            let value = dt.format("%Y%m%d%H%M").to_string();
            doc.write_segment_with_composites("DTM", &[&["137", &value, "303"]]);
        }

        // SG2: NAD+MS Absender (Counter=0100, Nr 00008)
        if let Some(ref mp_id) = nd.absender_mp_id {
            let cq = nd.absender_code_qualifier.as_deref().unwrap_or("293");
            let nad_value = format!("{mp_id}::{cq}");
            doc.write_segment("NAD", &["MS", &nad_value]);
        }

        // SG2: NAD+MR Empfaenger (Counter=0100, Nr 00011)
        if let Some(ref mp_id) = nd.empfaenger_mp_id {
            let cq = nd.empfaenger_code_qualifier.as_deref().unwrap_or("293");
            let nad_value = format!("{mp_id}::{cq}");
            doc.write_segment("NAD", &["MR", &nad_value]);
        }

        // SG4: Transactions (Counter=0180)
        for tx in &nachricht.transaktionen {
            Self::write_transaction(&mut doc, tx);
        }

        // UNT + UNZ
        doc.end_message();
        doc.end_interchange();

        Ok(doc.into_bytes())
    }

    /// Writes a single transaction (SG4) to the document writer.
    ///
    /// MIG segment ordering within SG4 (by Counter):
    /// - 0190: IDE+24
    /// - 0230: DTM (process dates)
    /// - 0250: STS (transaction reason)
    /// - 0280: FTX (remarks)
    /// - 0320: SG5/LOC (Z18, Z16, Z20, Z19, Z21, Z17, Z15)
    /// - 0350: SG6/RFF (Z13, Z47+DTM)
    /// - 0410: SG8/SEQ (Z78, Z79, Z98, Z03, Z18)
    /// - 0570: SG12/NAD (DP address, geschaeftspartner)
    fn write_transaction(doc: &mut EdifactDocumentWriter, tx: &UtilmdTransaktion) {
        // IDE+24+transaktions_id (Counter=0190, Nr 00020)
        doc.write_segment("IDE", &["24", &tx.transaktions_id]);

        // DTM + STS + FTX (Counter=0230, 0250, 0280)
        ProzessdatenWriter::write(doc, &tx.prozessdaten);

        // SG5: LOC segments (Counter=0320)
        // MIG order: Z18 (Nr 48), Z16 (Nr 49), Z20 (Nr 51), Z19 (Nr 52),
        //            Z21 (Nr 53), Z17 (Nr 54), Z15 (Nr 55)
        for nl in &tx.netzlokationen {
            NetzlokationWriter::write(doc, nl);
        }
        for ml in &tx.marktlokationen {
            MarktlokationWriter::write(doc, ml);
        }
        for tr in &tx.technische_ressourcen {
            TechnischeRessourceWriter::write(doc, tr);
        }
        for sr in &tx.steuerbare_ressourcen {
            SteuerbareRessourceWriter::write(doc, sr);
        }
        for t in &tx.tranchen {
            TrancheWriter::write(doc, t);
        }
        for ml in &tx.messlokationen {
            MesslokationWriter::write(doc, ml);
        }
        for mz in &tx.mabis_zaehlpunkte {
            MabisZaehlpunktWriter::write(doc, mz);
        }

        // SG6: RFF references (Counter=0350)
        // RFF+Z13 (Nr 00056)
        ProzessdatenWriter::write_references(doc, &tx.prozessdaten);
        // RFF+Z47 Zeitscheiben (Nr 00066)
        ZeitscheibeWriter::write(doc, &tx.zeitscheiben);

        // SG8: SEQ groups (Counter=0410)
        // MIG order: Z78 (Nr 74), Z79 (Nr 81), Z98 (Bilanzierung), Z03 (Nr 311), Z18 (Nr 291)
        for lz in &tx.lokationszuordnungen {
            LokationszuordnungWriter::write(doc, lz);
        }
        for pp in &tx.produktpakete {
            ProduktpaketWriter::write(doc, pp);
        }
        if let Some(ref b) = tx.bilanzierung {
            BilanzierungWriter::write(doc, b);
        }
        for z in &tx.zaehler {
            ZaehlerWriter::write(doc, z);
        }
        if let Some(ref v) = tx.vertrag {
            VertragWriter::write(doc, v);
        }

        // SG12: NAD parties (Counter=0570)
        // NAD+DP Marktlokationsanschrift (Nr 00518)
        for ml in &tx.marktlokationen {
            MarktlokationWriter::write_address(doc, ml);
        }
        // NAD+qualifier Geschaeftspartner (Nr varies by qualifier)
        for gp in &tx.parteien {
            GeschaeftspartnerWriter::write(doc, gp);
        }
    }
}

impl<V: VersionConfig> Default for UtilmdCoordinator<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: VersionConfig> EdifactHandler for UtilmdCoordinator<V> {
    fn on_delimiters(&mut self, _delimiters: &EdifactDelimiters, explicit_una: bool) {
        self.nachrichtendaten.explicit_una = explicit_una;
    }

    fn on_interchange_start(&mut self, unb: &RawSegment) -> Control {
        // Extract sender/recipient from UNB
        let sender = unb.get_component(1, 0);
        if !sender.is_empty() {
            self.nachrichtendaten.absender_mp_id = Some(sender.to_string());
        }
        let sender_qualifier = unb.get_component(1, 1);
        if !sender_qualifier.is_empty() {
            self.nachrichtendaten.absender_unb_qualifier = Some(sender_qualifier.to_string());
        }
        let recipient = unb.get_component(2, 0);
        if !recipient.is_empty() {
            self.nachrichtendaten.empfaenger_mp_id = Some(recipient.to_string());
        }
        let recipient_qualifier = unb.get_component(2, 1);
        if !recipient_qualifier.is_empty() {
            self.nachrichtendaten.empfaenger_unb_qualifier = Some(recipient_qualifier.to_string());
        }
        let unb_datum = unb.get_component(3, 0);
        if !unb_datum.is_empty() {
            self.nachrichtendaten.unb_datum = Some(unb_datum.to_string());
        }
        let unb_zeit = unb.get_component(3, 1);
        if !unb_zeit.is_empty() {
            self.nachrichtendaten.unb_zeit = Some(unb_zeit.to_string());
        }
        let ref_nr = unb.get_element(4);
        if !ref_nr.is_empty() {
            self.nachrichtendaten.datenaustauschreferenz = Some(ref_nr.to_string());
        }
        Control::Continue
    }

    fn on_message_start(&mut self, unh: &RawSegment) -> Control {
        let msg_ref = unh.get_element(0);
        if !msg_ref.is_empty() {
            self.nachrichtendaten.nachrichtenreferenz = Some(msg_ref.to_string());
            self.context.set_message_reference(msg_ref);
        }
        let msg_type_components = unh.get_components(1);
        if !msg_type_components.is_empty() {
            self.nachrichtendaten.nachrichtentyp = Some(msg_type_components.join(":"));
        }
        Control::Continue
    }

    fn on_segment(&mut self, segment: &RawSegment) -> Control {
        match segment.id {
            "BGM" => self.handle_bgm(segment),
            "IDE" => self.handle_ide(segment),
            "NAD" => {
                let q = segment.get_element(0);
                if q == "MS" || q == "MR" {
                    self.handle_message_level_nad(segment);
                }
                // Also route to mappers (Geschaeftspartner handles party NADs)
                self.route_to_mappers(segment);
            }
            "DTM" if !self.in_transaction => {
                let qualifier = segment.get_component(0, 0);
                if qualifier == "137" {
                    let value = segment.get_component(0, 1);
                    let format_code = segment.get_component(0, 2);
                    if let Some(dt) = parse_edifact_dtm(value, format_code) {
                        self.nachrichtendaten.erstellungsdatum = Some(dt);
                    }
                }
                self.route_to_mappers(segment);
            }
            _ => {
                self.route_to_mappers(segment);
            }
        }
        Control::Continue
    }

    fn on_message_end(&mut self, _unt: &RawSegment) {
        // Collect the transaction if we have one
        if self.in_transaction || !self.prozessdaten_mapper.is_empty() {
            let tx = self.collect_transaction();
            self.transactions.push(tx);
            self.reset_mappers();
        }
    }

    fn on_interchange_end(&mut self, _unz: &RawSegment) {}
}

impl<V: VersionConfig> Coordinator for UtilmdCoordinator<V> {
    fn parse(&mut self, input: &[u8]) -> Result<Vec<UtilmdTransaktion>, AutomapperError> {
        edifact_parser::EdifactStreamParser::parse(input, self)?;
        Ok(std::mem::take(&mut self.transactions))
    }

    fn parse_nachricht(&mut self, input: &[u8]) -> Result<UtilmdNachricht, AutomapperError> {
        edifact_parser::EdifactStreamParser::parse(input, self)?;
        Ok(self.build_nachricht())
    }

    fn generate(&self, nachricht: &UtilmdNachricht) -> Result<Vec<u8>, AutomapperError> {
        Self::generate_impl(nachricht, V::VERSION)
    }

    fn format_version(&self) -> FormatVersion {
        V::VERSION
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::{FV2504, FV2510};

    #[test]
    fn test_utilmd_coordinator_new() {
        let coord = UtilmdCoordinator::<FV2504>::new();
        assert_eq!(coord.format_version(), FormatVersion::FV2504);
        assert!(coord.transactions.is_empty());
    }

    #[test]
    fn test_utilmd_coordinator_fv2510() {
        let coord = UtilmdCoordinator::<FV2510>::new();
        assert_eq!(coord.format_version(), FormatVersion::FV2510);
    }

    #[test]
    fn test_utilmd_coordinator_parse_minimal() {
        let input = b"UNA:+.? 'UNB+UNOC:3+9900123000002:500+9900456000001:500+251217:1229+REF001'UNH+MSG001+UTILMD:D:11A:UN:S2.1'BGM+E03+DOC001'NAD+MS+9900123000002::293'NAD+MR+9900456000001::293'IDE+24+TXID001'LOC+Z16+DE00014545768S0000000000000003054'STS+E01+E01'UNT+8+MSG001'UNZ+1+REF001'";

        let mut coord = UtilmdCoordinator::<FV2504>::new();
        let result = coord.parse(input).unwrap();

        assert_eq!(result.len(), 1);
        let tx = &result[0];
        assert_eq!(tx.transaktions_id, "TXID001");
        assert_eq!(tx.absender.mp_id, Some("9900123000002".to_string()));
        assert_eq!(tx.empfaenger.mp_id, Some("9900456000001".to_string()));
        assert_eq!(tx.marktlokationen.len(), 1);
        assert_eq!(
            tx.marktlokationen[0].data.marktlokations_id,
            Some("DE00014545768S0000000000000003054".to_string())
        );
    }

    #[test]
    fn test_utilmd_coordinator_parse_with_messlokation() {
        let input = b"UNA:+.? 'UNB+UNOC:3+S+R+251217:1229+REF'UNH+MSG+UTILMD:D:11A:UN:S2.1'BGM+E03+DOC'IDE+24+TX1'LOC+Z16+MALO001'LOC+Z17+MELO001'LOC+Z18+NELO001'UNT+7+MSG'UNZ+1+REF'";

        let mut coord = UtilmdCoordinator::<FV2504>::new();
        let result = coord.parse(input).unwrap();

        assert_eq!(result.len(), 1);
        let tx = &result[0];
        assert_eq!(tx.marktlokationen.len(), 1);
        assert_eq!(tx.messlokationen.len(), 1);
        assert_eq!(tx.netzlokationen.len(), 1);
        assert_eq!(
            tx.messlokationen[0].data.messlokations_id,
            Some("MELO001".to_string())
        );
        assert_eq!(
            tx.netzlokationen[0].data.netzlokations_id,
            Some("NELO001".to_string())
        );
    }

    #[test]
    fn test_utilmd_coordinator_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<UtilmdCoordinator<FV2504>>();
    }
}
