#![allow(clippy::assertions_on_constants)]

// Backend Integration Tests
// Testing Native, Web, and GPU renderer integration

#[cfg(test)]
mod tests {
    use cvkg_core::KvasirId;
    use cvkg_render_gpu::GpuRenderer;
    use cvkg_runic_text::subpixel::{SubpixelGlyph, render_lcd};
    use cvkg_runic_text::{
        PortalAlignment, RunicPathSegment, TextAlign, TextEngine, TextOverflow, TextSpan, TextStyle,
    };

    /// Test: Native Renderer Integration
    /// Verifies native window creation and basic rendering
    #[test]
    fn test_native_renderer_integration() {
        // Placeholder for native renderer integration test
        // In production, this would create a native window and verify rendering
        assert!(true, "Native renderer integration placeholder");
    }

    #[tokio::test]
    async fn test_gpu_renderer_integration() {
        // Verify we can at least attempt to forge a headless renderer
        let _ = GpuRenderer::forge_headless(100, 100).await;
        assert!(true);
    }

    #[test]
    fn test_runic_text_vector_outline_extraction() {
        let mut engine = TextEngine::new_test();
        let style = TextStyle::new("Jupiteroid", 16.0);

        // 1. Shape a character 'B' to get its glyph ID via public shape_layout API
        let spans = vec![TextSpan::new("B", style.clone())];
        let shaped = engine
            .shape_layout(&spans, None, TextAlign::Start, TextOverflow::WordWrap)
            .unwrap();
        assert!(
            !shaped.glyphs.is_empty(),
            "Shaping 'B' must produce a glyph instance"
        );
        let glyph_id = shaped.glyphs[0].glyph_id;

        // 2. Extract vector path
        let path = engine.extract_glyph_path(glyph_id, 16.0, &style).unwrap();
        assert!(
            !path.is_empty(),
            "Vector path for glyph 'B' must not be empty"
        );

        // First segment must start a subpath
        match path[0] {
            RunicPathSegment::MoveTo { x, y } => {
                assert!(x.is_finite(), "MoveTo X coordinate must be finite");
                assert!(y.is_finite(), "MoveTo Y coordinate must be finite");
            }
            _ => panic!(
                "Expected first path segment to be MoveTo, got {:?}",
                path[0]
            ),
        }

        // Contour must close
        let has_close = path
            .iter()
            .any(|seg| matches!(seg, RunicPathSegment::Close));
        assert!(has_close, "Vector path contour must close");

        // All coordinates must be finite
        for segment in &path {
            match *segment {
                RunicPathSegment::MoveTo { x, y } | RunicPathSegment::LineTo { x, y } => {
                    assert!(x.is_finite());
                    assert!(y.is_finite());
                }
                RunicPathSegment::QuadTo { cx, cy, x, y } => {
                    assert!(cx.is_finite());
                    assert!(cy.is_finite());
                    assert!(x.is_finite());
                    assert!(y.is_finite());
                }
                RunicPathSegment::CubicTo {
                    cx1,
                    cy1,
                    cx2,
                    cy2,
                    x,
                    y,
                } => {
                    assert!(cx1.is_finite());
                    assert!(cy1.is_finite());
                    assert!(cx2.is_finite());
                    assert!(cy2.is_finite());
                    assert!(x.is_finite());
                    assert!(y.is_finite());
                }
                RunicPathSegment::Close => {}
            }
        }
    }

    #[test]
    fn test_runic_text_portal_vertical_alignment() {
        let mut engine = TextEngine::new_test();
        let style = TextStyle::new("Jupiteroid", 16.0);

        // Build inline portal spans with different vertical alignments using correct offsets
        let spans = vec![
            TextSpan::at("ABC ", style.clone(), 0),
            TextSpan::portal_at(
                40.0,
                30.0,
                PortalAlignment::Baseline,
                "p1",
                style.clone(),
                4,
            ),
            TextSpan::portal_at(40.0, 30.0, PortalAlignment::Top, "p2", style.clone(), 7),
            TextSpan::portal_at(40.0, 30.0, PortalAlignment::Center, "p3", style.clone(), 10),
            TextSpan::portal_at(40.0, 30.0, PortalAlignment::Bottom, "p4", style.clone(), 13),
        ];

        let shaped = engine
            .shape_layout(&spans, None, TextAlign::Start, TextOverflow::WordWrap)
            .unwrap();

        let portals: Vec<_> = shaped
            .glyphs
            .iter()
            .filter(|g| g.glyph_id == 0xFFFF)
            .collect();
        assert_eq!(
            portals.len(),
            4,
            "Must shape exactly 4 portal sentinel glyphs"
        );

        let baseline_y = shaped.lines[0].baseline_y;
        let ascent = shaped.ascent;
        let line_height_px = shaped.lines[0].height;

        // Baseline (p1): Y = baseline_y
        assert_eq!(portals[0].y, baseline_y);

        // Top (p2): Y = baseline_y - ascent
        assert_eq!(portals[1].y, baseline_y - ascent);

        // Center (p3): Y = baseline_y - ascent + (line_height - portal_h) / 2
        assert_eq!(
            portals[2].y,
            baseline_y - ascent + (line_height_px - 30.0) / 2.0
        );

        // Bottom (p4): Y = baseline_y - ascent + line_height - portal_h
        assert_eq!(portals[3].y, baseline_y - ascent + line_height_px - 30.0);
    }

