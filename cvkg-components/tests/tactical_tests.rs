use cvkg_components::*;
use cvkg_core::{Rect, Renderer, View};

#[test]
fn test_tactical_component_rendering() {
    // This test ensures the new tactical components can be instantiated and "rendered"
    // without panicking. In a real environment, we'd use a MockRenderer.
    
    let alert = GjallarAlert::new("System Alert", "Breach detected")
        .kind(AlertKind::Critical);
    
    let rating = ValhallaRating::new(4.0).max(5);
    let picker = BifrostColorPicker::new([1.0, 0.0, 0.0, 1.0]);
    let card = RunesCard::new().header(Text::new("Tactical Card"));
    
    // Check if they implement View (compile time check)
    fn assert_view<V: View>(_: &V) {}
    assert_view(&alert);
    assert_view(&rating);
    assert_view(&picker);
    assert_view(&card);
    
    println!("Tactical components verified.");
}

#[test]
fn test_valkyrie_analytics_radar() {
    let analytics = ValkyrieAnalytics::new(ChartType::Radar, vec![0.1, 0.5, 0.9]);
    assert_eq!(analytics.data.len(), 3);
}
