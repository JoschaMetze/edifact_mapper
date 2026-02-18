/// Flow control signal returned by handler methods.
///
/// Handlers return this to tell the parser whether to continue
/// processing or stop early.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Control {
    /// Continue processing the next segment.
    #[default]
    Continue,
    /// Stop processing immediately.
    Stop,
}

impl Control {
    /// Returns `true` if this is `Control::Continue`.
    pub fn should_continue(&self) -> bool {
        matches!(self, Self::Continue)
    }

    /// Returns `true` if this is `Control::Stop`.
    pub fn should_stop(&self) -> bool {
        matches!(self, Self::Stop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_continue() {
        let c = Control::Continue;
        assert!(c.should_continue());
        assert!(!c.should_stop());
    }

    #[test]
    fn test_control_stop() {
        let c = Control::Stop;
        assert!(c.should_stop());
        assert!(!c.should_continue());
    }

    #[test]
    fn test_control_default_is_continue() {
        assert_eq!(Control::default(), Control::Continue);
    }

    #[test]
    fn test_control_equality() {
        assert_eq!(Control::Continue, Control::Continue);
        assert_eq!(Control::Stop, Control::Stop);
        assert_ne!(Control::Continue, Control::Stop);
    }
}
