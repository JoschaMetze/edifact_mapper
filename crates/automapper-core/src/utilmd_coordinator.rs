//! UTILMD-specific coordinator that orchestrates all mappers.
//!
//! Implements `EdifactHandler` and `Coordinator`. Routes segments to
//! registered mappers and collects built objects into `UtilmdTransaktion`.
//!
//! Mirrors C# `UtilmdCoordinator.cs`.

use std::marker::PhantomData;

use bo4e_extensions::{
    LinkRegistry, Marktteilnehmer, Nachrichtendaten, PassthroughSegment, SegmentZone,
    UtilmdNachricht, UtilmdTransaktion,
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

    // Passthrough state
    delimiters: EdifactDelimiters,
    current_zone: SegmentZone,
    passthrough_segments: Vec<PassthroughSegment>,
    message_passthrough: Vec<PassthroughSegment>,

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
            delimiters: EdifactDelimiters::default(),
            current_zone: SegmentZone::MessageHeader,
            passthrough_segments: Vec::new(),
            message_passthrough: Vec::new(),
            _version: PhantomData,
        }
    }

    /// Routes a segment to all mappers that can handle it.
    /// If no mapper handles the segment, it is captured as a passthrough segment.
    /// When `coordinator_handled` is true, the segment won't become passthrough even
    /// if no mapper claims it (the coordinator already processed it).
    fn route_to_mappers_ex(&mut self, segment: &RawSegment, coordinator_handled: bool) {
        // Service segments (UNA, UNB, UNH, UNT, UNZ) are handled by the parser's
        // dedicated callbacks and should never become passthrough.
        if matches!(segment.id, "UNB" | "UNH" | "UNT" | "UNZ") {
            return;
        }

        // Update zone based on segment type
        self.update_zone(segment);

        // Route to mappers and track if handled
        let mut handled = coordinator_handled;

        if self.prozessdaten_mapper.can_handle(segment) {
            self.prozessdaten_mapper.handle(segment, &mut self.context);
            handled = true;
        }
        if self.zeitscheibe_mapper.can_handle(segment) {
            self.zeitscheibe_mapper.handle(segment, &mut self.context);
            handled = true;
        }
        if self.marktlokation_mapper.can_handle(segment) {
            self.marktlokation_mapper.handle(segment, &mut self.context);
            handled = true;
        }
        if self.messlokation_mapper.can_handle(segment) {
            self.messlokation_mapper.handle(segment, &mut self.context);
            handled = true;
        }
        if self.netzlokation_mapper.can_handle(segment) {
            self.netzlokation_mapper.handle(segment, &mut self.context);
            handled = true;
        }
        if self.geschaeftspartner_mapper.can_handle(segment) {
            self.geschaeftspartner_mapper
                .handle(segment, &mut self.context);
            handled = true;
        }
        if self.vertrag_mapper.can_handle(segment) {
            self.vertrag_mapper.handle(segment, &mut self.context);
            handled = true;
        }
        if self.zaehler_mapper.can_handle(segment) {
            self.zaehler_mapper.handle(segment, &mut self.context);
            handled = true;
        }
        if self.steuerbare_ressource_mapper.can_handle(segment) {
            self.steuerbare_ressource_mapper
                .handle(segment, &mut self.context);
            handled = true;
        }
        if self.technische_ressource_mapper.can_handle(segment) {
            self.technische_ressource_mapper
                .handle(segment, &mut self.context);
            handled = true;
        }
        if self.tranche_mapper.can_handle(segment) {
            self.tranche_mapper.handle(segment, &mut self.context);
            handled = true;
        }
        if self.mabis_zaehlpunkt_mapper.can_handle(segment) {
            self.mabis_zaehlpunkt_mapper
                .handle(segment, &mut self.context);
            handled = true;
        }
        if self.bilanzierung_mapper.can_handle(segment) {
            self.bilanzierung_mapper.handle(segment, &mut self.context);
            handled = true;
        }
        if self.produktpaket_mapper.can_handle(segment) {
            self.produktpaket_mapper.handle(segment, &mut self.context);
            handled = true;
        }
        if self.lokationszuordnung_mapper.can_handle(segment) {
            self.lokationszuordnung_mapper
                .handle(segment, &mut self.context);
            handled = true;
        }

        // If no mapper handled it, store as passthrough
        if !handled {
            let raw = segment.to_raw_string(&self.delimiters);
            let ps = PassthroughSegment {
                raw,
                zone: self.current_zone,
            };
            if self.in_transaction {
                self.passthrough_segments.push(ps);
            } else {
                self.message_passthrough.push(ps);
            }
        }
    }

    /// Routes a segment to all mappers that can handle it.
    /// If no mapper handles the segment, it is captured as a passthrough segment.
    fn route_to_mappers(&mut self, segment: &RawSegment) {
        self.route_to_mappers_ex(segment, false);
    }

    /// Updates the current zone based on the segment type.
    fn update_zone(&mut self, segment: &RawSegment) {
        if !self.in_transaction {
            self.current_zone = SegmentZone::MessageHeader;
            return;
        }

        match segment.id {
            "LOC" => self.current_zone = SegmentZone::Locations,
            "RFF"
                if self.current_zone == SegmentZone::Locations
                    || self.current_zone == SegmentZone::TransactionHeader =>
            {
                self.current_zone = SegmentZone::References;
            }
            "SEQ" => self.current_zone = SegmentZone::Sequences,
            // CCI, CAV, QTY, PIA after SEQ stay in Sequences
            "CCI" | "CAV" | "QTY" | "PIA" if self.current_zone == SegmentZone::Sequences => {}
            // RFF after SEQ stays in Sequences (e.g. RFF+Z19 inside a SEQ+Z03 group)
            "RFF" if self.current_zone == SegmentZone::Sequences => {}
            // DTM after SEQ stays in Sequences
            "DTM" if self.current_zone == SegmentZone::Sequences => {}
            "NAD"
                if self.current_zone == SegmentZone::Sequences
                    || self.current_zone == SegmentZone::Parties =>
            {
                self.current_zone = SegmentZone::Parties;
            }
            // Default: stay in current zone for segments within their group
            _ => {
                // For DTM/STS/FTX before LOC, stay in TransactionHeader
                if self.current_zone == SegmentZone::MessageHeader {
                    self.current_zone = SegmentZone::TransactionHeader;
                }
            }
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
                self.current_zone = SegmentZone::TransactionHeader;
                self.passthrough_segments.clear();
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
            passthrough_segments: std::mem::take(&mut self.passthrough_segments),
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
        self.current_zone = SegmentZone::TransactionHeader;
        self.passthrough_segments.clear();
    }

    /// Builds a UtilmdNachricht from the current coordinator state.
    fn build_nachricht(&mut self) -> UtilmdNachricht {
        let transactions = std::mem::take(&mut self.transactions);
        let mut nachrichtendaten = std::mem::take(&mut self.nachrichtendaten);
        nachrichtendaten.passthrough_segments = std::mem::take(&mut self.message_passthrough);

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
        if let Some(ref raw) = nd.raw_nachrichtendatum {
            let composite = format!("137:{}", raw);
            doc.write_segment("DTM", &[&composite]);
        } else if let Some(ref dt) = nd.erstellungsdatum {
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

        // Message-level passthrough segments
        for ps in &nd.passthrough_segments {
            doc.write_raw_segment(&ps.raw);
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

    /// Replays passthrough segments for a specific zone.
    fn replay_passthrough(
        doc: &mut EdifactDocumentWriter,
        segments: &[PassthroughSegment],
        zone: SegmentZone,
    ) {
        for ps in segments.iter().filter(|ps| ps.zone == zone) {
            doc.write_raw_segment(&ps.raw);
        }
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
        // Replay TransactionHeader passthrough (unmodeled DTM qualifiers, etc.)
        Self::replay_passthrough(
            doc,
            &tx.passthrough_segments,
            SegmentZone::TransactionHeader,
        );

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
        // Replay Locations passthrough (unmodeled LOC qualifiers)
        Self::replay_passthrough(doc, &tx.passthrough_segments, SegmentZone::Locations);

        // SG6: RFF references (Counter=0350)
        // RFF+Z13 (Nr 00056)
        ProzessdatenWriter::write_references(doc, &tx.prozessdaten);
        // RFF+Z47 Zeitscheiben (Nr 00066)
        ZeitscheibeWriter::write(doc, &tx.zeitscheiben);
        // Replay References passthrough
        Self::replay_passthrough(doc, &tx.passthrough_segments, SegmentZone::References);

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
        // Replay Sequences passthrough (unmodeled SEQ+qualifier groups)
        Self::replay_passthrough(doc, &tx.passthrough_segments, SegmentZone::Sequences);

        // SG12: NAD parties (Counter=0570)
        // NAD+DP Marktlokationsanschrift (Nr 00518)
        for ml in &tx.marktlokationen {
            MarktlokationWriter::write_address(doc, ml);
        }
        // NAD+qualifier Geschaeftspartner (Nr varies by qualifier)
        for gp in &tx.parteien {
            GeschaeftspartnerWriter::write(doc, gp);
        }
        // Replay Parties passthrough (unmodeled NAD qualifiers)
        Self::replay_passthrough(doc, &tx.passthrough_segments, SegmentZone::Parties);
    }
}

impl<V: VersionConfig> Default for UtilmdCoordinator<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: VersionConfig> EdifactHandler for UtilmdCoordinator<V> {
    fn on_delimiters(&mut self, delimiters: &EdifactDelimiters, explicit_una: bool) {
        self.nachrichtendaten.explicit_una = explicit_una;
        self.delimiters = *delimiters;
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
                    // Route to mappers (MarktlokationMapper uses NAD+MS for Sparte)
                    // but mark as coordinator-handled so NAD+MR doesn't become passthrough
                    self.route_to_mappers_ex(segment, true);
                } else {
                    self.route_to_mappers(segment);
                }
            }
            "DTM" if !self.in_transaction => {
                let qualifier = segment.get_component(0, 0);
                if qualifier == "137" {
                    let value = segment.get_component(0, 1);
                    let format_code = segment.get_component(0, 2);
                    // Store raw for roundtrip
                    if !value.is_empty() {
                        let raw = if format_code.is_empty() {
                            value.to_string()
                        } else {
                            format!("{}:{}", value, format_code)
                        };
                        self.nachrichtendaten.raw_nachrichtendatum = Some(raw);
                    }
                    if let Some(dt) = parse_edifact_dtm(value, format_code) {
                        self.nachrichtendaten.erstellungsdatum = Some(dt);
                    }
                } else {
                    // Non-137 message-level DTMs: store as passthrough
                    let raw = segment.to_raw_string(&self.delimiters);
                    self.message_passthrough.push(PassthroughSegment {
                        raw,
                        zone: SegmentZone::MessageHeader,
                    });
                }
                // Don't route message-level DTMs to mappers — they would
                // incorrectly populate the transaction's prozessdaten.
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

    #[test]
    fn test_passthrough_captures_unhandled_segments() {
        use crate::coordinator::create_coordinator;

        // EDIFACT with IMD (unhandled at message level) and various transaction segments
        let edifact = b"UNA:+.? '\
UNB+UNOC:3+SENDER:500+RECEIVER:500+251217:1229+REF001'\
UNH+MSG001+UTILMD:D:11A:UN:S2.1'\
BGM+E03+DOC001'\
DTM+137:202507011330:303'\
IMD++Z36+Z13'\
NAD+MS+SENDER::293'\
NAD+MR+RECEIVER::293'\
IDE+24+TX001'\
DTM+92:20220624:102'\
STS+7++E01'\
LOC+Z16+MALO001'\
SEQ+Z01'\
CCI+Z30++Z07'\
CAV+Z74:::Z09'\
SEQ+Z03'\
PIA+5+ZAEHLER001'\
NAD+Z09+++Test:Person'\
UNT+17+MSG001'\
UNZ+1+REF001'";

        let mut coord = create_coordinator(FormatVersion::FV2504).unwrap();
        let nachricht = coord.parse_nachricht(edifact).unwrap();

        // IMD should be message-level passthrough (not handled by any mapper)
        assert!(
            !nachricht.nachrichtendaten.passthrough_segments.is_empty(),
            "IMD should be captured as message-level passthrough"
        );
        assert!(
            nachricht
                .nachrichtendaten
                .passthrough_segments
                .iter()
                .any(|ps| ps.raw.starts_with("IMD")),
            "IMD segment should be in message passthrough"
        );
        // IMD is in MessageHeader zone (before IDE)
        let imd = nachricht
            .nachrichtendaten
            .passthrough_segments
            .iter()
            .find(|ps| ps.raw.starts_with("IMD"))
            .unwrap();
        assert_eq!(imd.zone, SegmentZone::MessageHeader);

        // Transaction passthrough: segments not handled by any mapper
        let tx = &nachricht.transaktionen[0];
        let passthrough_ids: Vec<&str> = tx
            .passthrough_segments
            .iter()
            .map(|ps| ps.raw.split('+').next().unwrap_or(""))
            .collect();
        eprintln!(
            "Transaction passthrough segments: {:?}",
            tx.passthrough_segments
        );

        // CCI+Z30 and CAV+Z74 may or may not be handled depending on mapper support.
        // The key invariant: segments not handled by any mapper appear in passthrough.
        // NAD+MS/MR should NOT appear in passthrough (handled at message level).
        assert!(
            !nachricht
                .nachrichtendaten
                .passthrough_segments
                .iter()
                .any(|ps| ps.raw.starts_with("NAD+MS") || ps.raw.starts_with("NAD+MR")),
            "NAD+MS/MR should NOT be in passthrough — handled at message level"
        );

        // BGM should NOT appear in passthrough (handled by handle_bgm)
        assert!(
            !nachricht
                .nachrichtendaten
                .passthrough_segments
                .iter()
                .any(|ps| ps.raw.starts_with("BGM")),
            "BGM should NOT be in passthrough"
        );

        // Verify zone tracking: any passthrough in the SEQ area should be Sequences zone
        for ps in &tx.passthrough_segments {
            if ps.raw.starts_with("CCI") || ps.raw.starts_with("CAV") {
                assert_eq!(
                    ps.zone,
                    SegmentZone::Sequences,
                    "CCI/CAV should be in Sequences zone, got {:?}",
                    ps.zone
                );
            }
        }

        // Verify passthrough_ids is used (avoid unused variable warning)
        let _ = passthrough_ids;
    }
}