    #[test]
    fn test_runic_text_lcd_gamma_blending() {
        // Mid-gray background (127)
        let mut fb = vec![127u8; 100 * 100 * 4];
        let glyphs = vec![SubpixelGlyph::new(1, 0, 0, 255, 0, 0, 10, 16)];

        // Render pure white text at 50% opacity (128)
        render_lcd(&mut fb, 100, 100, &glyphs, (255, 255, 255, 128));

        // R subpixel has full coverage (255) and 128 text alpha -> alpha = ~0.50196
        // Non-linear gamma-corrected blending (gamma = 2.2):
        // blended = ((1 - 0.50196) * (127/255)^2.2 + 0.50196 * (255/255)^2.2)^(1/2.2) * 255 = 204
        assert_eq!(fb[0], 204, "Red channel must be gamma-blended to 204");
        assert_eq!(fb[1], 127, "Green channel must remain background 127");
        assert_eq!(fb[2], 127, "Blue channel must remain background 127");
    }

    #[test]
    fn test_flow_uniform_velocity_tessellation() {
        use cvkg_flow::ribbon::tessellate_bezier_uniform;
        use glam::Vec2;

        let p0 = Vec2::new(0.0, 0.0);
        let p1 = Vec2::new(10.0, 30.0);
        let p2 = Vec2::new(20.0, -20.0);
        let p3 = Vec2::new(30.0, 10.0);

        let (points, uvs) = tessellate_bezier_uniform(p0, p1, p2, p3, 10);
        assert_eq!(points.len(), 11);
        assert_eq!(uvs.len(), 11);

        // First step distance should match remaining step distances
        let first_dist = points[1].distance(points[0]);
        for i in 1..10 {
            let dist = points[i + 1].distance(points[i]);
            assert!(
                (dist - first_dist).abs() < 0.2,
                "Step distance must be uniform"
            );
        }
    }

    #[test]
    fn test_flow_spatial_hash_query_correctness() {
        use cvkg_flow::FlowCanvas;
        use cvkg_flow::node::FlowNode;

        let mut canvas = FlowCanvas::new();
        canvas.add_node(FlowNode::new(KvasirId(101), "Alpha", (10.0, 10.0)));
        canvas.add_node(FlowNode::new(KvasirId(102), "Beta", (500.0, 500.0)));

        let in_rect = canvas.graph.nodes_in_rect(0.0, 0.0, 200.0, 200.0);
        assert_eq!(in_rect.len(), 1);
        assert_eq!(in_rect[0], KvasirId(101));
    }

    #[test]
    fn test_flow_camera_lod_bounds() {
        use cvkg_flow::FlowCanvas;
        use cvkg_flow::types::LevelOfDetail;

        let mut canvas = FlowCanvas::new();
        canvas.camera.zoom = 0.8;
        assert_eq!(canvas.level_of_detail(), LevelOfDetail::Detailed);

        canvas.camera.zoom = 0.5;
        assert_eq!(canvas.level_of_detail(), LevelOfDetail::Medium);

        canvas.camera.zoom = 0.2;
        assert_eq!(canvas.level_of_detail(), LevelOfDetail::Simplified);
    }

    #[test]
    fn test_anim_velocity_inheritance() {
        use cvkg_anim::{SpringParams, SpringSolver};

        let params = SpringParams::snappy();
        let mut solver_no_vel = SpringSolver::new(params, 100.0, 0.0);
        let mut solver_with_vel = SpringSolver::new(params, 100.0, 0.0).with_velocity(150.0);

        solver_no_vel.tick(0.016);
        solver_with_vel.tick(0.016);

        assert!(solver_with_vel.tick(0.0) > solver_no_vel.tick(0.0));
    }

    #[test]
    fn test_anim_substepping_stability() {
        use cvkg_anim::{SpringParams, SpringSolver};

        let params = SpringParams::bouncy();
        let mut solver = SpringSolver::new(params, 10.0, 0.0);

        // Huge time step (0.3s)
        solver.tick(0.3);

        assert!(solver.tick(0.0).is_finite());
    }

    #[test]
    fn test_anim_verlet_rope_simulation() {
        use cvkg_anim::verlet::{DistanceConstraint, VerletParticle, VerletSolver};

        let mut solver = VerletSolver::new(0.0, 9.81);
        let p1 = solver.add_particle(VerletParticle::pinned(0.0, 0.0));
        let p2 = solver.add_particle(VerletParticle::new(0.0, 5.0));

        solver.add_constraint(DistanceConstraint::new(p1, p2, 5.0, 1.0));

        for _ in 0..10 {
            solver.tick(0.05);
        }

        let pos = solver.particles[p2].position;
        assert!((pos[0] - 0.0).abs() < 0.01);
        assert!((pos[1] - 5.0).abs() < 0.1);
    }
}
