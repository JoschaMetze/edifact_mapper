//! Embedded seed data for generating realistic German names and addresses
//! in EDIFACT fixture files.

/// A coherent German postal address.
#[derive(Debug, Clone, Copy)]
pub struct SeedAddress {
    pub strasse: &'static str,
    pub hausnummer: &'static str,
    pub plz: &'static str,
    pub ort: &'static str,
    pub bundesland: &'static str,
}

/// 20 common German surnames.
pub static NACHNAMEN: &[&str] = &[
    "Mueller",
    "Schmidt",
    "Schneider",
    "Fischer",
    "Weber",
    "Meyer",
    "Wagner",
    "Becker",
    "Schulz",
    "Hoffmann",
    "Koch",
    "Richter",
    "Klein",
    "Wolf",
    "Schroeder",
    "Neumann",
    "Braun",
    "Zimmermann",
    "Hartmann",
    "Lange",
];

/// 20 common German first names.
pub static VORNAMEN: &[&str] = &[
    "Anna",
    "Thomas",
    "Maria",
    "Michael",
    "Julia",
    "Andreas",
    "Katharina",
    "Stefan",
    "Laura",
    "Markus",
    "Sarah",
    "Christian",
    "Lisa",
    "Daniel",
    "Sophie",
    "Matthias",
    "Lena",
    "Martin",
    "Hannah",
    "Jan",
];

/// Salutations: "Herr" and "Frau".
pub static ANREDEN: &[&str] = &["Herr", "Frau"];

/// Titles: mostly empty (no title), occasionally academic.
pub static TITEL: &[&str] = &["", "", "", "", "", "Dr.", "Prof.", "Dr.-Ing."];

