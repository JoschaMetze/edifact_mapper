//! Auto-generated segment structs from MIG XML.
//! Do not edit manually.

#![allow(non_snake_case)]

use super::composites::*;
use super::enums::*;
use serde::{Deserialize, Serialize};

/// AGR segment — Es erfolgt die Angabe, ob ein entsprechender Vertrag zwischen dem Anschlussnutzer und MSB vorliegt. Zusätzlich kann auch noch angegeben werden, ob auch die Vertragsbeendigung mit dem vorigen Vertragspartner vor der Anmeldung beendet wurde und vorliegt. Entsprechend oft ist dieses Segment zu wiederholen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegAgr {
    /// Art der Vereinbarung, Identifikation
    pub cc543: CompositeC543,
}

/// BGM segment — Dieses Segment dient dazu, Typ und Funktion einer Nachricht anzuzeigen und die Identifikationsnummer zu übermitteln. DE1001: Es ist festgelegt, dass innerhalb einer Nachricht nur Meldungen eines Typs enthalten sein können. Alle einzelnen Vorgänge der Nachricht gehören daher der gleichen Kategorie an. In einer Nachricht sind bspw. also nur Anmeldungen oder Änderungen enthalten. Der Grund einer Meldung wird pro Vorgang im Transaktionsgrund beschrieben. Die Nutzung der Codes ist den entsprechenden Anwendungshandbüchern (= AHB) zu entnehmen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegBgm {
    /// Dokumenten-/Nachrichtenname
    pub cc002: CompositeC002,
    /// Dokumenten-/Nachrichten-Identifikation
    pub cc106: CompositeC106,
}

/// CAV segment — Die Codes der Produkteigenschaften sind in der Codeliste der Konfigurationen beschrieben
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegCav {
    /// Merkmalswert
    pub cc889: CompositeC889,
}

/// CCI segment —
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegCci {
    /// Klassentyp, Code
    pub d7059: D7059Qualifier,
    /// Relevanz für Zuordnung des LF
    pub d4051: D4051Qualifier,
    /// Einzelheiten zu Maßangaben
    pub cc502: Option<CompositeC502>,
    /// Merkmalsbeschreibung
    pub cc240: Option<CompositeC240>,
}

/// COM segment — Zur Angabe einer Kommunikationsnummer einer Abteilung oder einer Person, die als Ansprechpartner dient.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegCom {
    /// Kommunikationsverbindung
    pub cc076: CompositeC076,
}

/// CTA segment — Dieses Segment dient der Identifikation von Ansprechpartnern innerhalb des im vorangegangenen NAD-Segment spezifizierten Unternehmens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegCta {
    /// Funktion des Ansprechpartners, Code
    pub d3139: D3139Qualifier,
    /// Kontaktangaben
    pub cc056: CompositeC056,
}

/// DTM segment — Dieses Segment wird zur Angabe des Dokumentendatums verwendet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegDtm {
    /// Datum/Uhrzeit/Zeitspanne
    pub cc507: CompositeC507,
}

/// FTX segment — Dieses Segment dient der Angabe von unformatierten oder codierten Textinformationen.  Hinweise: DE4440: Der in diesen Datenelementen enthaltene Text muss in Deutsch verfasst sein.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegFtx {
    /// Textbezug, Qualifier
    pub d4451: D4451Qualifier,
    /// Textfunktion, Code
    pub d4453: Option<String>,
    /// Text-Referenz
    pub cc107: Option<CompositeC107>,
    /// Text
    pub cc108: CompositeC108,
}

/// IDE segment — Bemerkung:  Hinweis zu DE7402: Es ist zu beachten, dass eine Vorgangsnummer / Listennummer nicht im IDE+Z01 und in einem IDE+24 verwendet wird. Die Eindeutigkeit ist sowohl für das IDE+Z01 als auch für das IDE+24 übergreifend einzuhalten. Das bedeutet, eine bereits verwendete Nummer in einem IDE+Z01 darf in einem anderem IDE+24 nicht mehr genutzt werden.  In einem Use-Case (z. B. Übermittlung der Lieferantenclearingliste) kann es erforderlich sein, eine große Menge an Informationen übermitteln zu müssen, die aus technischen Gründen nicht in einer UTILMD Nachricht übertragen werden können. Um diesen Use- Case gerecht zu werden, ist es erforderlich die Informationen in mehrere UTILMD Nachrichten aufzuteilen (Details siehe UNH). Bei einer Aufteilung der Liste ist für die gesamte Liste eine Listennummer zu vergeben. Diese Listennummer ist bei einer Aufteilung in mehrere UTILMD Nachrichten in allen IDE+Z01 die zusammengefasst die gesamte Liste darstellen anzugeben.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegIde {
    /// Objekt, Qualifier
    pub d7495: D7495Qualifier,
    /// Identifikationsnummer
    pub cc206: CompositeC206,
}

