use thiserror::Error;

#[derive(Debug, Error)]
pub enum LayoutError {
    #[error("Layout constraint conflict in node {node_id}: {reason}. Check flex properties for circular or over-constrained layouts.")]
    ConstraintConflict { node_id: u64, reason: String },

    #[error("Layout engine capacity exceeded: {0}. Reduce UI complexity or increase limits.")]
    CapacityExceeded(String),

    #[error("NaN or Inf value propagated through layout calculations at node {node_id}. Check intrinsic_size return values for invalid floats.")]
    InvalidFloat { node_id: u64 },

    #[error("Layout computation failed: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constraint_conflict_includes_node_id_and_reason() {
        let err = LayoutError::ConstraintConflict {
            node_id: 42,
            reason: "over-constrained flex".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("42"), "should contain node ID");
        assert!(msg.contains("over-constrained flex"), "should contain reason");
        assert!(msg.contains("flex properties"), "should contain suggestion");
    }

    #[test]
    fn capacity_exceeded_includes_detail() {
        let err = LayoutError::CapacityExceeded("node limit 1024 reached".into());
        let msg = err.to_string();
        assert!(msg.contains("1024"), "should contain detail");
        assert!(msg.contains("Reduce UI complexity"), "should contain suggestion");
    }

    #[test]
    fn invalid_float_includes_node_id() {
        let err = LayoutError::InvalidFloat { node_id: 7 };
        let msg = err.to_string();
        assert!(msg.contains("7"), "should contain node ID");
        assert!(msg.contains("NaN"), "should mention NaN");
    }

    #[test]
    fn internal_includes_message() {
        let err = LayoutError::Internal("taffy solver diverged".into());
        let msg = err.to_string();
        assert!(msg.contains("taffy solver diverged"), "should contain internal message");
    }

    #[test]
    fn error_trait_satisfied() {
        let _boxed: Box<dyn std::error::Error> = Box::new(LayoutError::Internal("test".into()));
    }
}
