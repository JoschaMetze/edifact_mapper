//! Auto-generated segment structs from MIG XML.
//! Do not edit manually.

#![allow(non_snake_case)]

use super::composites::*;
use super::enums::*;
use serde::{Deserialize, Serialize};

/// AGR segment — Es erfolgt die Angabe, ob ein entsprechender Vertrag zwischen dem Anschlussnutzer und MSB vorliegt. Zusätzlich kann auch noch angegeben werden, ob auch die Vertragsbeendigung mit dem vorigen Vertragspartner vor der Anmeldung beendet wurde und vorliegt. Entsprechend oft ist dieses Segment zu wiederholen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegAgr {
    pub cc543: CompositeC543,
}

/// BGM segment — Dieses Segment dient dazu, Typ und Funktion einer Nachricht anzuzeigen und die Identifikationsnummer zu übermitteln. DE1001: Es ist festgelegt, dass innerhalb einer Nachricht nur Meldungen eines Typs enthalten sein können. Alle einzelnen Vorgänge der Nachricht gehören daher der gleichen Kategorie an. In einer Nachricht sind bspw. also nur Anmeldungen oder Änderungen enthalten. Der Grund einer Meldung wird pro Vorgang im Transaktionsgrund beschrieben. Die Nutzung der Codes ist den entsprechenden Anwendungshandbüchern (= AHB) zu entnehmen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegBgm {
    pub cc002: CompositeC002,
    pub cc106: CompositeC106,
}

/// CAV segment — Die Codes der Produkteigenschaften sind in der Codeliste der Konfigurationen beschrieben
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegCav {
    pub cc889: CompositeC889,
}

/// CCI segment — Dieses Segment wird zur Angabe des Bilanzkreises benutzt. Hier muss der vom BDEW vergebene EIC-Code verwendet werden. Es wird der Bilanzkreis bzw. das Konto angegeben, auf dem die Bilanzierung durchgeführt wird.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegCci {
    pub d7059: D7059Qualifier,
    pub cc502: Option<CompositeC502>,
    pub cc240: CompositeC240,
}

/// COM segment — Zur Angabe einer Kommunikationsnummer einer Abteilung oder einer Person, die als Ansprechpartner dient.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegCom {
    pub cc076: CompositeC076,
}

/// CTA segment — Dieses Segment dient der Identifikation von Ansprechpartnern innerhalb des im vorangegangenen NAD-Segment spezifizierten Unternehmens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegCta {
    pub d3139: D3139Qualifier,
    pub cc056: CompositeC056,
}

/// DTM segment — Dieses Segment wird zur Angabe des Dokumentendatums verwendet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegDtm {
    pub cc507: CompositeC507,
}

/// FTX segment — Dieses Segment dient der Angabe von unformatierten oder codierten Textinformationen.  Hinweise: DE4440: Der in diesen Datenelementen enthaltene Text muss in Deutsch verfasst sein.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegFtx {
    pub d4451: D4451Qualifier,
    pub d4453: Option<String>,
    pub cc107: Option<CompositeC107>,
    pub cc108: CompositeC108,
}

/// IDE segment — Bemerkung:  Hinweis zu DE7402: Es ist zu beachten, dass eine Vorgangsnummer / Listennummer nicht im IDE+Z01 und in einem IDE+24 verwendet wird. Die Eindeutigkeit ist sowohl für das IDE+Z01 als auch für das IDE+24 übergreifend einzuhalten. Das bedeutet, eine bereits verwendete Nummer in einem IDE+Z01 darf in einem anderem IDE+24 nicht mehr genutzt werden.  In einem Use-Case (z. B. Übermittlung der Lieferantenclearingliste) kann es erforderlich sein, eine große Menge an Informationen übermitteln zu müssen, die aus technischen Gründen nicht in einer UTILMD Nachricht übertragen werden können. Um diesen Use- Case gerecht zu werden, ist es erforderlich die Informationen in mehrere UTILMD Nachrichten aufzuteilen (Details siehe UNH). Bei einer Aufteilung der Liste ist für die gesamte Liste eine Listennummer zu vergeben. Diese Listennummer ist bei einer Aufteilung in mehrere UTILMD Nachrichten in allen IDE+Z01 die zusammengefasst die gesamte Liste darstellen anzugeben.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegIde {
    pub d7495: D7495Qualifier,
    pub cc206: CompositeC206,
}

/// LOC segment — Dieses Segment wird zur Angabe der ID für den MaBiS-Zählpunkt benutzt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegLoc {
    pub d3227: D3227Qualifier,
    pub cc517: CompositeC517,
}

/// NAD segment — DE3039: Zur Identifikation der Marktpartner wird die MP-ID angegeben.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegNad {
    pub d3035: D3035Qualifier,
    pub cc082: CompositeC082,
}

