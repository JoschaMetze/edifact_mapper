# UTILMD MIG Segment Ordering (FV2504 / S2.1)

Source: `xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml`

This document defines the segment ordering used by the `generate()` method
in `UtilmdCoordinator`. The ordering is derived from the MIG XML Counter
attributes, which determine the transmission sequence of segments.

## How to re-derive this ordering

Run the following to extract the structure from the MIG XML:

```bash
python3 -c "
import xml.etree.ElementTree as ET
tree = ET.parse('xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml')
m = tree.getroot().find('M_UTILMD')
sg4s = m.findall('G_SG4')
sg4 = sg4s[1]  # Second SG4 = Vorgangs-Identifikation (IDE+24)
# Print all child segments/groups with Counter, Name, qualifier
"
```

The `Counter` attribute determines ordering. Elements with the same Counter
value appear in document order within the XML.

## Message envelope (outside SG4)

| Counter | Segment  | Qualifier | Name                        | Nr    |
|---------|----------|----------|-----------------------------|-------|
| 0000    | UNA      |          | Delimiter advice             | 00001 |
| 0000    | UNB      |          | Interchange header           | 00002 |
| 0010    | UNH      | UTILMD   | Message header               | 00003 |
| 0020    | BGM      | E01..E03 | Message type/kategorie       | 00004 |
| 0030    | DTM      | 137      | Nachrichtendatum             | 00005 |
| 0100    | SG2/NAD  | MS       | Absender MP-ID               | 00008 |
| 0100    | SG2/NAD  | MR       | Empfaenger MP-ID             | 00011 |
| ...     | SG4      |          | Transactions (see below)     |       |
| -       | UNT      |          | Message trailer              |       |
| -       | UNZ      |          | Interchange trailer          |       |

## SG4: Transaction (Vorgangs-Identifikation, IDE+24)

Counter=0180, MaxRep=99999

### Direct segments in SG4 (Counter order)

| Counter | Segment | Qualifier | Nr    | Name                                        |
|---------|---------|-----------|-------|---------------------------------------------|
| 0190    | IDE     | 24        | 00020 | Vorgang                                     |
| 0230    | DTM     | 76        | 00021 | Datum zum geplanten Leistungsbeginn         |
| 0230    | DTM     | 294       | 00022 | Datum und Uhrzeit der Übergabe              |
| 0230    | DTM     | 92        | 00023 | Beginn zum                                  |
| 0230    | DTM     | 93        | 00024 | Ende zum                                    |
| 0230    | DTM     | Z05       | 00025 | Datum bestätigtes Vertragsende              |
| 0230    | DTM     | 157       | 00026 | Änderung zum / Gültigkeit Beginndatum       |
| 0230    | DTM     | 471       | 00027 | Ende zum (nächstmöglich)                    |
| 0230    | DTM     | 158       | 00028 | Bilanzierungsbeginn                         |
| 0230    | DTM     | 159       | 00029 | Bilanzierungsende                           |
| 0230    | DTM     | 154       | 00030 | ÜT der Lieferanmeldung des LFN             |
| 0230    | DTM     | Z01       | 00031 | Kündigungsfrist des Vertrags                |
| 0230    | DTM     | Z10       | 00032 | Kündigungstermin des Vertrags               |
| 0230    | DTM     | Z07       | 00033 | Lieferbeginndatum in Bearbeitung            |
| 0230    | DTM     | Z08       | 00034 | Datum für nächste Bearbeitung               |
| 0250    | STS     | 7         | 00035 | Transaktionsgrund / Ergänzung               |
| 0250    | STS     | E01       | 00036 | Status der Antwort                          |
| 0250    | STS     | Z35       | 00037 | Status Antwort dritter Marktbeteiligter     |
| 0280    | FTX     | ACB       | 00038 | Bemerkung                                   |
| 0280    | FTX     | Z01       | 00039 | Profilbeschreibung                          |
| 0280    | FTX     | Z17..Z13  | 00040-46 | Various FTX types                       |
| 0290    | AGR     | 9         | 00047 | Beauftragung/Beendigung                     |

### SG5: Lokationen (Counter=0320)

All SG5 groups share Counter=0320. Within SG5, the LOC segment is at Counter=0330.
The MIG defines them in this qualifier order (Nr determines position):

| Nr    | LOC Qual | Name                      |
|-------|----------|---------------------------|
| 00048 | Z18      | Netzlokation              |
| 00049 | Z16      | Marktlokation             |
| 00050 | Z22      | Ruhende Marktlokation     |
| 00051 | Z20      | Technische Ressource      |
| 00052 | Z19      | Steuerbare Ressource      |
| 00053 | Z21      | Tranche                   |
| 00054 | Z17      | Messlokation              |
| 00055 | Z15      | MaBiS-Zählpunkt           |

### SG6: Referenzen (Counter=0350)

