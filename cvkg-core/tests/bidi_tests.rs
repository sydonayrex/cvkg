use cvkg_core::Rect;
use cvkg_core::{BidiDirection, mirror_rect};

#[test]
fn auto_direction_uses_first_strong_character() {
    assert_eq!(BidiDirection::Auto.resolve("hello"), BidiDirection::Ltr);
    assert_eq!(BidiDirection::Auto.resolve("مرحبا"), BidiDirection::Rtl);
    assert_eq!(BidiDirection::Auto.resolve("שלום"), BidiDirection::Rtl);
    assert_eq!(BidiDirection::Auto.resolve("123 hello"), BidiDirection::Ltr);
    assert_eq!(BidiDirection::Rtl.resolve("hello"), BidiDirection::Rtl);
}

#[test]
fn mirroring_flips_horizontal_position_within_the_container() {
    let mirrored = mirror_rect(Rect::new(10.0, 4.0, 20.0, 8.0), 100.0);
    assert_eq!(mirrored.x, 70.0);
    assert_eq!(mirrored.y, 4.0);
    assert_eq!(mirrored.width, 20.0);
    assert_eq!(mirrored.height, 8.0);
}
