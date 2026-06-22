use cvkg_components::*;
use cvkg_core::View;

/// A tactical showcase of the Cyberpunk Viking UI components.
/// This example demonstrates the high-fidelity Norse HUD patterns.
fn tactical_hud_showcase() -> impl View {
    VStack::new(24.0)
        .child(
            // 1. High-Priority Header
            RunesCard::new()
                .header(
                    Text::new("Tactical Command Center")
                        .font_size(24.0)
                        .color([0.0, 1.0, 1.0, 1.0])
                        .erase(),
                )
                .content(
                    HStack::new(20.0)
                        .child(Text::new("Commander").erase())
                        .child(
                            MerkiBadge::new("Sector 7G")
                                .color([1.0, 0.84, 0.0, 1.0])
                                .erase(),
                        )
                        .erase(),
                )
                .erase(),
        )
        .child(
            // 2. Data Visualization
            HStack::new(20.0)
                .child(
                    ValkyrieAnalytics::new("Radar")
                        .chart_type(ChartType::Radar)
                        .data(vec![
                            ("A".to_string(), 0.8),
                            ("B".to_string(), 0.6),
                            ("C".to_string(), 0.9),
                            ("D".to_string(), 0.4),
                            ("E".to_string(), 0.7),
                        ])
                        .erase(),
                )
                .child(
                    TacticalGauge::new("Kinetics", 0.85)
                        .warning_level(0.7)
                        .critical_level(0.9)
                        .erase(),
                )
                .erase(),
        )
        .child(
            // 3. Narrative & History
            SagaAccordion::new()
                .item(
                    "Mission Log: Ragnarok Init",
                    UrdrTimeline::new()
                        .event("System Boot", "08:00")
                        .event("Niflheim Connection Established", "08:05")
                        .event("Anomaly Detected", "08:15")
                        .erase(),
                )
                .item("Resource Allocation", DraumaSkeleton::new().erase())
                .erase(),
        )
        .child(
            // 4. Interactive Tools
            HStack::new(20.0)
                .child(ValhallaRating::new(4.5).max(5).erase())
                .child(BifrostColorPicker::new([0.0, 0.8, 1.0, 1.0]).erase())
                .erase(),
        )
}

fn main() {
    println!("Cyberpunk Viking Tactical HUD Showcase initialized.");
    // In a real app, this would be passed to the cvkg renderer.
    let _view = tactical_hud_showcase();
}
