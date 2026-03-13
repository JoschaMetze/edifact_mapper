//! Concrete [`ExternalConditionProvider`] implementations.
//!
//! - [`MapExternalProvider`]: wraps a `HashMap<String, bool>` for simple lookup.
//! - [`CompositeExternalProvider`]: chains multiple providers, returning the first
//!   non-[`Unknown`](super::ConditionResult::Unknown) result.

use std::collections::{HashMap, HashSet};

use super::evaluator::{ConditionResult, ExternalConditionProvider};

/// Countries where postal codes (PLZ) are required in the EU/EEA energy market.
const COUNTRIES_WITH_PLZ: &[&str] = &[
    "DE", "AT", "CH", "BE", "BG", "CZ", "DK", "EE", "FI", "FR", "GR", "HR", "HU", "IE", "IT", "LT",
    "LU", "LV", "MT", "NL", "PL", "PT", "RO", "SE", "SI", "SK", "ES", "GB", "NO", "CY", "IS", "LI",
];

const COUNTRY_PLZ_CONDITIONS: &[&str] = &[
    "country_code_has_plz",
    "country_has_postal_code",
    "country_has_postal_code_requirement",
];

/// Provider that resolves country→postal code (PLZ) conditions.
///
/// Returns True if the country code is in the set of countries with PLZ requirements,
/// False if it's a known country without PLZ, Unknown if no country code is set.
pub struct CountryPostalCodeProvider {
    country_code: Option<String>,
}

impl CountryPostalCodeProvider {
    /// Create a new provider with the given ISO 3166-1 alpha-2 country code.
    pub fn new(country_code: Option<String>) -> Self {
        Self { country_code }
    }
}

impl ExternalConditionProvider for CountryPostalCodeProvider {
    fn evaluate(&self, condition_name: &str) -> ConditionResult {
        if !COUNTRY_PLZ_CONDITIONS.contains(&condition_name) {
            return ConditionResult::Unknown;
        }
        match &self.country_code {
            Some(code) => ConditionResult::from(COUNTRIES_WITH_PLZ.contains(&code.as_str())),
            None => ConditionResult::Unknown,
        }
    }
}

/// An [`ExternalConditionProvider`] backed by a `HashMap<String, bool>`.
///
/// Returns `True`/`False` for keys present in the map and `Unknown` for
/// missing keys. This is the simplest way for API callers to supply
/// external condition values.
///
/// # Example
///
/// ```
/// use std::collections::HashMap;
/// use automapper_validation::{MapExternalProvider, ConditionResult};
/// use automapper_validation::eval::ExternalConditionProvider;
///
/// let mut conditions = HashMap::new();
/// conditions.insert("DateKnown".to_string(), true);
///
/// let provider = MapExternalProvider::new(conditions);
/// assert_eq!(provider.evaluate("DateKnown"), ConditionResult::True);
/// assert_eq!(provider.evaluate("Unknown"), ConditionResult::Unknown);
/// ```
pub struct MapExternalProvider {
    conditions: HashMap<String, bool>,
}

impl MapExternalProvider {
    /// Creates a new `MapExternalProvider` from the given condition map.
    pub fn new(conditions: HashMap<String, bool>) -> Self {
        Self { conditions }
    }
}

impl ExternalConditionProvider for MapExternalProvider {
    fn evaluate(&self, condition_name: &str) -> ConditionResult {
        match self.conditions.get(condition_name) {
            Some(true) => ConditionResult::True,
            Some(false) => ConditionResult::False,
            None => ConditionResult::Unknown,
        }
    }
}

/// An [`ExternalConditionProvider`] that delegates to multiple providers in order.
///
/// For each `evaluate()` call, providers are consulted in sequence. The first
/// provider that returns a non-[`Unknown`](ConditionResult::Unknown) result wins.
/// If all providers return `Unknown` (or there are no providers), `Unknown` is
/// returned.
///
/// This is useful for layering: e.g., a caller-supplied map on top of a
/// system-default provider.
pub struct CompositeExternalProvider {
    providers: Vec<Box<dyn ExternalConditionProvider>>,
}

impl CompositeExternalProvider {
    /// Creates a new `CompositeExternalProvider` from the given provider list.
    ///
    /// Providers are consulted in the order they appear in the vector.
    pub fn new(providers: Vec<Box<dyn ExternalConditionProvider>>) -> Self {
        Self { providers }
    }

    /// Build a composite provider with the standard static providers.
    ///
    /// - `sector`: If Some, adds a `SectorProvider`
    /// - `roles`: If Some((sender_roles, recipient_roles)), adds a `MarketRoleProvider`
    /// - `code_list_json`: If Some, adds a `CodeListProvider` loaded from JSON
    /// - `konfigurationen_json`: If Some((json, product_code)), adds a `KonfigurationenProvider`
    pub fn with_defaults(
        sector: Option<Sector>,
        roles: Option<(Vec<MarketRole>, Vec<MarketRole>)>,
        code_list_json: Option<&str>,
    ) -> Self {
        Self::builder()
            .sector(sector)
            .roles(roles)
            .code_list_json(code_list_json)
            .build()
    }

    /// Start building a composite provider with fine-grained control.
    pub fn builder() -> CompositeProviderBuilder {
        CompositeProviderBuilder::default()
    }
}

/// Builder for constructing a [`CompositeExternalProvider`] with multiple provider types.
#[derive(Default)]
pub struct CompositeProviderBuilder {
    sector: Option<Sector>,
    roles: Option<(Vec<MarketRole>, Vec<MarketRole>)>,
    code_list_json: Option<String>,
    konfigurationen_json: Option<String>,
    product_code: Option<String>,
    country_code: Option<String>,
}

impl CompositeProviderBuilder {
    pub fn sector(mut self, sector: Option<Sector>) -> Self {
        self.sector = sector;
        self
    }

