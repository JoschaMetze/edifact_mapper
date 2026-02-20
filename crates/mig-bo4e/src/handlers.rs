//! Complex mapping handler registry.
//!
//! Handlers are Rust functions registered by name for mappings
//! that cannot be expressed declaratively in TOML.

use std::collections::HashMap;

use mig_assembly::assembler::AssembledGroupInstance;

use crate::error::MappingError;

type HandlerFn =
    Box<dyn Fn(&AssembledGroupInstance) -> Result<serde_json::Value, MappingError> + Send + Sync>;

/// Registry of named complex mapping handlers.
pub struct HandlerRegistry {
    handlers: HashMap<String, HandlerFn>,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a handler function by name.
    pub fn register<F>(&mut self, name: &str, handler: F)
    where
        F: Fn(&AssembledGroupInstance) -> Result<serde_json::Value, MappingError>
            + Send
            + Sync
            + 'static,
    {
        self.handlers.insert(name.to_string(), Box::new(handler));
    }

    /// Check if a handler exists.
    pub fn has_handler(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }

    /// Invoke a handler by name.
    pub fn invoke(
        &self,
        name: &str,
        instance: &AssembledGroupInstance,
    ) -> Result<serde_json::Value, MappingError> {
        let handler = self
            .handlers
            .get(name)
            .ok_or_else(|| MappingError::UnknownHandler {
                name: name.to_string(),
                file: String::new(),
            })?;
        handler(instance)
    }

    /// Get the number of registered handlers.
    pub fn len(&self) -> usize {
        self.handlers.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
