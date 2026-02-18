//! Transaction context shared across mappers during parsing.

use std::any::Any;
use std::collections::HashMap;

/// Holds cross-cutting transaction-level state during EDIFACT parsing.
///
/// Mappers use the context to share data across segment boundaries.
/// For example, the message reference from UNH is stored here so that
/// all mappers can access it.
///
/// Mirrors the C# `ITransactionContext` / `TransactionContext`.
#[derive(Debug)]
pub struct TransactionContext {
    /// The format version being processed (e.g., "FV2504").
    pub format_version: String,

    /// The message reference number from UNH segment.
    pub message_reference: Option<String>,

    /// The Pruefidentifikator for this transaction.
    pub pruefidentifikator: Option<String>,

    /// The sender MP-ID from the message header.
    pub sender_mp_id: Option<String>,

    /// The recipient MP-ID from the message header.
    pub recipient_mp_id: Option<String>,

    /// The current transaction ID from IDE segment.
    pub transaction_id: Option<String>,

    /// The current Zeitscheibe reference being processed.
    pub current_zeitscheibe_ref: Option<String>,

    /// Registered objects keyed by type name and ID.
    objects: HashMap<String, Box<dyn Any + Send>>,
}

impl TransactionContext {
    /// Creates a new context for the given format version.
    pub fn new(format_version: impl Into<String>) -> Self {
        Self {
            format_version: format_version.into(),
            message_reference: None,
            pruefidentifikator: None,
            sender_mp_id: None,
            recipient_mp_id: None,
            transaction_id: None,
            current_zeitscheibe_ref: None,
            objects: HashMap::new(),
        }
    }

    /// Sets the message reference from UNH.
    pub fn set_message_reference(&mut self, reference: impl Into<String>) {
        self.message_reference = Some(reference.into());
    }

    /// Sets the Pruefidentifikator.
    pub fn set_pruefidentifikator(&mut self, pi: impl Into<String>) {
        self.pruefidentifikator = Some(pi.into());
    }

    /// Sets the sender MP-ID.
    pub fn set_sender_mp_id(&mut self, id: impl Into<String>) {
        self.sender_mp_id = Some(id.into());
    }

    /// Sets the recipient MP-ID.
    pub fn set_recipient_mp_id(&mut self, id: impl Into<String>) {
        self.recipient_mp_id = Some(id.into());
    }

    /// Registers an object for later retrieval.
    pub fn register_object<T: Any + Send>(&mut self, key: impl Into<String>, obj: T) {
        self.objects.insert(key.into(), Box::new(obj));
    }

    /// Gets a registered object by key.
    pub fn get_object<T: Any + Send>(&self, key: &str) -> Option<&T> {
        self.objects.get(key).and_then(|v| v.downcast_ref::<T>())
    }

    /// Resets the context for a new message, clearing all transient state.
    pub fn reset(&mut self) {
        self.message_reference = None;
        self.pruefidentifikator = None;
        self.transaction_id = None;
        self.current_zeitscheibe_ref = None;
        self.objects.clear();
        // Note: format_version, sender_mp_id, recipient_mp_id persist across messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_context_new() {
        let ctx = TransactionContext::new("FV2504");
        assert_eq!(ctx.format_version, "FV2504");
        assert!(ctx.message_reference.is_none());
        assert!(ctx.pruefidentifikator.is_none());
        assert!(ctx.sender_mp_id.is_none());
        assert!(ctx.recipient_mp_id.is_none());
    }

    #[test]
    fn test_transaction_context_set_fields() {
        let mut ctx = TransactionContext::new("FV2510");
        ctx.set_message_reference("MSG001");
        ctx.set_pruefidentifikator("11042");
        ctx.set_sender_mp_id("9900123000002");
        ctx.set_recipient_mp_id("9900456000001");

        assert_eq!(ctx.message_reference, Some("MSG001".to_string()));
        assert_eq!(ctx.pruefidentifikator, Some("11042".to_string()));
        assert_eq!(ctx.sender_mp_id, Some("9900123000002".to_string()));
        assert_eq!(ctx.recipient_mp_id, Some("9900456000001".to_string()));
    }

    #[test]
    fn test_transaction_context_register_and_get_object() {
        let mut ctx = TransactionContext::new("FV2504");
        ctx.register_object("test_string", "hello".to_string());

        let retrieved = ctx.get_object::<String>("test_string");
        assert_eq!(retrieved, Some(&"hello".to_string()));

        // Wrong type returns None
        let wrong_type = ctx.get_object::<u32>("test_string");
        assert!(wrong_type.is_none());

        // Missing key returns None
        let missing = ctx.get_object::<String>("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_transaction_context_reset() {
        let mut ctx = TransactionContext::new("FV2504");
        ctx.set_message_reference("MSG001");
        ctx.set_pruefidentifikator("11042");
        ctx.set_sender_mp_id("9900123000002");
        ctx.register_object("key", 42u32);

        ctx.reset();

        // Transient state is cleared
        assert!(ctx.message_reference.is_none());
        assert!(ctx.pruefidentifikator.is_none());
        assert!(ctx.get_object::<u32>("key").is_none());

        // Persistent state is preserved
        assert_eq!(ctx.format_version, "FV2504");
        assert_eq!(ctx.sender_mp_id, Some("9900123000002".to_string()));
    }

    #[test]
    fn test_transaction_context_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<TransactionContext>();
    }
}