/// 30 geographically coherent German addresses.
///
/// Each address has a real street name, plausible house number, correct 5-digit PLZ,
/// real city name, and matching 2-character Bundesland code.
pub static ADDRESSES: &[SeedAddress] = &[
    // Berlin (BE)
    SeedAddress {
        strasse: "Unter den Linden",
        hausnummer: "12",
        plz: "10117",
        ort: "Berlin",
        bundesland: "BE",
    },
    SeedAddress {
        strasse: "Kurfuerstendamm",
        hausnummer: "195",
        plz: "10707",
        ort: "Berlin",
        bundesland: "BE",
    },
    SeedAddress {
        strasse: "Friedrichstrasse",
        hausnummer: "43",
        plz: "10969",
        ort: "Berlin",
        bundesland: "BE",
    },
    // Bayern (BY)
    SeedAddress {
        strasse: "Maximilianstrasse",
        hausnummer: "8",
        plz: "80539",
        ort: "Muenchen",
        bundesland: "BY",
    },
    SeedAddress {
        strasse: "Ludwigstrasse",
        hausnummer: "28",
        plz: "80539",
        ort: "Muenchen",
        bundesland: "BY",
    },
    SeedAddress {
        strasse: "Koenigsstrasse",
        hausnummer: "55",
        plz: "90402",
        ort: "Nuernberg",
        bundesland: "BY",
    },
    // Nordrhein-Westfalen (NW)
    SeedAddress {
        strasse: "Schildergasse",
        hausnummer: "72",
        plz: "50667",
        ort: "Koeln",
        bundesland: "NW",
    },
    SeedAddress {
        strasse: "Koenigsallee",
        hausnummer: "30",
        plz: "40212",
        ort: "Duesseldorf",
        bundesland: "NW",
    },
    SeedAddress {
        strasse: "Westendhellweg",
        hausnummer: "1",
        plz: "44137",
        ort: "Dortmund",
        bundesland: "NW",
    },
    SeedAddress {
        strasse: "Limbecker Platz",
        hausnummer: "5",
        plz: "45127",
        ort: "Essen",
        bundesland: "NW",
    },
    // Hamburg (HH)
    SeedAddress {
        strasse: "Jungfernstieg",
        hausnummer: "7",
        plz: "20354",
        ort: "Hamburg",
        bundesland: "HH",
    },
    SeedAddress {
        strasse: "Moenckebergstrasse",
        hausnummer: "21",
        plz: "20095",
        ort: "Hamburg",
        bundesland: "HH",
    },
    // Hessen (HE)
    SeedAddress {
        strasse: "Zeil",
        hausnummer: "106",
        plz: "60313",
        ort: "Frankfurt am Main",
        bundesland: "HE",
    },
    SeedAddress {
        strasse: "Luisenplatz",
        hausnummer: "3",
        plz: "64283",
        ort: "Darmstadt",
        bundesland: "HE",
    },
    // Niedersachsen (NI)
    SeedAddress {
        strasse: "Georgstrasse",
        hausnummer: "14",
        plz: "30159",
        ort: "Hannover",
        bundesland: "NI",
    },
    SeedAddress {
        strasse: "Schlosswall",
        hausnummer: "10",
        plz: "38100",
        ort: "Braunschweig",
        bundesland: "NI",
    },
    // Baden-Wuerttemberg (BW)
    SeedAddress {
        strasse: "Koenigstrasse",
        hausnummer: "26",
        plz: "70173",
        ort: "Stuttgart",
        bundesland: "BW",
    },
    SeedAddress {
        strasse: "Hauptstrasse",
        hausnummer: "120",
        plz: "69117",
        ort: "Heidelberg",
        bundesland: "BW",
    },
    // Sachsen (SN)
    SeedAddress {
        strasse: "Grimmaische Strasse",
        hausnummer: "2",
        plz: "04109",
        ort: "Leipzig",
        bundesland: "SN",
    },
    SeedAddress {
        strasse: "Prager Strasse",
        hausnummer: "15",
        plz: "01069",
        ort: "Dresden",
        bundesland: "SN",
    },
    // Bremen (HB)
    SeedAddress {
        strasse: "Obernstrasse",
        hausnummer: "33",
        plz: "28195",
        ort: "Bremen",
        bundesland: "HB",
    },
    // Schleswig-Holstein (SH)
    SeedAddress {
        strasse: "Holstenstrasse",
        hausnummer: "43",
        plz: "24103",
        ort: "Kiel",
        bundesland: "SH",
    },
    // Thueringen (TH)
    SeedAddress {
        strasse: "Anger",
        hausnummer: "18",
        plz: "99084",
        ort: "Erfurt",
        bundesland: "TH",
    },
    SeedAddress {
        strasse: "Schillerstrasse",
        hausnummer: "4",
        plz: "99423",
        ort: "Weimar",
        bundesland: "TH",
    },
    // Brandenburg (BB)
    SeedAddress {
        strasse: "Brandenburger Strasse",
        hausnummer: "9",
        plz: "14467",
        ort: "Potsdam",
        bundesland: "BB",
    },
    // Rheinland-Pfalz (RP)
    SeedAddress {
        strasse: "Ludwigstrasse",
        hausnummer: "15",
        plz: "55116",
        ort: "Mainz",
        bundesland: "RP",
    },
    SeedAddress {
        strasse: "Sternstrasse",
        hausnummer: "22",
        plz: "56068",
        ort: "Koblenz",
        bundesland: "RP",
    },
    // Sachsen-Anhalt (ST)
    SeedAddress {
        strasse: "Grosse Ulrichstrasse",
        hausnummer: "51",
        plz: "06108",
        ort: "Halle",
        bundesland: "ST",
    },
    // Mecklenburg-Vorpommern (MV)
    SeedAddress {
        strasse: "Kroepeliner Strasse",
        hausnummer: "38",
        plz: "18055",
        ort: "Rostock",
        bundesland: "MV",
    },
    // Saarland (SL)
    SeedAddress {
        strasse: "Bahnhofstrasse",
        hausnummer: "67",
        plz: "66111",
        ort: "Saarbruecken",
        bundesland: "SL",
    },
];

/// Deterministic selection from a slice using a seed value.
///
/// Same seed always returns the same element.
pub fn pick<T>(items: &[T], seed: u64) -> &T {
    &items[pick_index(items.len(), seed)]
}