    pub fn roles(mut self, roles: Option<(Vec<MarketRole>, Vec<MarketRole>)>) -> Self {
        self.roles = roles;
        self
    }

    pub fn code_list_json(mut self, json: Option<&str>) -> Self {
        self.code_list_json = json.map(String::from);
        self
    }

    pub fn konfigurationen_json(mut self, json: &str) -> Self {
        self.konfigurationen_json = Some(json.to_string());
        self
    }

    pub fn product_code(mut self, code: Option<String>) -> Self {
        self.product_code = code;
        self
    }

    pub fn country_code(mut self, code: Option<String>) -> Self {
        self.country_code = code;
        self
    }

    pub fn build(self) -> CompositeExternalProvider {
        let mut providers: Vec<Box<dyn ExternalConditionProvider>> = Vec::new();

        if let Some(sector) = self.sector {
            providers.push(Box::new(SectorProvider::new(sector)));
        }
        if let Some((sender, recipient)) = self.roles {
            providers.push(Box::new(MarketRoleProvider::new(sender, recipient)));
        }
        if let Some(json) = &self.code_list_json {
            if let Ok(provider) = CodeListProvider::from_json(json) {
                providers.push(Box::new(provider));
            }
        }
        if let Some(json) = &self.konfigurationen_json {
            if let Ok(provider) = KonfigurationenProvider::from_json(json, self.product_code) {
                providers.push(Box::new(provider));
            }
        }
        if self.country_code.is_some() {
            providers.push(Box::new(CountryPostalCodeProvider::new(self.country_code)));
        }

        CompositeExternalProvider::new(providers)
    }
}

impl ExternalConditionProvider for CompositeExternalProvider {
    fn evaluate(&self, condition_name: &str) -> ConditionResult {
        for provider in &self.providers {
            let result = provider.evaluate(condition_name);
            if !result.is_unknown() {
                return result;
            }
        }
        ConditionResult::Unknown
    }
}

/// Provider that checks whether a value belongs to a known code list.
///
/// Condition name format: `"code_in_<data_element_id>:<value>"`
/// Returns True if the value is in the list, False if not, Unknown if the list is unknown.
pub struct CodeListProvider {
    lists: HashMap<String, HashSet<String>>,
}

impl CodeListProvider {
    pub fn new(lists: HashMap<String, Vec<String>>) -> Self {
        Self {
            lists: lists
                .into_iter()
                .map(|(k, v)| (k, v.into_iter().collect()))
                .collect(),
        }
    }

    /// Load from the JSON format produced by `extract-code-lists` CLI.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        #[derive(serde::Deserialize)]
        struct Entry {
            codes: Vec<CodeValue>,
        }
        #[derive(serde::Deserialize)]
        struct CodeValue {
            value: String,
        }

        let raw: HashMap<String, Entry> = serde_json::from_str(json)?;
        let lists = raw
            .into_iter()
            .map(|(k, v)| (k, v.codes.into_iter().map(|c| c.value).collect()))
            .collect();
        Ok(Self { lists })
    }

    /// Load from a JSON file path.
    pub fn from_json_file(
        path: &std::path::Path,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let json = std::fs::read_to_string(path)?;
        Ok(Self::from_json(&json)?)
    }
}

impl ExternalConditionProvider for CodeListProvider {
    fn evaluate(&self, condition_name: &str) -> ConditionResult {
        let rest = match condition_name.strip_prefix("code_in_") {
            Some(r) => r,
            None => return ConditionResult::Unknown,
        };
        let (de_id, value) = match rest.split_once(':') {
            Some(pair) => pair,
            None => return ConditionResult::Unknown,
        };
        match self.lists.get(de_id) {
            Some(set) => ConditionResult::from(set.contains(value)),
            None => ConditionResult::Unknown,
        }
    }
}

/// Energy sector (Strom = electricity, Gas = natural gas).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sector {
    Strom,
    Gas,
}

/// Provider that resolves sector-based conditions from deployment configuration.
pub struct SectorProvider {
    sector: Sector,
}

impl SectorProvider {
    pub fn new(sector: Sector) -> Self {
        Self { sector }
    }

    /// Create from variant string (e.g., "Strom", "Gas").
    pub fn from_variant(variant: &str) -> Option<Self> {
        match variant {
            "Strom" => Some(Self::new(Sector::Strom)),
            "Gas" => Some(Self::new(Sector::Gas)),
            _ => None,
        }
    }
}

const STROM_CONDITIONS: &[&str] = &[
    "recipient_is_strom",
    "recipient_market_sector_is_strom",
    "recipient_is_electricity_sector",
    "sender_is_strom",
    "mp_id_is_strom",
    "mp_id_is_strom_sector",
    "mp_id_is_electricity_sector",
    "mp_id_from_electricity_sector",
    "mp_id_only_strom",
    "mp_id_strom_only",
    "mp_id_sparte_strom",
    "market_location_is_electricity",
    "marktpartner_is_strom",
    "location_is_strom",
    "metering_point_is_strom",
    "network_location_is_strom",
];

const GAS_CONDITIONS: &[&str] = &[
    "recipient_is_gas",
    "recipient_market_sector_is_gas",
    "recipient_is_gas_sector",
    "sender_is_gas",
    "sender_is_gas_sector",
    "mp_id_is_gas",
    "mp_id_is_gas_sector",
    "market_location_is_gas",
    "marktpartner_is_gas",
    "location_is_gas",
    "metering_point_is_gas",
    "network_location_is_gas",
    "recipient_is_msb_gas",
];

impl ExternalConditionProvider for SectorProvider {
    fn evaluate(&self, condition_name: &str) -> ConditionResult {
        if STROM_CONDITIONS.contains(&condition_name) {
            return ConditionResult::from(self.sector == Sector::Strom);
        }
        if GAS_CONDITIONS.contains(&condition_name) {
            return ConditionResult::from(self.sector == Sector::Gas);
        }
        ConditionResult::Unknown
    }
}