/// PIA segment — Die Produkt-Codes sind in der Codeliste der Konfigurationen beschrieben
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegPia {
    pub d4347: D4347Qualifier,
    pub cc212: CompositeC212,
}

/// QTY segment — Dieses Segment wird zur Angabe der spezifischen Arbeit für eine tagesparameterabhängige Marktlokation als Zahlenwert in kWh/K und für die Angabe der angepassten elektrischen Arbeit einer tagesparameterabhängigen Marktlokation nach dem Verfahren der VDN- Richtlinie „Temperaturabhängiges Lastprofilverfahren bei unterbrechbaren Verbrauchseinrichtungen Anhang D (Dez. 2002)“ kurz: „vereinfachtes Verfahren“ als Zahlenwert in kWh angewendet. Die Arbeit schließt bei gemeinsam gemessenen Marktlokationen (SLP- und TLP-Verbrauchsanteil vorhanden) die ggf. in SG10 CCI+++E17 verlagerte Energiemenge nicht mit ein.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegQty {
    pub cc186: CompositeC186,
}

/// RFF segment — Die Verwendung dieses Segments wird im jeweiligen Anwendungshandbuch beschrieben.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegRff {
    pub cc506: CompositeC506,
}

/// SEQ segment — Dieses Segment wird benutzt, um die Segmentgruppe einzuleiten. Das Segment dient dazu die nachfolgenden Daten einem Meldepunkt zuzuordnen. Die SG8 Daten des MaBiS-ZP eines Vorgangs enthält alle Informationen, die sich auf eine einzelne MaBiS-ZP in einem Vorgang beziehen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegSeq {
    pub d1229: D1229Qualifier,
}

/// STS segment — DE9013 Diesem Datenelement werden Codes aus den Codelisten des Dokumentes „Entscheidungsbaum-Diagramme“ verwendet. Jeder Entscheidungsbaum gilt als Codeliste. Die relevante Codeliste wird im DE1131 angegeben. Somit sind nur die Codes in einem Anwendungsfall möglich, welche in dem zugehörigen Entscheidungsbaum aufgeführt sind.  DE1131 des Segments ist genutzt und enthält die Codes der Entscheidungsbaumdiagramme bzw. die Codes der im Dokument Entscheidungsbaum-Diagramme enthaltenen Code-Tabellen, die in der Nachricht verwendet werden.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegSts {
    pub cc601: CompositeC601,
    pub cc555: Option<CompositeC555>,
    pub cc556: CompositeC556,
}

/// UNA segment — Dieses Segment wird benutzt, um den Empfänger der Übertragungsdatei darüber zu unterrichten, dass andere Trennzeichen als die Standardtrennzeichen benutzt werden. Bei Anwendung der Standardtrennzeichen braucht das UNA-Segment nicht gesendet werden. Wenn es gesendet wird, muss es unmittelbar dem UNB-Segment vorangehen und die sechs vom Absender gewählten Trennzeichen enthalten. Unabhängig davon, ob alle Trennzeichen geändert wurden, muss jedes Datenelement innerhalb dieses Segmentes gefüllt werden, d. h. wenn Standardzeichen mit nutzerdefinierten Zeichen gemischt verwendet werden, müssen alle verwendeten Trennzeichen angegeben werden. Die Angabe der Trennzeichen im UNA-Segment erfolgt ohne Verwendung von Trennzeichen zwischen den Datenelementen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegUna {
    pub dUNA1: String,
    pub dUNA2: String,
    pub dUNA3: String,
    pub dUNA4: String,
    pub dUNA5: String,
    pub dUNA6: String,
}

