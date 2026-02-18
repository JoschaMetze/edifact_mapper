//! UTILMD-specific coordinator that orchestrates all mappers.
//!
//! Implements `EdifactHandler` and `Coordinator`. Routes segments to
//! registered mappers and collects built objects into `UtilmdTransaktion`.
//!
//! Mirrors C# `UtilmdCoordinator.cs`.

use std::marker::PhantomData;

use bo4e_extensions::{LinkRegistry, Marktteilnehmer, Nachrichtendaten, UtilmdTransaktion};
use edifact_parser::EdifactHandler;
use edifact_types::{Control, EdifactDelimiters, RawSegment};

use crate::context::TransactionContext;
use crate::coordinator::Coordinator;
use crate::error::AutomapperError;
use crate::mappers::*;
use crate::traits::{Builder, FormatVersion, SegmentHandler};
use crate::version::VersionConfig;

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
    }

    /// Handles IDE segment (transaction identifier).
    fn handle_ide(&mut self, segment: &RawSegment) {
        // IDE+24+transactionId'
        let qualifier = segment.get_element(0);
        if qualifier == "24" {
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

        match qualifier {
            "MS" => {
                if !mp_id.is_empty() {
                    self.absender.mp_id = Some(mp_id.to_string());
                    self.context.set_sender_mp_id(mp_id);
                }
            }
            "MR" => {
                if !mp_id.is_empty() {
                    self.empfaenger.mp_id = Some(mp_id.to_string());
                    self.context.set_recipient_mp_id(mp_id);
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
            steuerbare_ressourcen: Vec::new(),
            technische_ressourcen: Vec::new(),
            tranchen: Vec::new(),
            mabis_zaehlpunkte: Vec::new(),
            parteien,
            vertrag,
            bilanzierung: None,
            zaehler,
            produktpakete: Vec::new(),
            lokationszuordnungen: Vec::new(),
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
        self.in_transaction = false;
    }
}

impl<V: VersionConfig> Default for UtilmdCoordinator<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: VersionConfig> EdifactHandler for UtilmdCoordinator<V> {
    fn on_delimiters(&mut self, _delimiters: &EdifactDelimiters, _explicit_una: bool) {}

    fn on_interchange_start(&mut self, unb: &RawSegment) -> Control {
        // Extract sender/recipient from UNB
        let sender = unb.get_component(1, 0);
        if !sender.is_empty() {
            self.nachrichtendaten.absender_mp_id = Some(sender.to_string());
        }
        let recipient = unb.get_component(2, 0);
        if !recipient.is_empty() {
            self.nachrichtendaten.empfaenger_mp_id = Some(recipient.to_string());
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

    fn generate(&self, _transaktion: &UtilmdTransaktion) -> Result<Vec<u8>, AutomapperError> {
        // Will be implemented in Epic 8 (Writer)
        Ok(Vec::new())
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