/// LOC segment — Dieses Segment wird zur Angabe der ID für den MaBiS-Zählpunkt benutzt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegLoc {
    /// Mit dem Code Z15 in diesem Segment wird der MaBiS-Zählpunkt beschrieben
    pub d3227: D3227Qualifier,
    /// Ortsangabe
    pub cc517: CompositeC517,
}

/// NAD segment — In diesem Segment wird die Korrespondenzanschrift des Endverbrauchers/Kunden übertragen. Falls ein gesetzlicher Vertreter und/oder Bevollmächtigter eingesetzt ist, der dann auch in den zusätzlichen Namensangaben genannt ist, kann hier dessen Korrespondenzanschrift angegeben werden. Hierdurch ist gewährleistet, dass die Information über die Korrespondenzanschrift des Kunden bzw. des gesetzlichen Vertreters und/oder Bevollmächtigten übermittelt werden kann, da diese nicht mit der Marktlokationsadresse übereinstimmen muss. Weiterführende Informationen zur Anwendung der Datenelementgruppen C059 sind aus den Allgemeinen Festlegungen zu entnehmen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegNad {
    /// Beteiligter, Qualifier
    pub d3035: D3035Qualifier,
    /// Ortsname, Klartext
    pub d3164: String,
    /// Postleitzahl
    pub d3251: Option<String>,
    /// ISO 3166-1 = Alpha-2-Code
    pub d3207: String,
    /// Identifikation des Beteiligten
    pub cc082: Option<CompositeC082>,
    /// Name und Anschrift
    pub cc058: Option<CompositeC058>,
    /// Name des Beteiligten
    pub cc080: CompositeC080,
    /// Korrespondenzanschrift des Endverbrauchers/Kunden
    pub cc059: CompositeC059,
    /// Land-Untereinheit, Einzelheiten
    pub cc819: Option<CompositeC819>,
}

/// PIA segment — Die Produkt-Codes sind in der Codeliste der Konfigurationen beschrieben
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegPia {
    /// Produkt-/Erzeugnisnummer, Qualifier
    pub d4347: D4347Qualifier,
    /// Waren-/Leistungsnummer, Identifikation
    pub cc212: CompositeC212,
}

/// QTY segment — Dieses Segment wird zur Angabe der spezifischen Arbeit für eine tagesparameterabhängige Marktlokation als Zahlenwert in kWh/K und für die Angabe der angepassten elektrischen Arbeit einer tagesparameterabhängigen Marktlokation nach dem Verfahren der VDN- Richtlinie „Temperaturabhängiges Lastprofilverfahren bei unterbrechbaren Verbrauchseinrichtungen Anhang D (Dez. 2002)“ kurz: „vereinfachtes Verfahren“ als Zahlenwert in kWh angewendet. Die Arbeit schließt bei gemeinsam gemessenen Marktlokationen (SLP- und TLP-Verbrauchsanteil vorhanden) die ggf. in SG10 CCI+++E17 verlagerte Energiemenge nicht mit ein.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegQty {
    /// Mengenangaben
    pub cc186: CompositeC186,
}

/// RFF segment — Die Verwendung dieses Segments wird im jeweiligen Anwendungshandbuch beschrieben.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegRff {
    /// Referenz
    pub cc506: CompositeC506,
}

/// SEQ segment — Dieses Segment wird benutzt, um die Segmentgruppe einzuleiten. Die Segmentgruppe dient dazu, alle Lokationsbündelstrukturen, welche sich hinter einer Netzlokation befinden, anzugeben.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegSeq {
    /// Handlung, Code
    pub d1229: D1229Qualifier,
    /// Information über eine Folge
    pub cc286: Option<CompositeC286>,
}

/// STS segment — Zur Angabe eines Status. Dieses Segment wird benutzt um den Transaktionsgrund mitzuteilen. Der Transaktionsgrund beschreibt den Geschäftsvorfall zur Kategorie genauer. Dies dient der Plausibilisierung und Prozesssteuerung. Die Erläuterung zu den einzelnen Transaktionsgründen ist im DE9013 beschrieben.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegSts {
    /// Statuskategorie
    pub cc601: CompositeC601,
    /// Status
    pub cc555: Option<CompositeC555>,
    /// Statusanlaß
    pub cc556_1: CompositeC556,
    /// Statusanlaß
    pub cc556_2: Option<CompositeC556>,
    /// Statusanlaß
    pub cc556_3: Option<CompositeC556>,
}