/// UNB segment — Dieses Segment dient der Umklammerung der Übertragungsdatei, zur Identifikation des Partners, für den die Übertragungsdatei bestimmt ist und den Partner, der die Übertragungsdatei gesendet hat. Das Prinzip des UNB-Segments gleicht dem eines physischen Umschlags, der einen oder mehrere Briefe oder Dokumente enthält und die Adressen angibt, wohin geliefert werden soll und woher der Umschlag gekommen ist. DE0001: Der empfohlene (Standard-) Zeichensatz zur Anwendung in der BDEW-Spezifikation ist der Zeichensatz C (UNOC). Sollten Anwender einen anderen als den Zeichensatz C nutzen wollen, sollten sie vor dem Beginn des Datenaustauschs auf bilateraler Basis eine Vereinbarung schließen. DE0004 und DE0010: Die Verwendung von Internationalen Lokationsnummern (ILN => neue Bezeichnung ist GLN) zur Identifikation des Absenders und Empfängers der Übertragungsdatei wird (soweit bekannt) empfohlen. Wahlweise kann hierfür die BDEW-Codenummer des Geschäftspartners verwendet werden. DE0008: Die Adresse für Rückleitung stellt der Absender bereit, um den Empfänger der Übertragungsdatei über die Adresse im System des Absenders zu informieren, an die Antwortdateien gesendet werden müssen. DE0014: Die Weiterleitungsadresse, die ursprünglich vom Empfänger der Übertragungsdatei bereitgestellt wurde, wird vom Absender benutzt, um dem Empfänger die Adresse im System des Empfängers mitzuteilen, an die die Übertragungsdatei geleitet werden soll. Über die hier mitgeteilte Adresse hat der Empfänger der Übertragungsdatei den Absender vor der Datenübertragung zu informieren.  DES004: Datums- und Zeitangaben in dieser Datenelementgruppe entsprechen dem Datum und der Uhrzeit, an dem der Absender die Übertragungsdatei vorbereitete. Diese Datums- und Zeitangaben müssen nicht notwendigerweise mit den Datums- und Zeitangaben der enthaltenen Nachrichten übereinstimmen. DE0020: Die Datenaustauschreferenz wird vom Absender der Übertragungsdatei generiert und dient der eindeutigen Identifikation jeder Übertragungsdatei. Sollte der Absender der Übertragungsdatei Datenaustauschreferenzen wiederverwenden wollen, wird empfohlen, jede Nummer für mindestens drei Monate aufzubewahren, bevor sie wieder benutzt wird. Zur Sicherstellung der Eindeutigkeit sollte die Datenaustauschreferenz mit der Absenderidentifikation (DE0004) verbunden werden. DES005: Die Anwendung des Passworts muss zunächst von den Datenaustauschpartnern bilateral vereinbart werden. DE0026: Dieses Datenelement wird zur Identifikation des Anwendungsprogramms im System des Empfängers benutzt, an das die Übertragungsdatei geleitet wird. Dieses Datenelement darf nur benutzt werden, wenn die Übertragungsdatei nur einen Nachrichtentyp enthält. Die verwendete Referenz in diesem Datenelement wird vom Absender der Übertragungsdatei festgelegt. DE0031: Die BNetzA hat vorgegeben, dass die CONTRL immer versandt wird, daher ist eine Angabe in diesem Datenelement nicht erforderlich.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegUnb {
    pub d0020: String,
    pub d0026: Option<String>,
    pub d0029: Option<D0029Qualifier>,
    pub d0031: Option<String>,
    pub d0032: Option<String>,
    pub d0035: Option<D0035Qualifier>,
    pub cs001: CompositeS001,
    pub cs002: CompositeS002,
    pub cs003: CompositeS003,
    pub cs004: CompositeS004,
    pub cs005: Option<CompositeS005>,
}

/// UNH segment — Dieses Segment dient dazu, eine Nachricht zu eröffnen, zu identifizieren und zu spezifizieren. Hinweis: DE0057: Es werden nur die Versions- und Release-Nummern der Nachrichtenbeschreibungen angegeben. S010: Diese Datenelementgruppe wird benötigt, um bei größeren Listen, wie z. B. Zuordnungslisten, Lieferantenclearinglisten, die auf mehrere Nachrichten verteilt werden, klammern zu können. Jede Nachricht wird jeweils in einer Übertragungsdatei übertragen. DE0068 ff.: Wenn Listen (z. B. Zuordnungs- oder Lieferantenclearinglisten) aufgeteilt werden, ist dies entsprechend zu kennzeichnen. Wird eine Liste auf mehrere Nachrichten aufgeteilt, ist unter Berücksichtigung der technischen Restriktionen die maximal mögliche Segmentanzahl im UNH zu verwenden. Falls keine Aufteilung vorgenommen wird ist die Datenelementgruppe nicht zu verwenden. DE0068: Dieses Datenelement wird verwendet, um bei Nutzung der S010 eine Referenzierung zur ersten UTILMD-Datei (DE0020 aus dem UNB-Segment) der Übertragungsserie zu ermöglichen. DE0073: C = Creation / F = Final
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegUnh {
    pub d0062: String,
    pub d0068: Option<String>,
    pub cs009: CompositeS009,
    pub cs010: Option<CompositeS010>,
}

/// UNT segment — Das UNT-Segment ist ein Muss-Segment in UN/EDIFACT. Es muss immer das letzte Segment in einer Nachricht sein.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegUnt {
    pub d0074: String,
    pub d0062: String,
}

/// UNZ segment — Dieses Segment dient der Anzeige des Endes der Übertragungsdatei. DE0036: Falls Nachrichtengruppen verwendet werden, wird hier deren Anzahl in der Übertragungsdatei angegeben. Wenn keine Nachrichtengruppen verwendet werden, steht hier die Anzahl der Nachrichten in der Übertragungsdatei.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegUnz {
    pub d0036: String,
    pub d0020: String,
}
