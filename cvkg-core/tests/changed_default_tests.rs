use cvkg_core::{LayoutView, Rect, Size, SizeProposal};

/// Static layout view that never changes — used to verify that
/// `changed()` returns false by default on the LayoutView trait.
struct StaticLabel {
    width: f32,
    height: f32,
}

impl LayoutView for StaticLabel {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut cvkg_core::LayoutCache,
    ) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn place_subviews(&self, _rect: Rect) {}
}

#[test]
fn static_view_changed_returns_false() {
    let label = StaticLabel {
        width: 100.0,
        height: 20.0,
    };

    // A static view should never report as changed.
    assert!(
        !label.changed(),
        "Static view should return false from changed()"
    );
}

#[test]
fn static_view_needs_update_returns_false() {
    let label = StaticLabel {
        width: 100.0,
        height: 20.0,
    };

    // A static view should not request per-frame updates.
    assert!(
        !label.needs_update(),
        "Static view should return false from needs_update()"
    );
}

#[test]
fn static_view_changed_consistent_across_calls() {
    let label = StaticLabel {
        width: 50.0,
        height: 10.0,
    };

    // Render twice — both should return false.
    let first = label.changed();
    let second = label.changed();
    assert!(!first, "First render changed() should be false");
    assert!(!second, "Second render changed() should be false");
}
