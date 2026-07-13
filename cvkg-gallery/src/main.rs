//! cvkg Gallery — Component showcase using NiflheimSidebar + MimirSpotlight + macOS Tahoe design.

use cvkg_gallery::GalleryApp;
use cvkg::native::NativeRenderer;

// Entry point
fn main() {
    // Install panic hook that writes crash dump to disk
    std::panic::set_hook(Box::new(|info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };
        let bt = std::backtrace::Backtrace::force_capture();
        eprintln!("[CVKG PANIC] {msg}");
        eprintln!(
            "[CVKG PANIC] Backtrace:\n{bt}"
        );
        if let Ok(mut file) = std::fs::File::create("cvkg-crash.log") {
            use std::io::Write;
            let _ = writeln!(file, "CVKG Panic Dump");
            let _ = writeln!(file, "Message: {msg}");
            let _ = writeln!(
                file,
                "Backtrace:\n{bt}"
            );
        }
    }));

    NativeRenderer::run(GalleryApp::new(), None);
}