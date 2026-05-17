// CVKG Norse Tools Example
// Demonstrates Phase 5 components: Bragi Creative, Hlin Accessibility, Eir Motion, Tyr Security
//
// Run with: cargo run --example norse_tools_example

use cvkg_components::eir_motion::Easing;
use cvkg_components::{
    A11yRole, BragiCreative, EirMotion, HlinAccessibility, PermissionLevel, TyrSecurity,
};

fn main() {
    println!("CVKG Norse Tools Example");
    println!("========================\n");

    // Bragi Creative - Creative suite
    let bragi = BragiCreative::new()
        .rich_text("main_doc", "Welcome to CVKG")
        .markdown("readme", "# Header\nContent here")
        .svg("logo")
        .active("main_doc");

    println!("Bragi Creative: {} components", bragi.components.len());

    // Hlin Accessibility - Accessibility infrastructure
    let hlin = HlinAccessibility::new()
        .node("btn_1", A11yRole::Button, "Submit")
        .node("nav_1", A11yRole::Navigation, "Main Menu")
        .node("main_1", A11yRole::Main, "Content Area")
        .high_contrast(true)
        .reduced_motion(true);

    println!("Hlin Accessibility: {} nodes", hlin.tree.len());

    // Eir Motion - Animation system
    let eir = EirMotion::new()
        .animation("fade_in", 0.5)
        .animation("slide_up", 0.3)
        .keyframe("fade_in", 0.0, 0.0, Easing::EaseOut)
        .keyframe("fade_in", 1.0, 1.0, Easing::EaseOut)
        .physics(100.0, 10.0)
        .state("running");

    println!("Eir Motion: {} animations", eir.animations.len());

    // Tyr Security - Security system
    let tyr = TyrSecurity::new()
        .role("viewer", PermissionLevel::User, vec!["read", "comment"])
        .role(
            "editor",
            PermissionLevel::Admin,
            vec!["read", "write", "delete"],
        )
        .audit("alice", "edit_document", "doc_123", true)
        .audit("bob", "delete_file", "file_456", false)
        .session("sess_abc", "workspace_1", 24.0);

    println!(
        "Tyr Security: {} roles, {} audit entries",
        tyr.roles.len(),
        tyr.audit_log.len()
    );

    println!("\n=== Norse Tools Components Created Successfully ===");
}