/// Market participant roles in the German energy market.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarketRole {
    /// Lieferant (supplier)
    LF,
    /// Netzbetreiber (network operator)
    NB,
    /// Messstellenbetreiber (metering point operator)
    MSB,
    /// Messdienstleister (metering service provider)
    MDL,
    /// Übertragungsnetzbetreiber (TSO)
    UENB,
    /// Bilanzkreisverantwortlicher
    BKV,
    /// Bilanzkoordinator
    BIKO,
    /// Einsatzverantwortlicher
    ESA,
    /// Marktgebietsverantwortlicher (market area manager)
    MGV,
    /// Kunde (customer)
    KN,
}

impl MarketRole {
    fn from_suffix(s: &str) -> Option<Self> {
        match s {
            "lf" => Some(Self::LF),
            "nb" => Some(Self::NB),
            "msb" => Some(Self::MSB),
            "mdl" => Some(Self::MDL),
            "uenb" => Some(Self::UENB),
            "bkv" => Some(Self::BKV),
            "biko" => Some(Self::BIKO),
            "esa" => Some(Self::ESA),
            "mgv" => Some(Self::MGV),
            "kn" => Some(Self::KN),
            _ => None,
        }
    }

    /// Parse a compound role suffix like "lf_msb_nb" or "lf_or_nb" into multiple roles.
    /// Tries `_or_` splitting first, then falls back to `_` splitting.
    /// Returns None if any component is unrecognized.
    fn parse_compound(s: &str) -> Option<Vec<Self>> {
        // Try splitting on "_or_" first (e.g., "lf_or_nb")
        if s.contains("_or_") {
            let parts: Vec<&str> = s.split("_or_").collect();
            let mut roles = Vec::new();
            for part in parts {
                roles.push(Self::from_suffix(part)?);
            }
            if !roles.is_empty() {
                return Some(roles);
            }
        }
        // Fall back to splitting on "_" (e.g., "lf_msb_nb")
        let parts: Vec<&str> = s.split('_').collect();
        let mut roles = Vec::new();
        for part in parts {
            roles.push(Self::from_suffix(part)?);
        }
        if roles.is_empty() {
            None
        } else {
            Some(roles)
        }
    }
}

/// Provider that resolves sender/recipient market role conditions.
pub struct MarketRoleProvider {
    sender_roles: HashSet<MarketRole>,
    recipient_roles: HashSet<MarketRole>,
}

impl MarketRoleProvider {
    pub fn new(sender_roles: Vec<MarketRole>, recipient_roles: Vec<MarketRole>) -> Self {
        Self {
            sender_roles: sender_roles.into_iter().collect(),
            recipient_roles: recipient_roles.into_iter().collect(),
        }
    }
}

impl ExternalConditionProvider for MarketRoleProvider {
    fn evaluate(&self, condition_name: &str) -> ConditionResult {
        // Single role: sender_is_lf, recipient_is_nb
        if let Some(role_str) = condition_name.strip_prefix("sender_is_") {
            if let Some(role) = MarketRole::from_suffix(role_str) {
                return ConditionResult::from(self.sender_roles.contains(&role));
            }
            // Compound: sender_is_lf_nb = sender is LF OR NB
            if let Some(roles) = MarketRole::parse_compound(role_str) {
                return ConditionResult::from(roles.iter().any(|r| self.sender_roles.contains(r)));
            }
        }
        if let Some(role_str) = condition_name.strip_prefix("recipient_is_") {
            if let Some(role) = MarketRole::from_suffix(role_str) {
                return ConditionResult::from(self.recipient_roles.contains(&role));
            }
            // Compound: recipient_is_lf_msb = recipient is LF OR MSB
            if let Some(roles) = MarketRole::parse_compound(role_str) {
                return ConditionResult::from(
                    roles.iter().any(|r| self.recipient_roles.contains(r)),
                );
            }
        }
        if let Some(role_str) = condition_name.strip_prefix("sender_role_is_not_") {
            if let Some(role) = MarketRole::from_suffix(role_str) {
                return ConditionResult::from(!self.sender_roles.contains(&role));
            }
        }
        if let Some(role_str) = condition_name.strip_prefix("recipient_role_is_not_") {
            if let Some(role) = MarketRole::from_suffix(role_str) {
                return ConditionResult::from(!self.recipient_roles.contains(&role));
            }
        }
        // recipient_not_<roles> = recipient is NOT any of the given roles
        if let Some(role_str) = condition_name.strip_prefix("recipient_not_") {
            if let Some(role) = MarketRole::from_suffix(role_str) {
                return ConditionResult::from(!self.recipient_roles.contains(&role));
            }
            if let Some(roles) = MarketRole::parse_compound(role_str) {
                return ConditionResult::from(
                    !roles.iter().any(|r| self.recipient_roles.contains(r)),
                );
            }
        }
        // sender_not_<roles> = sender is NOT any of the given roles
        if let Some(role_str) = condition_name.strip_prefix("sender_not_") {
            if let Some(role) = MarketRole::from_suffix(role_str) {
                return ConditionResult::from(!self.sender_roles.contains(&role));
            }
            if let Some(roles) = MarketRole::parse_compound(role_str) {
                return ConditionResult::from(!roles.iter().any(|r| self.sender_roles.contains(r)));
            }
        }
        // mp_id_role_is_<role> — treated like sender role check
        if let Some(role_str) = condition_name.strip_prefix("mp_id_role_is_") {
            if let Some(role) = MarketRole::from_suffix(role_str) {
                return ConditionResult::from(self.sender_roles.contains(&role));
            }
        }
        ConditionResult::Unknown
    }
}

