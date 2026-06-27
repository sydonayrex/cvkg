    use super::*;

    #[test]
    fn kvasir_id_new_is_non_zero() {
        // Contract: KvasirId::new() must never return the null sentinel.
        let id = KvasirId::new();
        assert!(!id.is_null(), "KvasirId::new() returned null sentinel");
    }

    #[test]
    fn kvasir_id_new_is_unique() {
        // Each call must produce a distinct ID.
        let a = KvasirId::new();
        let b = KvasirId::new();
        let c = KvasirId::new();
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
    }

    #[test]
    fn kvasir_id_null_sentinel() {
        assert!(KvasirId::NULL.is_null());
        assert!(!KvasirId::new().is_null());
    }

    #[test]
    fn kvasir_id_serde_roundtrip() {
        let id = KvasirId(42);
        let json = serde_json::to_string(&id).unwrap();
        let decoded: KvasirId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, decoded);
    }

    #[test]
    fn dirty_flags_clean_is_not_dirty() {
        assert!(!DirtyFlags::CLEAN.is_dirty());
    }

    #[test]
    fn dirty_flags_all_implies_all_layers() {
        let f = DirtyFlags::ALL;
        assert!(f.needs_state());
        assert!(f.needs_layout());
        assert!(f.needs_paint());
        assert!(f.needs_composite());
    }

    #[test]
    fn dirty_flags_composite_only() {
        let f = DirtyFlags::COMPOSITE;
        assert!(!f.needs_state());
        assert!(!f.needs_layout());
        assert!(!f.needs_paint());
        assert!(f.needs_composite());
    }

    #[test]
    fn dirty_flags_merge() {
        let a = DirtyFlags::COMPOSITE;
        let b = DirtyFlags::PAINT;
        let merged = a.merge(b);
        assert!(merged.needs_composite());
        assert!(merged.needs_paint());
        assert!(!merged.needs_layout());
    }

    #[test]
    fn dirty_flags_bitor() {
        let combined = DirtyFlags::PAINT | DirtyFlags::COMPOSITE;
        assert!(combined.needs_paint());
        assert!(combined.needs_composite());
    }

    #[test]
    fn dirty_flags_clear() {
        let dirty = DirtyFlags::ALL;
        let clean = dirty.clear();
        assert!(!clean.is_dirty());
    }

    #[test]
    fn dirty_flags_serde_roundtrip() {
        let f = DirtyFlags::LAYOUT;
        let json = serde_json::to_string(&f).unwrap();
        let decoded: DirtyFlags = serde_json::from_str(&json).unwrap();
        assert_eq!(f, decoded);
    }

    #[test]
    fn invalidation_record_full() {
        let id = KvasirId::new();
        let rec = InvalidationRecord::full(id);
        assert_eq!(rec.id, id);
        assert!(rec.flags.needs_state());
        assert!(rec.flags.needs_layout());
    }