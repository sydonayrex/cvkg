/// Calculate the relative luminance of an sRGB color.
pub fn relative_luminance(color: [f32; 4]) -> f32 {
    let f = |c: f32| {
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * f(color[0]) + 0.7152 * f(color[1]) + 0.0722 * f(color[2])
}

/// Calculate the contrast ratio between two colors.
pub fn contrast_ratio(c1: [f32; 4], c2: [f32; 4]) -> f32 {
    let l1 = relative_luminance(c1);
    let l2 = relative_luminance(c2);
    let (light, dark) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (light + 0.05) / (dark + 0.05)
}