/// Provider that resolves product/Konfigurationen conditions based on a product code.
///
/// Loads the categorized product codes from the Codeliste der Konfigurationen and checks
/// whether a given product code belongs to specific categories.
///
/// Supports conditions like:
/// - `messprodukt_standard_marktlokation` — product is in chapter 2.1
/// - `messprodukt_standard_tranche` — product is in chapter 2.2
/// - `config_product_leistungskurve` — product is the Leistungskurve product
/// - `code_list_membership_check` — product is in any category of the Konfigurationen
///
/// Aliases that map alternative condition names to Konfigurationen lookups.
enum KonfigurationenAlias {
    /// Check against all_codes (same as code_list_membership_check).
    AllCodes,
    /// Check against a single category by canonical name.
    Category(&'static str),
    /// Check against the union of all messprodukt_* categories.
    MessproduktUnion,
}

fn resolve_konfigurationen_alias(condition_name: &str) -> Option<KonfigurationenAlias> {
    match condition_name {
        "is_konfigurationsprodukt_code" | "lin_prefix_is_valid_product_code" => {
            Some(KonfigurationenAlias::AllCodes)
        }
        "is_messprodukt_code" => Some(KonfigurationenAlias::MessproduktUnion),
        "messprodukts_typ2_smgw"
        | "product_in_typ2_smgw_codelist"
        | "order_contains_smgw_type2_product" => {
            Some(KonfigurationenAlias::Category("messprodukt_typ2_smgw"))
        }
        "valid_adhoc_steuerkanal_product" => Some(KonfigurationenAlias::Category(
            "config_product_adhoc_steuerkanal",
        )),
        "product_code_level_messlokation" => Some(KonfigurationenAlias::Category(
            "messprodukt_standard_messlokation",
        )),
        "product_code_level_netzlokation" => Some(KonfigurationenAlias::Category(
            "messprodukt_standard_netzlokation",
        )),
        "product_code_abrechnungsdaten_valid" => Some(KonfigurationenAlias::Category(
            "produkte_aenderung_abrechnungsdaten",
        )),
        "lin_product_code_is_lokationsaenderung_strom" => Some(KonfigurationenAlias::Category(
            "produkte_aenderung_lokation",
        )),
        _ => None,
    }
}

/// Prefixes for messprodukt_* category matching to build the union set.
const MESSPRODUKT_CATEGORY_PREFIXES: &[&str] = &[
    "messprodukt_standard_",
    "messprodukt_typ2_smgw",
    "messprodukt_esa",
];

pub struct KonfigurationenProvider {
    /// Map from category name to set of product codes.
    categories: HashMap<String, HashSet<String>>,
    /// All known product codes across all categories.
    all_codes: HashSet<String>,
    /// Union of all messprodukt_* categories.
    messprodukt_union: HashSet<String>,
    /// The product code to check against.
    product_code: Option<String>,
}

impl KonfigurationenProvider {
    /// Create from pre-parsed categories and a product code to check.
    pub fn new(categories: HashMap<String, Vec<String>>, product_code: Option<String>) -> Self {
        let mut all_codes = HashSet::new();
        let categories: HashMap<String, HashSet<String>> = categories
            .into_iter()
            .map(|(k, v)| {
                for code in &v {
                    all_codes.insert(code.clone());
                }
                (k, v.into_iter().collect())
            })
            .collect();

        // Build the messprodukt union from all matching categories
        let mut messprodukt_union = HashSet::new();
        for (name, codes) in &categories {
            let is_messprodukt = MESSPRODUKT_CATEGORY_PREFIXES
                .iter()
                .any(|prefix| name.starts_with(prefix) || name == prefix);
            if is_messprodukt {
                messprodukt_union.extend(codes.iter().cloned());
            }
        }

        Self {
            categories,
            all_codes,
            messprodukt_union,
            product_code,
        }
    }

    /// Load from the JSON format of `konfigurationen_code_lists.json`.
    pub fn from_json(json: &str, product_code: Option<String>) -> Result<Self, serde_json::Error> {
        #[derive(serde::Deserialize)]
        struct KonfigurationenFile {
            categories: HashMap<String, Vec<String>>,
            #[serde(default)]
            all_product_codes: Vec<String>,
        }
        let file: KonfigurationenFile = serde_json::from_str(json)?;
        let mut provider = Self::new(file.categories, product_code);
        // Merge explicit all_product_codes list if present
        for code in file.all_product_codes {
            provider.all_codes.insert(code);
        }
        Ok(provider)
    }