| Nr    | RFF Qual | Name                                        | Child DTMs         |
|-------|----------|---------------------------------------------|--------------------|
| 00056 | Z13      | Prüfidentifikator                           |                    |
| 00057 | TN       | Referenz Vorgangsnummer (aus Anfrage)        |                    |
| 00058 | ACW      | Referenz zu stornierende Vorgangsnummer       |                    |
| 00059 | AAV      | Referenz vorangegangene Anfrage              |                    |
| 00060 | Z42      | Referenznr. Nachricht betr. Antwort          |                    |
| 00061 | Z43      | Referenznr. Vorgang betr. Antwort            |                    |
| 00062 | Z60      | Geplantes Produktpaket                       |                    |
| 00063 | Z22      | Termine beteiligte Marktrolle                | DTM+Z15, DTM+Z16   |
| 00066 | Z47      | Verwendungszeitraum der Daten                | DTM+Z25, DTM+Z26   |
| 00069 | Z50      | Termine der Marktlokation                    | DTM+752,Z21,Z09,Z22|

### SG8: Sequenzgruppen (Counter=0410)

SG8 groups are identified by their SEQ qualifier. The MIG defines them in
this Nr/qualifier order:

| Nr    | SEQ Qual | Name                                              |
|-------|----------|---------------------------------------------------|
| 00074 | Z78      | Referenz Lokationsbündelstruktur                   |
| 00076 | Z58      | Zuordnung Lokation zum Objektcode                  |
| 00081 | Z79      | Bestandteil eines Produktpakets                    |
| 00086 | ZH0      | Priorisierung erforderliches Produktpaket          |
| 00089 | Z51      | Daten der Netzlokation                             |
| 00094 | Z71      | Abrechnungsdaten der Netzlokation                  |
| 00099 | Z57      | OBIS-Daten der Netzlokation                        |
| 00106 | Z60      | Produkt-Daten der Netzlokation                     |
| 00114 | Z01      | Daten der Marktlokation                            |
| 00158 | Z29      | Daten der Marktlokation beteiligte Marktrolle      |
| 00170 | Z45      | Netznutzungsabrechnungsdaten der Marktlokation     |
| 00181 | Z76      | Messstellenbetriebsabrechnungsdaten                |
| 00185 | Z27      | Erforderliches Messprodukt der Marktlokation       |
| 00195 | Z02      | OBIS-Daten der Marktlokation                       |
| 00208 | Z59      | Produkt-Daten der Marktlokation                    |
| 00214 | Z44      | Verbrauchsart/Nutzung OBIS-Kennzahl Marktlokation  |
| 00221 | Z30      | OBIS-Daten der Marktlokation beteiligte Marktrolle |
| 00223 | Z40      | Produkt-Daten der Marktlokation des NB             |
| 00225 | Z15      | Daten der Tranche                                  |
| 00236 | Z31      | Daten der Tranche beteiligte Marktrolle            |
| 00238 | Z16      | Erforderliches Messprodukt der Tranche             |
| 00247 | Z17      | OBIS-Daten der Tranche                             |
| 00256 | Z32      | OBIS-Daten der Tranche beteiligte Marktrolle       |
| 00258 | Z52      | Daten der Technischen Ressource                    |
| 00278 | Z62      | Daten der Steuerbaren Ressource                    |
| 00284 | Z61      | Produkt-Daten der Steuerbaren Ressource            |
| 00291 | Z18      | Daten der Messlokation                             |
| 00303 | Z19      | Erforderliches Messprodukt der Messlokation        |
| 00311 | Z03      | Zähleinrichtungsdaten                              |
| 00324 | Z20      | OBIS-Daten Zähleinrichtung / Smartmeter-Gateway    |
| 00338 | Z04      | Wandlerdaten                                       |
| 00343 | Z05      | Kommunikationseinrichtungsdaten                    |
| 00348 | Z06      | Daten technische Steuereinrichtung                 |
| 00353 | Z13      | Smartmeter-Gateway                                 |
| 00356 | Z14      | Daten der Steuerbox                                |
| 00361 | Z21      | Profildaten                                        |
| 00369 | Z33      | Profildaten beteiligte Marktrolle                  |
| 00372 | Z08      | Profilschardaten                                   |
| 00390 | Z38      | Referenzprofildaten                                |
| 00398 | Z22      | Daten der Summenzeitreihe                          |
| 00416 | Z23      | Produkt-Daten der Summenzeitreihe                  |
| 00422 | Z24      | Daten der Überführungszeitreihe                    |
| 00432 | Z25      | Produkt-Daten der Überführungszeitreihe            |
| 00437 | Z47      | Datenstand des ÜNB                                |
| 00453 | Z72      | Datenstand des NB                                  |
| 00466 | Z48      | Abgerechnete Daten Bilanzkreissummenzeitreihe      |
| 00491 | Z75      | Daten des Kunden des Lieferanten                   |