/// UNA segment — Dieses Segment wird benutzt, um den Empfänger der Übertragungsdatei darüber zu unterrichten, dass andere Trennzeichen als die Standardtrennzeichen benutzt werden. Bei Anwendung der Standardtrennzeichen braucht das UNA-Segment nicht gesendet werden. Wenn es gesendet wird, muss es unmittelbar dem UNB-Segment vorangehen und die sechs vom Absender gewählten Trennzeichen enthalten. Unabhängig davon, ob alle Trennzeichen geändert wurden, muss jedes Datenelement innerhalb dieses Segmentes gefüllt werden, d. h. wenn Standardzeichen mit nutzerdefinierten Zeichen gemischt verwendet werden, müssen alle verwendeten Trennzeichen angegeben werden. Die Angabe der Trennzeichen im UNA-Segment erfolgt ohne Verwendung von Trennzeichen zwischen den Datenelementen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegUna {
    /// Wird verwendet als Trennzeichen zwischen Gruppendatenelementen innerhalb einer Datenelementgruppe (Standardwert : )
    pub dUNA1: String,
    /// Wird zur Trennung von zwei einfachen Datenelementen oder Gruppendatenelementen verwendet (Standardwert + )
    pub dUNA2: String,
    /// Wird zur Angabe des Dezimalzeichens verwendet (Standardwert . )
    pub dUNA3: String,
    /// Wird verwendet, um den Trennzeichen und dem Segment-Endezeichen ihre normale Bedeutung zurückzugeben (Standardwert ? )
    pub dUNA4: String,
    /// (Standardwert <Leerzeichen> )
    pub dUNA5: String,
    /// Wird zur Anzeige des Endes der Segmentdaten verwendet (Standardwert ' )
    pub dUNA6: String,
}

/// UNB segment — Dieses Segment dient der Umklammerung der Übertragungsdatei, zur Identifikation des Partners, für den die Übertragungsdatei bestimmt ist und den Partner, der die Übertragungsdatei gesendet hat. Das Prinzip des UNB-Segments gleicht dem eines physischen Umschlags, der einen oder mehrere Briefe oder Dokumente enthält und die Adressen angibt, wohin geliefert werden soll und woher der Umschlag gekommen ist. DE0001: Der empfohlene (Standard-) Zeichensatz zur Anwendung in der BDEW-Spezifikation ist der Zeichensatz C (UNOC). Sollten Anwender einen anderen als den Zeichensatz C nutzen wollen, sollten sie vor dem Beginn des Datenaustauschs auf bilateraler Basis eine Vereinbarung schließen. DE0004 und DE0010: Die Verwendung von Internationalen Lokationsnummern (ILN => neue Bezeichnung ist GLN) zur Identifikation des Absenders und Empfängers der Übertragungsdatei wird (soweit bekannt) empfohlen. Wahlweise kann hierfür die BDEW-Codenummer des Geschäftspartners verwendet werden. DE0008: Die Adresse für Rückleitung stellt der Absender bereit, um den Empfänger der Übertragungsdatei über die Adresse im System des Absenders zu informieren, an die Antwortdateien gesendet werden müssen. DE0014: Die Weiterleitungsadresse, die ursprünglich vom Empfänger der Übertragungsdatei bereitgestellt wurde, wird vom Absender benutzt, um dem Empfänger die Adresse im System des Empfängers mitzuteilen, an die die Übertragungsdatei geleitet werden soll. Über die hier mitgeteilte Adresse hat der Empfänger der Übertragungsdatei den Absender vor der Datenübertragung zu informieren.  DES004: Datums- und Zeitangaben in dieser Datenelementgruppe entsprechen dem Datum und der Uhrzeit, an dem der Absender die Übertragungsdatei vorbereitete. Diese Datums- und Zeitangaben müssen nicht notwendigerweise mit den Datums- und Zeitangaben der enthaltenen Nachrichten übereinstimmen. DE0020: Die Datenaustauschreferenz wird vom Absender der Übertragungsdatei generiert und dient der eindeutigen Identifikation jeder Übertragungsdatei. Sollte der Absender der Übertragungsdatei Datenaustauschreferenzen wiederverwenden wollen, wird empfohlen, jede Nummer für mindestens drei Monate aufzubewahren, bevor sie wieder benutzt wird. Zur Sicherstellung der Eindeutigkeit sollte die Datenaustauschreferenz mit der Absenderidentifikation (DE0004) verbunden werden. DES005: Die Anwendung des Passworts muss zunächst von den Datenaustauschpartnern bilateral vereinbart werden. DE0026: Dieses Datenelement wird zur Identifikation des Anwendungsprogramms im System des Empfängers benutzt, an das die Übertragungsdatei geleitet wird. Dieses Datenelement darf nur benutzt werden, wenn die Übertragungsdatei nur einen Nachrichtentyp enthält. Die verwendete Referenz in diesem Datenelement wird vom Absender der Übertragungsdatei festgelegt. DE0031: Die BNetzA hat vorgegeben, dass die CONTRL immer versandt wird, daher ist eine Angabe in diesem Datenelement nicht erforderlich.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegUnb {
    /// Eindeutige Referenz zur Identifikation der Übertragungsdatei, vergeben vom Absender.
    pub d0020: String,
    /// Nachrichtentyp, falls die Übertragungsdatei nur einen Nachrichtentyp enthält.
    pub d0026: Option<String>,
    /// Verarbeitungspriorität, Code
    pub d0029: Option<D0029Qualifier>,
    /// Bestätigungsanforderung
    pub d0031: Option<String>,
    /// Austauschvereinbarungskennung
    pub d0032: Option<String>,
    /// Test-Kennzeichen
    pub d0035: Option<D0035Qualifier>,
    /// Syntax-Bezeichner
    pub cs001: CompositeS001,
    /// Absender der Übertragungsdatei
    pub cs002: CompositeS002,
    /// Empfänger der Übertragungsdatei
    pub cs003: CompositeS003,
    /// Datum/Uhrzeit der Erstellung
    pub cs004: CompositeS004,
    /// Referenz/Passwort des Empfängers
    pub cs005: Option<CompositeS005>,
}