    /// Load from a JSON file path.
    pub fn from_json_file(
        path: &std::path::Path,
        product_code: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let json = std::fs::read_to_string(path)?;
        Ok(Self::from_json(&json, product_code)?)
    }
}

impl ExternalConditionProvider for KonfigurationenProvider {
    fn evaluate(&self, condition_name: &str) -> ConditionResult {
        let code = match &self.product_code {
            Some(c) => c,
            None => return ConditionResult::Unknown,
        };

        // Generic membership: is this product in the Konfigurationen at all?
        if condition_name == "code_list_membership_check" {
            return ConditionResult::from(self.all_codes.contains(code.as_str()));
        }

        // Check aliases before category lookup
        if let Some(alias) = resolve_konfigurationen_alias(condition_name) {
            return match alias {
                KonfigurationenAlias::AllCodes => {
                    ConditionResult::from(self.all_codes.contains(code.as_str()))
                }
                KonfigurationenAlias::Category(cat) => {
                    if let Some(set) = self.categories.get(cat) {
                        ConditionResult::from(set.contains(code.as_str()))
                    } else {
                        ConditionResult::Unknown
                    }
                }
                KonfigurationenAlias::MessproduktUnion => {
                    ConditionResult::from(self.messprodukt_union.contains(code.as_str()))
                }
            };
        }

        // Category-specific: is this product in the named category?
        if let Some(set) = self.categories.get(condition_name) {
            return ConditionResult::from(set.contains(code.as_str()));
        }

        ConditionResult::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- MapExternalProvider tests ----

    #[test]
    fn map_provider_returns_true_for_true_entry() {
        let mut conditions = HashMap::new();
        conditions.insert("DateKnown".to_string(), true);
        let provider = MapExternalProvider::new(conditions);

        assert_eq!(provider.evaluate("DateKnown"), ConditionResult::True);
    }

    #[test]
    fn map_provider_returns_false_for_false_entry() {
        let mut conditions = HashMap::new();
        conditions.insert("MessageSplitting".to_string(), false);
        let provider = MapExternalProvider::new(conditions);

        assert_eq!(
            provider.evaluate("MessageSplitting"),
            ConditionResult::False
        );
    }

    #[test]
    fn map_provider_returns_unknown_for_missing_key() {
        let mut conditions = HashMap::new();
        conditions.insert("DateKnown".to_string(), true);
        let provider = MapExternalProvider::new(conditions);

        assert_eq!(provider.evaluate("NonExistent"), ConditionResult::Unknown);
    }

    #[test]
    fn map_provider_empty_map_returns_unknown() {
        let provider = MapExternalProvider::new(HashMap::new());

        assert_eq!(provider.evaluate("Anything"), ConditionResult::Unknown);
    }

    // ---- CompositeExternalProvider tests ----

    #[test]
    fn composite_first_known_wins() {
        // Provider 1 knows "A" = true, but not "B"
        let mut p1_map = HashMap::new();
        p1_map.insert("A".to_string(), true);
        let p1 = MapExternalProvider::new(p1_map);

        // Provider 2 knows "B" = false, but not "A"
        let mut p2_map = HashMap::new();
        p2_map.insert("B".to_string(), false);
        let p2 = MapExternalProvider::new(p2_map);

        let composite = CompositeExternalProvider::new(vec![Box::new(p1), Box::new(p2)]);

        // "A" resolved by p1
        assert_eq!(composite.evaluate("A"), ConditionResult::True);
        // "B" not in p1 (Unknown), resolved by p2
        assert_eq!(composite.evaluate("B"), ConditionResult::False);
    }

    #[test]
    fn composite_all_unknown_returns_unknown() {
        // Two providers, neither knows "X"
        let p1 = MapExternalProvider::new(HashMap::new());
        let p2 = MapExternalProvider::new(HashMap::new());

        let composite = CompositeExternalProvider::new(vec![Box::new(p1), Box::new(p2)]);

        assert_eq!(composite.evaluate("X"), ConditionResult::Unknown);
    }

    #[test]
    fn composite_empty_returns_unknown() {
        let composite = CompositeExternalProvider::new(vec![]);

        assert_eq!(composite.evaluate("Anything"), ConditionResult::Unknown);
    }

    // ---- CodeListProvider tests ----

    #[test]
    fn test_code_list_provider_known_code() {
        let mut lists = HashMap::new();
        lists.insert(
            "7111".to_string(),
            vec!["Z91".to_string(), "Z90".to_string()],
        );
        let provider = CodeListProvider::new(lists);
        assert_eq!(provider.evaluate("code_in_7111:Z91"), ConditionResult::True);
        assert_eq!(
            provider.evaluate("code_in_7111:ZZZ"),
            ConditionResult::False
        );
        assert_eq!(
            provider.evaluate("code_in_9999:Z91"),
            ConditionResult::Unknown
        );
    }

    #[test]
    fn test_code_list_provider_loads_json() {
        let json = r#"{
            "7111": { "name": "Eigenschaft", "codes": [{"value": "Z91", "name": "MSB"}] },
            "3225": { "name": "Ort", "codes": [{"value": "Z16", "name": "MaLo"}] }
        }"#;
        let provider = CodeListProvider::from_json(json).unwrap();
        assert_eq!(provider.evaluate("code_in_7111:Z91"), ConditionResult::True);
        assert_eq!(provider.evaluate("code_in_3225:Z16"), ConditionResult::True);
        assert_eq!(
            provider.evaluate("code_in_3225:Z99"),
            ConditionResult::False
        );
    }

    #[test]
    fn test_code_list_provider_invalid_format() {
        let mut lists = HashMap::new();
        lists.insert("7111".to_string(), vec!["Z91".to_string()]);
        let provider = CodeListProvider::new(lists);
        assert_eq!(
            provider.evaluate("not_a_code_check"),
            ConditionResult::Unknown
        );
        assert_eq!(provider.evaluate("code_in_7111"), ConditionResult::Unknown);
        // no colon
    }

    // ---- SectorProvider tests ----

    #[test]
    fn test_sector_provider_strom() {
        let provider = SectorProvider::new(Sector::Strom);
        assert_eq!(
            provider.evaluate("recipient_is_strom"),
            ConditionResult::True
        );
        assert_eq!(
            provider.evaluate("recipient_is_gas"),
            ConditionResult::False
        );
        assert_eq!(provider.evaluate("mp_id_is_strom"), ConditionResult::True);
        assert_eq!(provider.evaluate("mp_id_is_gas"), ConditionResult::False);
        assert_eq!(
            provider.evaluate("market_location_is_electricity"),
            ConditionResult::True
        );
        assert_eq!(
            provider.evaluate("market_location_is_gas"),
            ConditionResult::False
        );
        assert_eq!(
            provider.evaluate("unrelated_condition"),
            ConditionResult::Unknown
        );
    }

    #[test]
    fn test_sector_provider_gas() {
        let provider = SectorProvider::new(Sector::Gas);
        assert_eq!(
            provider.evaluate("recipient_is_strom"),
            ConditionResult::False
        );
        assert_eq!(provider.evaluate("recipient_is_gas"), ConditionResult::True);
        assert_eq!(
            provider.evaluate("recipient_is_msb_gas"),
            ConditionResult::True
        );
    }

    #[test]
    fn test_sector_from_variant() {
        assert!(SectorProvider::from_variant("Strom").is_some());
        assert!(SectorProvider::from_variant("Gas").is_some());
        assert!(SectorProvider::from_variant("Water").is_none());
    }

    // ---- MarketRoleProvider tests ----