### SG12: Parteien / NAD (Counter=0570)

| Nr    | NAD Qual | Name                                      |
|-------|----------|-------------------------------------------|
| 00494 | Z09      | Kunde des Lieferanten                     |
| 00498 | Z04      | Korrespondenzanschrift Kunde LF           |
| 00501 | Z07      | Kunde des Messstellenbetreibers           |
| 00504 | Z08      | Korrespondenzanschrift Kunde MSB          |
| 00506 | Z25      | Kunde des Netzbetreibers                  |
| 00509 | Z26      | Korrespondenzanschrift Kunde NB           |
| 00512 | EO       | Anschlussnehmer                           |
| 00514 | DDO      | Hausverwalter                             |
| 00516 | VY       | Beteiligter Marktpartner MP-ID            |
| 00518 | DP       | Marktlokationsanschrift                   |
| 00520 | Z03      | Messlokationsadresse                      |
| 00523 | Z05      | Name und Adresse für Ablesekarte          |

## Simplified ordering for generate()

For the initial `generate()` implementation, we write segments in this order
(matching the MIG Counter ordering). Only entities with mappers AND writers
are included; others are skipped if their Vecs are empty.

```
UNA
UNB
UNH
BGM
DTM+137  (Nachrichtendatum from Nachrichtendaten)
NAD+MS   (Absender from Nachrichtendaten)
NAD+MR   (Empfänger from Nachrichtendaten)
--- per transaction (SG4) ---
  IDE+24              (transaktions_id)
  DTM+137             (prozessdatum)                     Ctr=0230
  DTM+471             (wirksamkeitsdatum)                Ctr=0230
  DTM+92              (vertragsbeginn)                   Ctr=0230
  DTM+93              (vertragsende)                     Ctr=0230
  DTM+Z07             (lieferbeginndatum_in_bearbeitung) Ctr=0230
  DTM+Z08             (datum_naechste_bearbeitung)       Ctr=0230
  STS+7               (transaktionsgrund/ergaenzung)     Ctr=0250
  FTX+ACB             (bemerkung)                        Ctr=0280
  --- SG5: Lokationen (Ctr=0320) ---
  LOC+Z18             (netzlokationen)                   Nr=00048
  LOC+Z16             (marktlokationen)                  Nr=00049
  LOC+Z20             (technische_ressourcen)             Nr=00051
  LOC+Z19             (steuerbare_ressourcen)             Nr=00052
  LOC+Z21             (tranchen)                          Nr=00053
  LOC+Z17             (messlokationen)                   Nr=00054
  LOC+Z15             (mabis_zaehlpunkte)                Nr=00055
  --- SG6: Referenzen (Ctr=0350) ---
  RFF+Z13             (referenz_vorgangsnummer)          Nr=00056
  RFF+Z47 + DTM+Z25/Z26  (zeitscheiben)                 Nr=00066
  --- SG8: Sequenzgruppen (Ctr=0410) ---
  SEQ+Z78             (lokationszuordnungen)             Nr=00074
  SEQ+Z79             (produktpakete)                    Nr=00081
  SEQ+Z98             (bilanzierung)                     Nr≈00200
  SEQ+Z03             (zaehler)                          Nr=00311
  SEQ+Z18             (messlokation/vertrag data)        Nr=00291
  --- SG12: Parteien (Ctr=0570) ---
  NAD+DP              (marktlokation address)            Nr=00518
  NAD+qualifier       (geschaeftspartner)                Nr=varies
--- end transactions ---
UNT
UNZ
```

## Notes on Zeitscheibe mapping to MIG

Zeitscheiben are mapped to **SG6 with RFF+Z47** (Verwendungszeitraum der Daten,
Nr=00066, Counter=0350). Each Zeitscheibe produces:
- `RFF+Z47:zeitscheiben_id`
- `DTM+Z25:von:303` (if gueltigkeitszeitraum.von present)
- `DTM+Z26:bis:303` (if gueltigkeitszeitraum.bis present)

The parser currently uses RFF+Z49/Z50/Z53 to identify Zeitscheiben. For
generation, we write RFF+Z47 which is the canonical MIG form for
"Verwendungszeitraum der Daten".

## Vertrag mapping to MIG

Vertrag maps to **SG8 with SEQ+Z18** (Daten der Messlokation, Nr=00291).
Note: In the MIG, SEQ+Z18 is "Daten der Messlokation" but in the current
codebase it's used for Vertrag data. This needs review against the C#
reference. The existing VertragWriter uses SEQ+Z18 with CCI segments.

## Version differences (FV2504 vs FV2510)

The segment ordering is identical between S2.1 (FV2504) and S2.2 (FV2510).
The difference is only in the UNH message type identifier:
- FV2504: `UTILMD:D:11A:UN:S2.1`
- FV2510: `UTILMD:D:11A:UN:S2.2`
