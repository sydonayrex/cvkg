use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[test]
fn discover_incomplete_code_placeholders() {
    let workspace_root = Path::new(".."); // Adjusted for tests/ directory execution
    let mut incomplete_areas = Vec::new();

    let patterns = vec!["TODO", "FIXME", "STUB", "unimplemented!", "todo!", "unreachable!"];

    for entry in WalkDir::new(workspace_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs")) 
    {
        let content = fs::read_to_string(entry.path()).unwrap_or_default();
        for (line_num, line) in content.lines().enumerate() {
            for pattern in &patterns {
                if line.contains(pattern) {
                    // Ignore intentional unreachables in fn body for primitives
                    if pattern == &"unreachable!" && line.contains("fn body") {
                        continue;
                    }
                    incomplete_areas.push(format!(
                        "{}:{}: [{}] {}",
                        entry.path().display(),
                        line_num + 1,
                        pattern,
                        line.trim()
                    ));
                }
            }
        }
    }

    if !incomplete_areas.is_empty() {
        println!("\n=== Incomplete Code Placeholders Found ===");
        for area in &incomplete_areas {
            println!("{}", area);
        }
        println!("==========================================\n");
    }

    // We don't necessarily want this to FAIL the build yet, but we want it visible.
    // However, the user asked to "identify" them.
}