    #[test]
    fn test_market_role_provider() {
        let provider =
            MarketRoleProvider::new(vec![MarketRole::LF], vec![MarketRole::NB, MarketRole::MSB]);
        assert_eq!(provider.evaluate("sender_is_lf"), ConditionResult::True);
        assert_eq!(provider.evaluate("sender_is_nb"), ConditionResult::False);
        assert_eq!(provider.evaluate("recipient_is_nb"), ConditionResult::True);
        assert_eq!(provider.evaluate("recipient_is_msb"), ConditionResult::True);
        assert_eq!(provider.evaluate("recipient_is_lf"), ConditionResult::False);
        assert_eq!(provider.evaluate("unrelated"), ConditionResult::Unknown);
    }

    #[test]
    fn test_market_role_provider_negated() {
        let provider = MarketRoleProvider::new(vec![MarketRole::MSB], vec![MarketRole::NB]);
        assert_eq!(
            provider.evaluate("sender_role_is_not_msb"),
            ConditionResult::False
        );
        assert_eq!(
            provider.evaluate("sender_role_is_not_lf"),
            ConditionResult::True
        );
        assert_eq!(
            provider.evaluate("recipient_role_is_not_nb"),
            ConditionResult::False
        );
        assert_eq!(
            provider.evaluate("recipient_role_is_not_lf"),
            ConditionResult::True
        );
    }

    #[test]
    fn test_market_role_unknown_suffix() {
        let provider = MarketRoleProvider::new(vec![MarketRole::LF], vec![]);
        // "sender_is_xyz" — xyz not a known role, so Unknown
        assert_eq!(provider.evaluate("sender_is_xyz"), ConditionResult::Unknown);
    }

    #[test]
    fn test_market_role_compound() {
        let provider = MarketRoleProvider::new(vec![MarketRole::LF], vec![MarketRole::NB]);
        // recipient_is_lf_msb = recipient is LF OR MSB → NB is neither → False
        assert_eq!(
            provider.evaluate("recipient_is_lf_msb"),
            ConditionResult::False
        );
        // recipient_is_nb_lf = recipient is NB OR LF → NB matches → True
        assert_eq!(
            provider.evaluate("recipient_is_nb_lf"),
            ConditionResult::True
        );
        // recipient_is_lf_nb_msb = LF or NB or MSB → NB matches → True
        assert_eq!(
            provider.evaluate("recipient_is_lf_nb_msb"),
            ConditionResult::True
        );
        // mp_id_role_is_lf → sender has LF → True
        assert_eq!(provider.evaluate("mp_id_role_is_lf"), ConditionResult::True);
    }

    #[test]
    fn test_sector_provider_additional_names() {
        let provider = SectorProvider::new(Sector::Strom);
        assert_eq!(
            provider.evaluate("mp_id_is_strom_sector"),
            ConditionResult::True
        );
        assert_eq!(
            provider.evaluate("marktpartner_is_strom"),
            ConditionResult::True
        );
        assert_eq!(
            provider.evaluate("recipient_is_gas_sector"),
            ConditionResult::False
        );
    }

    #[test]
    fn test_composite_with_defaults() {
        let composite = CompositeExternalProvider::with_defaults(
            Some(Sector::Strom),
            Some((vec![MarketRole::LF], vec![MarketRole::NB])),
            None,
        );
        // Sector resolved
        assert_eq!(
            composite.evaluate("recipient_is_strom"),
            ConditionResult::True
        );
        // Market role resolved
        assert_eq!(composite.evaluate("sender_is_lf"), ConditionResult::True);
        // Unknown conditions still return Unknown
        assert_eq!(composite.evaluate("some_unknown"), ConditionResult::Unknown);
    }

    // ---- KonfigurationenProvider tests ----

    fn make_konfigurationen_provider(product_code: Option<&str>) -> KonfigurationenProvider {
        let mut categories = HashMap::new();
        categories.insert(
            "messprodukt_standard_marktlokation".to_string(),
            vec!["9991000000044".to_string(), "9991000000052".to_string()],
        );
        categories.insert(
            "messprodukt_standard_tranche".to_string(),
            vec!["9991000000143".to_string()],
        );
        categories.insert(
            "config_product_leistungskurve".to_string(),
            vec!["9991000000721".to_string()],
        );
        KonfigurationenProvider::new(categories, product_code.map(String::from))
    }

    #[test]
    fn test_konfigurationen_category_match() {
        let provider = make_konfigurationen_provider(Some("9991000000044"));
        assert_eq!(
            provider.evaluate("messprodukt_standard_marktlokation"),
            ConditionResult::True
        );
        assert_eq!(
            provider.evaluate("messprodukt_standard_tranche"),
            ConditionResult::False
        );
        assert_eq!(
            provider.evaluate("config_product_leistungskurve"),
            ConditionResult::False
        );
    }

    #[test]
    fn test_konfigurationen_generic_membership() {
        let provider = make_konfigurationen_provider(Some("9991000000044"));
        assert_eq!(
            provider.evaluate("code_list_membership_check"),
            ConditionResult::True
        );

        let provider2 = make_konfigurationen_provider(Some("0000000000000"));
        assert_eq!(
            provider2.evaluate("code_list_membership_check"),
            ConditionResult::False
        );
    }

    #[test]
    fn test_konfigurationen_no_product_code() {
        let provider = make_konfigurationen_provider(None);
        assert_eq!(
            provider.evaluate("messprodukt_standard_marktlokation"),
            ConditionResult::Unknown
        );
        assert_eq!(
            provider.evaluate("code_list_membership_check"),
            ConditionResult::Unknown
        );
    }

    #[test]
    fn test_konfigurationen_unknown_condition() {
        let provider = make_konfigurationen_provider(Some("9991000000044"));
        assert_eq!(
            provider.evaluate("completely_unrelated"),
            ConditionResult::Unknown
        );
    }