/// UNH segment — Dieses Segment dient dazu, eine Nachricht zu eröffnen, zu identifizieren und zu spezifizieren. Hinweis: DE0057: Es werden nur die Versions- und Release-Nummern der Nachrichtenbeschreibungen angegeben. S010: Diese Datenelementgruppe wird benötigt, um bei größeren Listen, wie z. B. Zuordnungslisten, Lieferantenclearinglisten, die auf mehrere Nachrichten verteilt werden, klammern zu können. Jede Nachricht wird jeweils in einer Übertragungsdatei übertragen. DE0068 ff.: Wenn Listen (z. B. Zuordnungs- oder Lieferantenclearinglisten) aufgeteilt werden, ist dies entsprechend zu kennzeichnen. Wird eine Liste auf mehrere Nachrichten aufgeteilt, ist unter Berücksichtigung der technischen Restriktionen die maximal mögliche Segmentanzahl im UNH zu verwenden. Falls keine Aufteilung vorgenommen wird ist die Datenelementgruppe nicht zu verwenden. DE0068: Dieses Datenelement wird verwendet, um bei Nutzung der S010 eine Referenzierung zur ersten UTILMD-Datei (DE0020 aus dem UNB-Segment) der Übertragungsserie zu ermöglichen. DE0073: C = Creation / F = Final
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegUnh {
    /// Eindeutige Nachrichtenreferenz in einer Nachricht des Absenders. Nummer der Nachrichten einer Übertragungsdatei im Datenaustausch. Identisch mit DE0062 im UNT, i. d. R. vom sendenden Konverter vergeben.
    pub d0062: String,
    /// Identifikation einer Übertragungsserie
    pub d0068: Option<String>,
    /// Nachrichten-Kennung
    pub cs009: CompositeS009,
    /// Status der Übermittlung
    pub cs010: Option<CompositeS010>,
}

/// UNT segment — Das UNT-Segment ist ein Muss-Segment in UN/EDIFACT. Es muss immer das letzte Segment in einer Nachricht sein.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegUnt {
    /// Hier wird die Gesamtzahl der Segmente einer Nachricht angegeben.
    pub d0074: String,
    /// Die Referenznummer aus dem UNH-Segment muss hier wiederholt werden.
    pub d0062: String,
}

/// UNZ segment — Dieses Segment dient der Anzeige des Endes der Übertragungsdatei. DE0036: Falls Nachrichtengruppen verwendet werden, wird hier deren Anzahl in der Übertragungsdatei angegeben. Wenn keine Nachrichtengruppen verwendet werden, steht hier die Anzahl der Nachrichten in der Übertragungsdatei.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegUnz {
    /// Anzahl der Nachrichten oder Nachrichtengruppen in der Übertragungsdatei
    pub d0036: String,
    /// Identisch mit DE0020 im UNB-Segment
    pub d0020: String,
}
