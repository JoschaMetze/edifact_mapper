use std::collections::HashMap;

use crate::uri::Bo4eUri;

/// Registry for managing links between BO4E objects within a transaction.
#[derive(Debug, Clone, Default)]
pub struct LinkRegistry {
    links: HashMap<Bo4eUri, Vec<Bo4eUri>>,
}

impl LinkRegistry {
    /// Creates a new empty link registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a link from source to target.
    pub fn add_link(&mut self, source: Bo4eUri, target: Bo4eUri) {
        self.links.entry(source).or_default().push(target);
    }

    /// Gets all links from a specific source.
    pub fn get_links_from(&self, source: &Bo4eUri) -> &[Bo4eUri] {
        self.links.get(source).map_or(&[], |v| v.as_slice())
    }

    /// Returns all links as a map.
    pub fn get_all_links(&self) -> &HashMap<Bo4eUri, Vec<Bo4eUri>> {
        &self.links
    }

    /// Clears all registered links.
    pub fn clear(&mut self) {
        self.links.clear();
    }

    /// Returns the number of source entries.
    pub fn len(&self) -> usize {
        self.links.len()
    }

    /// Returns true if no links are registered.
    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_registry_add_and_get() {
        let mut reg = LinkRegistry::new();
        let ml = Bo4eUri::new("Marktlokation", "ML001");
        let melo = Bo4eUri::new("Messlokation", "MELO001");
        let nelo = Bo4eUri::new("Netzlokation", "NELO001");

        reg.add_link(ml.clone(), melo.clone());
        reg.add_link(ml.clone(), nelo.clone());

        let links = reg.get_links_from(&ml);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0], melo);
        assert_eq!(links[1], nelo);
    }

    #[test]
    fn test_link_registry_empty() {
        let reg = LinkRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);

        let ml = Bo4eUri::new("Marktlokation", "ML001");
        assert!(reg.get_links_from(&ml).is_empty());
    }

    #[test]
    fn test_link_registry_clear() {
        let mut reg = LinkRegistry::new();
        reg.add_link(Bo4eUri::new("A", "1"), Bo4eUri::new("B", "2"));
        assert!(!reg.is_empty());

        reg.clear();
        assert!(reg.is_empty());
    }
}
