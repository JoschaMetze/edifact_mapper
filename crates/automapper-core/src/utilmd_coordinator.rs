//! UTILMD-specific coordinator that orchestrates all mappers.
//!
//! Implements `EdifactHandler` and `Coordinator`. Routes segments to
//! registered mappers and collects built objects into `UtilmdTransaktion`.
//!
//! Mirrors C# `UtilmdCoordinator.cs`.

use std::marker::PhantomData;

use bo4e_extensions::{
    Ansprechpartner, Kommunikationsdetail, LinkRegistry, Marktteilnehmer, Nachrichtendaten,
    PassthroughSegment, SegmentZone, UtilmdNachricht, UtilmdTransaktion,
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
    ProduktpaketWriter, ProzessdatenWriter, SeqGroupWriter, SteuerbareRessourceWriter,
    TechnischeRessourceWriter, TrancheWriter, VertragWriter, ZaehlerWriter, ZeitscheibeWriter,
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
    seq_group_mapper: SeqGroupMapper,

    // Collected transactions
    transactions: Vec<UtilmdTransaktion>,

    // Nachrichtendaten from service segments
    nachrichtendaten: Nachrichtendaten,
    absender: Marktteilnehmer,
    empfaenger: Marktteilnehmer,

    // Current transaction state
    in_transaction: bool,
    current_transaction_id: Option<String>,
    current_ide_qualifier: Option<String>,

    // CTA/COM state for message-level contact details
    expecting_contact_for: Option<String>, // "MS" or "MR"
    current_contact: Option<Ansprechpartner>,

    // NAD+VY RFF tracking
    after_nad_vy: bool,

    // Entity LOC ordering for roundtrip fidelity
    entity_loc_order: Vec<String>,
    // NAD qualifier ordering for roundtrip fidelity (party and address NADs)
    nad_qualifier_order: Vec<String>,

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
            seq_group_mapper: SeqGroupMapper::new(),
            transactions: Vec::new(),
            nachrichtendaten: Nachrichtendaten::default(),
            absender: Marktteilnehmer::default(),
            empfaenger: Marktteilnehmer::default(),
            in_transaction: false,
            current_transaction_id: None,
            current_ide_qualifier: None,
            expecting_contact_for: None,
            current_contact: None,
            after_nad_vy: false,
            entity_loc_order: Vec::new(),
            nad_qualifier_order: Vec::new(),
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
        if self.seq_group_mapper.can_handle(segment) {
            self.seq_group_mapper.handle(segment, &mut self.context);
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
            "LOC" => {
                self.current_zone = SegmentZone::Locations;
                // Track entity LOC order for roundtrip fidelity
                let qualifier = segment.get_element(0);
                if self.in_transaction
                    && matches!(
                        qualifier,
                        "Z18" | "Z16" | "Z22" | "Z20" | "Z19" | "Z21" | "Z17" | "Z15"
                    )
                {
                    self.entity_loc_order.push(qualifier.to_string());
                }
            }
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
    /// When a new IDE is encountered while already processing a transaction,
    /// the current transaction is finalized first (multi-transaction support).
    /// Accepts both standard `IDE+24` and deprecated `IDE+Z01`.
    fn handle_ide(&mut self, segment: &RawSegment) {
        let qualifier = segment.get_element(0);
        if qualifier == "24" || qualifier == "Z01" {
            // Finalize previous transaction if we're already in one
            if self.in_transaction {
                let tx = self.collect_transaction();
                self.transactions.push(tx);
                self.reset_mappers();
            }

            let tx_id = segment.get_element(1);
            if !tx_id.is_empty() {
                self.current_transaction_id = Some(tx_id.to_string());
                self.current_ide_qualifier = Some(qualifier.to_string());
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
                    // Store NAD+MS MP-ID separately (may differ from UNB sender)
                    self.nachrichtendaten.nad_ms_mp_id = Some(mp_id.to_string());
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
                    // Store NAD+MR MP-ID separately (may differ from UNB recipient)
                    self.nachrichtendaten.nad_mr_mp_id = Some(mp_id.to_string());
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

    /// Finalizes the current contact details and assigns to the correct party.
    fn finalize_contact(&mut self) {
        if let Some(contact) = self.current_contact.take() {
            match self.expecting_contact_for.as_deref() {
                Some("MS") => {
                    self.nachrichtendaten.absender_ansprechpartner = Some(contact);
                }
                Some("MR") => {
                    self.nachrichtendaten.empfaenger_ansprechpartner = Some(contact);
                }
                _ => {}
            }
        }
        self.expecting_contact_for = None;
    }

    /// Handles CTA+IC segment (contact details).
    fn handle_cta(&mut self, segment: &RawSegment) {
        let qualifier = segment.get_element(0);
        if qualifier == "IC" {
            let name = segment.get_component(1, 1);
            let mut contact = Ansprechpartner::default();
            if !name.is_empty() {
                contact.name = Some(name.to_string());
            }
            self.current_contact = Some(contact);
        }
    }

    /// Handles COM segment (communication detail).
    fn handle_com(&mut self, segment: &RawSegment) {
        if let Some(ref mut contact) = self.current_contact {
            let value = segment.get_component(0, 0);
            let qualifier = segment.get_component(0, 1);
            if !value.is_empty() {
                contact.kommunikation.push(Kommunikationsdetail {
                    value: value.to_string(),
                    qualifier: qualifier.to_string(),
                });
            }
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
        let marktlokationen = self.marktlokation_mapper.build();
        let messlokationen = self.messlokation_mapper.build();
        let netzlokationen = self.netzlokation_mapper.build();
        let parteien = self.geschaeftspartner_mapper.build();
        let vertrag = self.vertrag_mapper.build();
        let zaehler = self.zaehler_mapper.build();
        let (seq_z45, seq_z71, seq_z21, seq_z08, seq_z01, seq_z20, seq_generic, seq_order) =
            self.seq_group_mapper.build_all();

        UtilmdTransaktion {
            transaktions_id: self.current_transaction_id.take().unwrap_or_default(),
            ide_qualifier: self
                .current_ide_qualifier
                .take()
                .unwrap_or_else(|| "24".to_string()),
            referenz_transaktions_id: None,
            absender: self.absender.clone(),
            empfaenger: self.empfaenger.clone(),
            prozessdaten,
            antwortstatus: None,
            zeitscheiben,
            marktlokationen,
            messlokationen,
            netzlokationen,
            steuerbare_ressourcen: self.steuerbare_ressource_mapper.build(),
            technische_ressourcen: self.technische_ressource_mapper.build(),
            tranchen: self.tranche_mapper.build(),
            mabis_zaehlpunkte: self.mabis_zaehlpunkt_mapper.build(),
            parteien,
            vertrag,
            bilanzierung: self.bilanzierung_mapper.build(),
            zaehler,
            produktpakete: self.produktpaket_mapper.build(),
            lokationszuordnungen: self.lokationszuordnung_mapper.build(),
            seq_z45_groups: seq_z45,
            seq_z71_groups: seq_z71,
            seq_z21_groups: seq_z21,
            seq_z08_groups: seq_z08,
            seq_z01_groups: seq_z01,
            seq_z20_groups: seq_z20,
            generic_seq_groups: seq_generic,
            seq_group_order: seq_order,
            entity_loc_order: std::mem::take(&mut self.entity_loc_order),
            nad_qualifier_order: std::mem::take(&mut self.nad_qualifier_order),
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
        self.seq_group_mapper.reset();
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

        // Use original delimiters (decimal mark) for roundtrip fidelity
        let delimiters = if nd.una_decimal_mark != 0 {
            EdifactDelimiters {
                decimal: nd.una_decimal_mark,
                ..EdifactDelimiters::default()
            }
        } else {
            EdifactDelimiters::default()
        };
        let mut doc = EdifactDocumentWriter::with_delimiters(delimiters);

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

        // Message-level passthrough segments (e.g. DTM+157 at message level)
        for ps in &nd.passthrough_segments {
            doc.write_raw_segment(&ps.raw);
        }

        // SG2: NAD+MS Absender (Counter=0100, Nr 00008)
        // Prefer nad_ms_mp_id (actual NAD+MS value) over absender_mp_id (UNB sender)
        let ms_mp_id = nd.nad_ms_mp_id.as_ref().or(nd.absender_mp_id.as_ref());
        if let Some(mp_id) = ms_mp_id {
            let cq = nd.absender_code_qualifier.as_deref().unwrap_or("293");
            let nad_value = format!("{mp_id}::{cq}");
            doc.write_segment("NAD", &["MS", &nad_value]);
        }
        // CTA/COM after NAD+MS
        Self::write_contact(&mut doc, nd.absender_ansprechpartner.as_ref());

        // SG2: NAD+MR Empfaenger (Counter=0100, Nr 00011)
        // Prefer nad_mr_mp_id (actual NAD+MR value) over empfaenger_mp_id (UNB recipient)
        let mr_mp_id = nd.nad_mr_mp_id.as_ref().or(nd.empfaenger_mp_id.as_ref());
        if let Some(mp_id) = mr_mp_id {
            let cq = nd.empfaenger_code_qualifier.as_deref().unwrap_or("293");
            let nad_value = format!("{mp_id}::{cq}");
            doc.write_segment("NAD", &["MR", &nad_value]);
        }
        // CTA/COM after NAD+MR
        Self::write_contact(&mut doc, nd.empfaenger_ansprechpartner.as_ref());

        // SG4: Transactions (Counter=0180)
        for tx in &nachricht.transaktionen {
            Self::write_transaction(&mut doc, tx);
        }

        // UNT + UNZ
        doc.end_message_with_raw_unt(nd.raw_unt_count.as_deref(), nd.raw_unt_reference.as_deref());
        doc.end_interchange();

        Ok(doc.into_bytes())
    }

    /// Writes CTA+IC and COM segments for a contact person.
    fn write_contact(doc: &mut EdifactDocumentWriter, contact: Option<&Ansprechpartner>) {
        if let Some(ap) = contact {
            // CTA+IC+:Name
            let name = ap.name.as_deref().unwrap_or("");
            let name_composite = format!(":{}", name);
            doc.write_segment("CTA", &["IC", &name_composite]);

            // COM+value:qualifier for each communication detail
            for kom in &ap.kommunikation {
                let composite = format!("{}:{}", kom.value, kom.qualifier);
                doc.write_segment("COM", &[&composite]);
            }
        }
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
        // IDE+qualifier+transaktions_id (Counter=0190, Nr 00020)
        doc.write_segment("IDE", &[&tx.ide_qualifier, &tx.transaktions_id]);

        // DTM + STS + FTX (Counter=0230, 0250, 0280)
        ProzessdatenWriter::write(doc, &tx.prozessdaten);
        // Replay TransactionHeader passthrough (unmodeled DTM qualifiers, etc.)
        Self::replay_passthrough(
            doc,
            &tx.passthrough_segments,
            SegmentZone::TransactionHeader,
        );

        // SG5: LOC segments (Counter=0320)
        // Write entities in original LOC order when available.
        if !tx.entity_loc_order.is_empty() {
            let mut netz_idx = 0usize;
            let mut markt_idx = 0usize;
            let mut mess_idx = 0usize;
            let mut tech_idx = 0usize;
            let mut steuer_idx = 0usize;
            let mut tranche_idx = 0usize;
            let mut mabis_idx = 0usize;

            for qualifier in &tx.entity_loc_order {
                match qualifier.as_str() {
                    "Z18" => {
                        if let Some(nl) = tx.netzlokationen.get(netz_idx) {
                            NetzlokationWriter::write(doc, nl);
                            netz_idx += 1;
                        }
                    }
                    "Z16" => {
                        if let Some(ml) = tx.marktlokationen.get(markt_idx) {
                            MarktlokationWriter::write(doc, ml);
                            markt_idx += 1;
                        }
                    }
                    "Z22" => {
                        if let Some(ref id) = tx.prozessdaten.schlafende_marktlokation_id {
                            doc.write_segment("LOC", &["Z22", id]);
                        }
                    }
                    "Z20" => {
                        if let Some(tr) = tx.technische_ressourcen.get(tech_idx) {
                            TechnischeRessourceWriter::write(doc, tr);
                            tech_idx += 1;
                        }
                    }
                    "Z19" => {
                        if let Some(sr) = tx.steuerbare_ressourcen.get(steuer_idx) {
                            SteuerbareRessourceWriter::write(doc, sr);
                            steuer_idx += 1;
                        }
                    }
                    "Z21" => {
                        if let Some(t) = tx.tranchen.get(tranche_idx) {
                            TrancheWriter::write(doc, t);
                            tranche_idx += 1;
                        }
                    }
                    "Z17" => {
                        if let Some(ml) = tx.messlokationen.get(mess_idx) {
                            MesslokationWriter::write(doc, ml);
                            mess_idx += 1;
                        }
                    }
                    "Z15" => {
                        if let Some(mz) = tx.mabis_zaehlpunkte.get(mabis_idx) {
                            MabisZaehlpunktWriter::write(doc, mz);
                            mabis_idx += 1;
                        }
                    }
                    _ => {}
                }
            }
        } else {
            // Fallback: MIG order Z18, Z16, Z22, Z20, Z19, Z21, Z17, Z15
            for nl in &tx.netzlokationen {
                NetzlokationWriter::write(doc, nl);
            }
            for ml in &tx.marktlokationen {
                MarktlokationWriter::write(doc, ml);
            }
            if let Some(ref id) = tx.prozessdaten.schlafende_marktlokation_id {
                doc.write_segment("LOC", &["Z22", id]);
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
        }
        // Replay Locations passthrough (unmodeled LOC qualifiers)
        Self::replay_passthrough(doc, &tx.passthrough_segments, SegmentZone::Locations);

        // SG6: RFF references (Counter=0350)
        // RFF+Z13 (Nr 00056)
        ProzessdatenWriter::write_references(doc, &tx.prozessdaten);
        // RFF+Z47/Z49/Z50/Z53 Zeitscheiben (Nr 00066)
        // Only write via ZeitscheibeWriter if ProzessdatenWriter didn't already
        // write raw zeitscheibe_refs (which contain the original RFF format).
        if tx.prozessdaten.zeitscheibe_refs.is_empty() {
            ZeitscheibeWriter::write(doc, &tx.zeitscheiben);
        }
        // Replay References passthrough
        Self::replay_passthrough(doc, &tx.passthrough_segments, SegmentZone::References);

        // SG8: SEQ groups (Counter=0410)
        // Write in original order using seq_group_order to preserve message-specific ordering.
        // Each entry is (qualifier, index_within_type).
        // Dedicated mapper types (Z03, Z78, Z79, ZH0, Z98) have index=0 as placeholders.

        // Track which dedicated entities have been written for each occurrence
        let mut zaehler_idx = 0usize;
        let mut produktpaket_idx = 0usize;
        let mut lokationszuordnung_idx = 0usize;
        let mut bilanzierung_written = false;

        for (qualifier, idx) in &tx.seq_group_order {
            match qualifier.as_str() {
                "Z01" => {
                    if let Some(g) = tx.seq_z01_groups.get(*idx) {
                        SeqGroupWriter::write_z01(doc, g);
                    }
                }
                "Z45" => {
                    if let Some(g) = tx.seq_z45_groups.get(*idx) {
                        SeqGroupWriter::write_z45(doc, g);
                    }
                }
                "Z71" => {
                    if let Some(g) = tx.seq_z71_groups.get(*idx) {
                        SeqGroupWriter::write_z71(doc, g);
                    }
                }
                "Z21" => {
                    if let Some(g) = tx.seq_z21_groups.get(*idx) {
                        SeqGroupWriter::write_z21(doc, g);
                    }
                }
                "Z08" => {
                    if let Some(g) = tx.seq_z08_groups.get(*idx) {
                        SeqGroupWriter::write_z08(doc, g);
                    }
                }
                "Z20" => {
                    if let Some(g) = tx.seq_z20_groups.get(*idx) {
                        SeqGroupWriter::write_z20(doc, g);
                    }
                }
                "Z03" => {
                    // Zaehler — write next unwritten zaehler
                    if let Some(z) = tx.zaehler.get(zaehler_idx) {
                        ZaehlerWriter::write(doc, z);
                        zaehler_idx += 1;
                    }
                }
                "Z79" | "ZH0" => {
                    // Produktpaket
                    if let Some(pp) = tx.produktpakete.get(produktpaket_idx) {
                        ProduktpaketWriter::write(doc, pp);
                        produktpaket_idx += 1;
                    }
                }
                "Z78" => {
                    // Lokationszuordnung
                    if let Some(lz) = tx.lokationszuordnungen.get(lokationszuordnung_idx) {
                        LokationszuordnungWriter::write(doc, lz);
                        lokationszuordnung_idx += 1;
                    }
                }
                "Z98" | "Z81" => {
                    // Bilanzierung
                    if !bilanzierung_written {
                        if let Some(ref b) = tx.bilanzierung {
                            BilanzierungWriter::write(doc, b);
                        }
                        bilanzierung_written = true;
                    }
                }
                "_generic" => {
                    if let Some(g) = tx.generic_seq_groups.get(*idx) {
                        SeqGroupWriter::write_generic(doc, g);
                    }
                }
                _ => {} // Unknown qualifier, skip
            }
        }

        // Write any remaining entities not covered by seq_group_order
        // (fallback for messages parsed before order tracking was added)
        if tx.seq_group_order.is_empty() {
            for pp in &tx.produktpakete {
                ProduktpaketWriter::write(doc, pp);
            }
            for lz in &tx.lokationszuordnungen {
                LokationszuordnungWriter::write(doc, lz);
            }
            for g in &tx.seq_z01_groups {
                SeqGroupWriter::write_z01(doc, g);
            }
            let z18_in_generic = tx.generic_seq_groups.iter().any(|g| g.qualifier == "Z18");
            if !z18_in_generic {
                if let Some(ref v) = tx.vertrag {
                    VertragWriter::write(doc, v);
                }
            }
            for z in &tx.zaehler {
                ZaehlerWriter::write(doc, z);
            }
            for g in &tx.seq_z45_groups {
                SeqGroupWriter::write_z45(doc, g);
            }
            for g in &tx.seq_z71_groups {
                SeqGroupWriter::write_z71(doc, g);
            }
            for g in &tx.seq_z21_groups {
                SeqGroupWriter::write_z21(doc, g);
            }
            for g in &tx.seq_z08_groups {
                SeqGroupWriter::write_z08(doc, g);
            }
            for g in &tx.seq_z20_groups {
                SeqGroupWriter::write_z20(doc, g);
            }
            for g in &tx.generic_seq_groups {
                SeqGroupWriter::write_generic(doc, g);
            }
            if let Some(ref b) = tx.bilanzierung {
                BilanzierungWriter::write(doc, b);
            }
        } else {
            // Write any remaining zaehler/produktpakete not covered by order
            for z in tx.zaehler.get(zaehler_idx..).unwrap_or_default() {
                ZaehlerWriter::write(doc, z);
            }
            for pp in tx.produktpakete.get(produktpaket_idx..).unwrap_or_default() {
                ProduktpaketWriter::write(doc, pp);
            }
            for lz in tx
                .lokationszuordnungen
                .get(lokationszuordnung_idx..)
                .unwrap_or_default()
            {
                LokationszuordnungWriter::write(doc, lz);
            }
            if !bilanzierung_written {
                if let Some(ref b) = tx.bilanzierung {
                    BilanzierungWriter::write(doc, b);
                }
            }
            // Vertrag: only write from structured data if Z18 not captured as generic
            let z18_in_generic = tx.generic_seq_groups.iter().any(|g| g.qualifier == "Z18");
            if !z18_in_generic {
                if let Some(ref v) = tx.vertrag {
                    VertragWriter::write(doc, v);
                }
            }
        }

        // Replay Sequences passthrough (truly unmodeled SEQ groups)
        Self::replay_passthrough(doc, &tx.passthrough_segments, SegmentZone::Sequences);

        // SG12: NAD parties (Counter=0570)
        // Use nad_qualifier_order when available to preserve original ordering.
        // Fall back to MIG order: Z09, Z04, Z25, Z26, Z65 → NAD+DP → Z05, Z03, Z07, Z08, Z10
        if !tx.nad_qualifier_order.is_empty() {
            let mut party_written: Vec<bool> = vec![false; tx.parteien.len()];
            let mut address_written = false;
            for nad_q in &tx.nad_qualifier_order {
                if nad_q == "DP" || nad_q == "Z63" || nad_q == "Z59" || nad_q == "Z60" {
                    if !address_written {
                        for ml in &tx.marktlokationen {
                            MarktlokationWriter::write_address(doc, ml);
                        }
                        address_written = true;
                    }
                } else {
                    // Find next unwritten party with this qualifier
                    for (i, gp) in tx.parteien.iter().enumerate() {
                        if !party_written[i] {
                            let q = gp.edifact.nad_qualifier.as_deref().unwrap_or("Z04");
                            if q == nad_q {
                                GeschaeftspartnerWriter::write(doc, gp);
                                party_written[i] = true;
                                break;
                            }
                        }
                    }
                }
            }
            // Write any remaining parties not yet written
            for (i, gp) in tx.parteien.iter().enumerate() {
                if !party_written[i] {
                    GeschaeftspartnerWriter::write(doc, gp);
                }
            }
            // Write address if not yet written
            if !address_written {
                for ml in &tx.marktlokationen {
                    MarktlokationWriter::write_address(doc, ml);
                }
            }
        } else {
            // Fallback: MIG-ordered pre_dp/post_dp split
            let pre_dp_qualifiers = [
                "Z09", "Z04", "Z25", "Z26", "Z65", "Z66", "Z67", "Z68", "Z69", "Z70", "EO", "DDO",
            ];
            for gp in &tx.parteien {
                let q = gp.edifact.nad_qualifier.as_deref().unwrap_or("Z04");
                if pre_dp_qualifiers.contains(&q) {
                    GeschaeftspartnerWriter::write(doc, gp);
                }
            }
            // NAD+DP Marktlokationsanschrift (Nr 00518) — between pre-DP and post-DP parties
            for ml in &tx.marktlokationen {
                MarktlokationWriter::write_address(doc, ml);
            }
            // Post-DP party NADs (Z05, Z03, Z07, Z08, Z10)
            for gp in &tx.parteien {
                let q = gp.edifact.nad_qualifier.as_deref().unwrap_or("Z04");
                if !pre_dp_qualifiers.contains(&q) {
                    GeschaeftspartnerWriter::write(doc, gp);
                }
            }
        }
        // NAD+VY: Andere zugehörige Partei (other related party)
        if let Some(ref mp_id) = tx.prozessdaten.andere_partei_mp_id {
            let cq = tx
                .prozessdaten
                .andere_partei_code_qualifier
                .as_deref()
                .unwrap_or("9");
            let nad_value = format!("{}::{}", mp_id, cq);
            doc.write_segment("NAD", &["VY", &nad_value]);
            // RFF following NAD+VY
            if let Some(ref raw_rff) = tx.prozessdaten.andere_partei_rff {
                doc.write_raw_segment(raw_rff);
            }
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
        self.nachrichtendaten.una_decimal_mark = delimiters.decimal;
        self.delimiters = *delimiters;
        self.prozessdaten_mapper.set_delimiters(*delimiters);
        self.seq_group_mapper.set_delimiters(*delimiters);
        self.geschaeftspartner_mapper.set_delimiters(*delimiters);
        self.marktlokation_mapper.set_delimiters(*delimiters);
        self.technische_ressource_mapper.set_delimiters(*delimiters);
        self.steuerbare_ressource_mapper.set_delimiters(*delimiters);
        self.tranche_mapper.set_delimiters(*delimiters);
        self.mabis_zaehlpunkt_mapper.set_delimiters(*delimiters);
        self.zaehler_mapper.set_delimiters(*delimiters);
        self.produktpaket_mapper.set_delimiters(*delimiters);
        self.netzlokation_mapper.set_delimiters(*delimiters);
        self.messlokation_mapper.set_delimiters(*delimiters);
        self.bilanzierung_mapper.set_delimiters(*delimiters);
        self.lokationszuordnung_mapper.set_delimiters(*delimiters);
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
            "IDE" => {
                self.finalize_contact();
                self.handle_ide(segment);
            }
            "NAD" => {
                let q = segment.get_element(0);
                if q == "MS" || q == "MR" {
                    // Finalize any previous contact
                    self.finalize_contact();
                    self.handle_message_level_nad(segment);
                    // Track for CTA/COM that follows
                    self.expecting_contact_for = Some(q.to_string());
                    // Route to mappers (MarktlokationMapper uses NAD+MS for Sparte)
                    self.route_to_mappers_ex(segment, true);
                } else if q == "VY" && self.in_transaction {
                    // NAD+VY: Andere zugehörige Partei (other related party)
                    self.finalize_contact();
                    let mp_id = segment.get_component(1, 0);
                    let code_qualifier = segment.get_component(1, 2);
                    if !mp_id.is_empty() {
                        self.prozessdaten_mapper
                            .set_andere_partei(mp_id, code_qualifier);
                    }
                    self.after_nad_vy = true;
                    self.route_to_mappers_ex(segment, true);
                } else {
                    // Any other NAD finalizes contact tracking
                    self.finalize_contact();
                    // Track NAD qualifier order for roundtrip fidelity
                    if self.in_transaction {
                        self.nad_qualifier_order.push(q.to_string());
                    }
                    self.route_to_mappers(segment);
                }
            }
            "CTA" if !self.in_transaction => {
                self.handle_cta(segment);
            }
            "COM" if !self.in_transaction => {
                self.handle_com(segment);
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
            // LOC+Z22: Schlafende (dormant) Marktlokation
            "LOC" if self.in_transaction && segment.get_element(0) == "Z22" => {
                let id = segment.get_component(1, 0);
                if !id.is_empty() {
                    self.prozessdaten_mapper.set_schlafende_marktlokation(id);
                }
                // Also route to mappers in case any mapper wants it
                self.route_to_mappers_ex(segment, true);
            }
            "SEQ" if self.in_transaction => {
                // Once SEQ zone begins, ProzessdatenMapper stops claiming RFF/DTM
                self.prozessdaten_mapper.notify_seq_entered();
                self.route_to_mappers(segment);
            }
            _ => {
                // RFF after NAD+VY: capture as andere_partei_rff
                if segment.id == "RFF" && self.after_nad_vy {
                    self.after_nad_vy = false;
                    let raw = segment.to_raw_string(&self.delimiters);
                    self.prozessdaten_mapper.set_andere_partei_rff(&raw);
                    // Still route to mappers (coordinator_handled = true)
                    self.route_to_mappers_ex(segment, true);
                } else {
                    if self.after_nad_vy && segment.id != "RFF" {
                        self.after_nad_vy = false;
                    }
                    self.route_to_mappers(segment);
                }
            }
        }
        Control::Continue
    }

    fn on_message_end(&mut self, unt: &RawSegment) {
        // Collect the transaction if we have one
        if self.in_transaction || !self.prozessdaten_mapper.is_empty() {
            let tx = self.collect_transaction();
            self.transactions.push(tx);
            self.reset_mappers();
        }
        // Preserve original UNT count and reference for byte-identical roundtrip
        let unt_count = unt.get_element(0);
        if !unt_count.is_empty() {
            self.nachrichtendaten.raw_unt_count = Some(unt_count.to_string());
        }
        // Store original UNT reference (element 1) — may differ from UNH reference
        // or be empty. Use sentinel "" to distinguish "no reference" from "not captured".
        let unt_ref = unt.get_element(1);
        self.nachrichtendaten.raw_unt_reference = Some(unt_ref.to_string());
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
        let input = b"UNA:+.? 'UNB+UNOC:3+9900123000002:500+9900456000001:500+251217:1229+REF001'UNH+MSG001+UTILMD:D:11A:UN:S2.1'BGM+E03+DOC001'NAD+MS+9900123000002::293'NAD+MR+9900456000001::293'IDE+24+TXID001'LOC+Z16+DE00014545768S0000000000000003054'STS+7++E01'UNT+8+MSG001'UNZ+1+REF001'";

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

        // IMD is now handled by ProzessdatenMapper (stored in raw_imd), not passthrough.
        // Since IMD appears before IDE, it gets captured by the prozessdaten_mapper
        // at message level and folded into the first transaction.
        let tx_imd = &nachricht.transaktionen[0];
        assert!(
            !tx_imd.prozessdaten.raw_imd.is_empty(),
            "IMD should be captured in prozessdaten.raw_imd"
        );
        assert!(
            tx_imd.prozessdaten.raw_imd[0].starts_with("IMD"),
            "raw_imd should contain IMD segment"
        );

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