    #[test]
    fn test_konfigurationen_from_json() {
        let json = r#"{
            "source": "Test",
            "categories": {
                "messprodukt_standard_marktlokation": ["9991000000044", "9991000000052"],
                "config_product_leistungskurve": ["9991000000721"]
            },
            "all_product_codes": ["9991000000044", "9991000000052", "9991000000721"]
        }"#;
        let provider =
            KonfigurationenProvider::from_json(json, Some("9991000000721".to_string())).unwrap();
        assert_eq!(
            provider.evaluate("messprodukt_standard_marktlokation"),
            ConditionResult::False
        );
        assert_eq!(
            provider.evaluate("config_product_leistungskurve"),
            ConditionResult::True
        );
        assert_eq!(
            provider.evaluate("code_list_membership_check"),
            ConditionResult::True
        );
    }

    #[test]
    fn test_konfigurationen_specific_product() {
        // Test config_product_leistungskurve matches exactly one code
        let provider = make_konfigurationen_provider(Some("9991000000721"));
        assert_eq!(
            provider.evaluate("config_product_leistungskurve"),
            ConditionResult::True
        );
        assert_eq!(
            provider.evaluate("messprodukt_standard_marktlokation"),
            ConditionResult::False
        );
    }

    // ---- CountryPostalCodeProvider tests ----

    #[test]
    fn test_country_plz_provider_de() {
        let provider = CountryPostalCodeProvider::new(Some("DE".to_string()));
        assert_eq!(
            provider.evaluate("country_code_has_plz"),
            ConditionResult::True
        );
        assert_eq!(
            provider.evaluate("country_has_postal_code"),
            ConditionResult::True
        );
        assert_eq!(
            provider.evaluate("country_has_postal_code_requirement"),
            ConditionResult::True
        );
    }

    #[test]
    fn test_country_plz_provider_various_countries() {
        for code in &["AT", "CH", "FR", "NL", "GB", "NO", "IS", "LI"] {
            let provider = CountryPostalCodeProvider::new(Some(code.to_string()));
            assert_eq!(
                provider.evaluate("country_code_has_plz"),
                ConditionResult::True,
                "{code} should have PLZ"
            );
        }
    }

    #[test]
    fn test_country_plz_provider_unknown_country() {
        let provider = CountryPostalCodeProvider::new(Some("XX".to_string()));
        assert_eq!(
            provider.evaluate("country_code_has_plz"),
            ConditionResult::False
        );
    }

    #[test]
    fn test_country_plz_provider_no_country() {
        let provider = CountryPostalCodeProvider::new(None);
        assert_eq!(
            provider.evaluate("country_code_has_plz"),
            ConditionResult::Unknown
        );
    }

    #[test]
    fn test_country_plz_provider_unrelated_condition() {
        let provider = CountryPostalCodeProvider::new(Some("DE".to_string()));
        assert_eq!(
            provider.evaluate("unrelated_condition"),
            ConditionResult::Unknown
        );
    }

    // ---- SectorProvider: sender_is_gas_sector ----

    #[test]
    fn test_sector_provider_sender_is_gas_sector() {
        let provider = SectorProvider::new(Sector::Gas);
        assert_eq!(
            provider.evaluate("sender_is_gas_sector"),
            ConditionResult::True
        );
        let provider_strom = SectorProvider::new(Sector::Strom);
        assert_eq!(
            provider_strom.evaluate("sender_is_gas_sector"),
            ConditionResult::False
        );
    }

    // ---- MarketRole: MGV and KN ----

    #[test]
    fn test_market_role_mgv_kn() {
        let provider = MarketRoleProvider::new(vec![MarketRole::MGV], vec![MarketRole::KN]);
        assert_eq!(provider.evaluate("sender_is_mgv"), ConditionResult::True);
        assert_eq!(provider.evaluate("recipient_is_kn"), ConditionResult::True);
        assert_eq!(provider.evaluate("sender_is_kn"), ConditionResult::False);
        assert_eq!(
            provider.evaluate("recipient_is_mgv"),
            ConditionResult::False
        );
    }

    // ---- MarketRole: _or_ compound ----

    #[test]
    fn test_market_role_or_compound() {
        let provider = MarketRoleProvider::new(vec![MarketRole::LF], vec![MarketRole::NB]);
        // recipient_is_lf_or_nb = recipient is LF OR NB → NB matches → True
        assert_eq!(
            provider.evaluate("recipient_is_lf_or_nb"),
            ConditionResult::True
        );
        // recipient_is_msb_or_mdl = recipient is MSB OR MDL → NB is neither → False
        assert_eq!(
            provider.evaluate("recipient_is_msb_or_mdl"),
            ConditionResult::False
        );
    }

    // ---- MarketRole: recipient_not_ prefix ----

    #[test]
    fn test_market_role_recipient_not() {
        let provider = MarketRoleProvider::new(vec![MarketRole::LF], vec![MarketRole::NB]);
        // recipient_not_mgv_or_kn = recipient is NOT (MGV or KN) → NB is neither → True
        assert_eq!(
            provider.evaluate("recipient_not_mgv_or_kn"),
            ConditionResult::True
        );
        // recipient_not_nb = recipient is NOT NB → False (NB is the recipient)
        assert_eq!(
            provider.evaluate("recipient_not_nb"),
            ConditionResult::False
        );
        // recipient_not_lf = recipient is NOT LF → True (NB is the recipient)
        assert_eq!(provider.evaluate("recipient_not_lf"), ConditionResult::True);
        // sender_not_lf = sender is NOT LF → False (LF is the sender)
        assert_eq!(provider.evaluate("sender_not_lf"), ConditionResult::False);
        // sender_not_nb_or_msb = sender is NOT (NB or MSB) → LF is neither → True
        assert_eq!(
            provider.evaluate("sender_not_nb_or_msb"),
            ConditionResult::True
        );
    }

    // ---- KonfigurationenProvider alias tests ----