/// Deterministic index selection for correlated picks.
///
/// Returns an index in `0..len` derived from `seed`. Same (len, seed) pair
/// always produces the same index.
pub fn pick_index(len: usize, seed: u64) -> usize {
    assert!(len > 0, "pick_index: slice must not be empty");
    (seed as usize) % len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nachnamen_non_empty() {
        assert!(!NACHNAMEN.is_empty());
        assert_eq!(NACHNAMEN.len(), 20);
    }

    #[test]
    fn test_vornamen_non_empty() {
        assert!(!VORNAMEN.is_empty());
        assert_eq!(VORNAMEN.len(), 20);
    }

    #[test]
    fn test_anreden_non_empty() {
        assert!(!ANREDEN.is_empty());
        assert_eq!(ANREDEN.len(), 2);
    }

    #[test]
    fn test_titel_non_empty() {
        assert!(!TITEL.is_empty());
        assert_eq!(TITEL.len(), 8);
    }

    #[test]
    fn test_addresses_non_empty() {
        assert!(!ADDRESSES.is_empty());
        assert_eq!(ADDRESSES.len(), 30);
    }

    #[test]
    fn test_all_addresses_have_five_digit_plz() {
        for (i, addr) in ADDRESSES.iter().enumerate() {
            assert_eq!(
                addr.plz.len(),
                5,
                "Address {} ({}) has PLZ '{}' which is not 5 digits",
                i,
                addr.ort,
                addr.plz
            );
            assert!(
                addr.plz.chars().all(|c| c.is_ascii_digit()),
                "Address {} ({}) has non-digit PLZ '{}'",
                i,
                addr.ort,
                addr.plz
            );
        }
    }

    #[test]
    fn test_all_addresses_have_two_char_bundesland() {
        for (i, addr) in ADDRESSES.iter().enumerate() {
            assert_eq!(
                addr.bundesland.len(),
                2,
                "Address {} ({}) has Bundesland '{}' which is not 2 chars",
                i,
                addr.ort,
                addr.bundesland
            );
        }
    }

    #[test]
    fn test_all_addresses_have_non_empty_fields() {
        for (i, addr) in ADDRESSES.iter().enumerate() {
            assert!(!addr.strasse.is_empty(), "Address {} has empty strasse", i);
            assert!(
                !addr.hausnummer.is_empty(),
                "Address {} has empty hausnummer",
                i
            );
            assert!(!addr.ort.is_empty(), "Address {} has empty ort", i);
        }
    }

    #[test]
    fn test_pick_is_deterministic() {
        let a = pick(NACHNAMEN, 42);
        let b = pick(NACHNAMEN, 42);
        assert_eq!(a, b, "pick must return the same value for the same seed");

        let c = pick(VORNAMEN, 123);
        let d = pick(VORNAMEN, 123);
        assert_eq!(c, d, "pick must return the same value for the same seed");

        let e = pick(ADDRESSES, 7);
        let f = pick(ADDRESSES, 7);
        assert_eq!(
            e.plz, f.plz,
            "pick must return the same address for the same seed"
        );
    }

    #[test]
    fn test_pick_different_seeds_vary() {
        // Different seeds should (in general) produce different results
        // for a large enough collection
        let a = pick(NACHNAMEN, 0);
        let b = pick(NACHNAMEN, 7);
        // We just check they are valid; they might be the same by chance
        assert!(!a.is_empty());
        assert!(!b.is_empty());
    }

    #[test]
    fn test_pick_index_deterministic() {
        let a = pick_index(30, 42);
        let b = pick_index(30, 42);
        assert_eq!(a, b);
    }

    #[test]
    fn test_pick_index_in_range() {
        for seed in 0..100 {
            let idx = pick_index(30, seed);
            assert!(
                idx < 30,
                "pick_index({}, {}) = {} which is out of range",
                30,
                seed,
                idx
            );
        }
    }

    #[test]
    #[should_panic(expected = "pick_index: slice must not be empty")]
    fn test_pick_index_panics_on_empty() {
        pick_index(0, 42);
    }
}