    fn make_konfigurationen_provider_with_messprodukt(
        product_code: Option<&str>,
    ) -> KonfigurationenProvider {
        let mut categories = HashMap::new();
        categories.insert(
            "messprodukt_standard_marktlokation".to_string(),
            vec!["9991000000044".to_string(), "9991000000052".to_string()],
        );
        categories.insert(
            "messprodukt_standard_tranche".to_string(),
            vec!["9991000000143".to_string()],
        );
        categories.insert(
            "messprodukt_standard_messlokation".to_string(),
            vec!["9991000000200".to_string()],
        );
        categories.insert(
            "messprodukt_standard_netzlokation".to_string(),
            vec!["9991000000300".to_string()],
        );
        categories.insert(
            "messprodukt_typ2_smgw".to_string(),
            vec!["9991000000500".to_string()],
        );
        categories.insert(
            "messprodukt_esa".to_string(),
            vec!["9991000000600".to_string()],
        );
        categories.insert(
            "config_product_leistungskurve".to_string(),
            vec!["9991000000721".to_string()],
        );
        categories.insert(
            "config_product_adhoc_steuerkanal".to_string(),
            vec!["9991000000800".to_string()],
        );
        categories.insert(
            "produkte_aenderung_abrechnungsdaten".to_string(),
            vec!["9991000000900".to_string()],
        );
        categories.insert(
            "produkte_aenderung_lokation".to_string(),
            vec!["9991000001000".to_string()],
        );
        KonfigurationenProvider::new(categories, product_code.map(String::from))
    }

    #[test]
    fn test_konfigurationen_alias_all_codes() {
        let provider = make_konfigurationen_provider_with_messprodukt(Some("9991000000044"));
        // is_konfigurationsprodukt_code → all_codes check
        assert_eq!(
            provider.evaluate("is_konfigurationsprodukt_code"),
            ConditionResult::True
        );
        // lin_prefix_is_valid_product_code → all_codes check
        assert_eq!(
            provider.evaluate("lin_prefix_is_valid_product_code"),
            ConditionResult::True
        );

        let provider2 = make_konfigurationen_provider_with_messprodukt(Some("0000000000000"));
        assert_eq!(
            provider2.evaluate("is_konfigurationsprodukt_code"),
            ConditionResult::False
        );
    }

    #[test]
    fn test_konfigurationen_alias_messprodukt_union() {
        // Code in messprodukt_standard_marktlokation (part of union)
        let provider = make_konfigurationen_provider_with_messprodukt(Some("9991000000044"));
        assert_eq!(
            provider.evaluate("is_messprodukt_code"),
            ConditionResult::True
        );

        // Code in messprodukt_typ2_smgw (part of union)
        let provider2 = make_konfigurationen_provider_with_messprodukt(Some("9991000000500"));
        assert_eq!(
            provider2.evaluate("is_messprodukt_code"),
            ConditionResult::True
        );

        // Code in messprodukt_esa (part of union)
        let provider3 = make_konfigurationen_provider_with_messprodukt(Some("9991000000600"));
        assert_eq!(
            provider3.evaluate("is_messprodukt_code"),
            ConditionResult::True
        );

        // Code in config_product_leistungskurve (NOT in union)
        let provider4 = make_konfigurationen_provider_with_messprodukt(Some("9991000000721"));
        assert_eq!(
            provider4.evaluate("is_messprodukt_code"),
            ConditionResult::False
        );
    }

    #[test]
    fn test_konfigurationen_alias_single_category() {
        let provider = make_konfigurationen_provider_with_messprodukt(Some("9991000000500"));
        // messprodukts_typ2_smgw → messprodukt_typ2_smgw
        assert_eq!(
            provider.evaluate("messprodukts_typ2_smgw"),
            ConditionResult::True
        );
        // product_in_typ2_smgw_codelist → messprodukt_typ2_smgw
        assert_eq!(
            provider.evaluate("product_in_typ2_smgw_codelist"),
            ConditionResult::True
        );
        // order_contains_smgw_type2_product → messprodukt_typ2_smgw
        assert_eq!(
            provider.evaluate("order_contains_smgw_type2_product"),
            ConditionResult::True
        );

        let provider2 = make_konfigurationen_provider_with_messprodukt(Some("9991000000800"));
        // valid_adhoc_steuerkanal_product → config_product_adhoc_steuerkanal
        assert_eq!(
            provider2.evaluate("valid_adhoc_steuerkanal_product"),
            ConditionResult::True
        );

        let provider3 = make_konfigurationen_provider_with_messprodukt(Some("9991000000200"));
        // product_code_level_messlokation → messprodukt_standard_messlokation
        assert_eq!(
            provider3.evaluate("product_code_level_messlokation"),
            ConditionResult::True
        );

        let provider4 = make_konfigurationen_provider_with_messprodukt(Some("9991000000300"));
        // product_code_level_netzlokation → messprodukt_standard_netzlokation
        assert_eq!(
            provider4.evaluate("product_code_level_netzlokation"),
            ConditionResult::True
        );

        let provider5 = make_konfigurationen_provider_with_messprodukt(Some("9991000000900"));
        // product_code_abrechnungsdaten_valid → produkte_aenderung_abrechnungsdaten
        assert_eq!(
            provider5.evaluate("product_code_abrechnungsdaten_valid"),
            ConditionResult::True
        );

        let provider6 = make_konfigurationen_provider_with_messprodukt(Some("9991000001000"));
        // lin_product_code_is_lokationsaenderung_strom → produkte_aenderung_lokation
        assert_eq!(
            provider6.evaluate("lin_product_code_is_lokationsaenderung_strom"),
            ConditionResult::True
        );
    }

    // ---- CompositeProviderBuilder: country_code ----

    #[test]
    fn test_builder_with_country_code() {
        let composite = CompositeExternalProvider::builder()
            .country_code(Some("DE".to_string()))
            .build();
        assert_eq!(
            composite.evaluate("country_code_has_plz"),
            ConditionResult::True
        );
    }
}
