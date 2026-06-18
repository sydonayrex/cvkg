#!/usr/bin/env python3
"""
P1-13: Extract modules from cvkg-core/src/lib.rs
"""

import os

LIB_RS = '/D/rex/projects/cvkg/cvkg-core/src/lib.rs'

# Read original
with open(LIB_RS, 'r') as f:
    original_lines = f.readlines()

# Define sections to extract as (start_line_1indexed, end_line_1indexed_inclusive)
SECTIONS = [
    (45, 258, 'error_boundary.rs'),
    (260, 437, 'knowledge.rs'),
    (439, 639, 'undo.rs'),
    (641, 782, 'window.rs'),
    (784, 890, 'asset.rs'),
]

# For each section, extract the content and create the module file
for start, end, filename in SECTIONS:
    section_lines = original_lines[start-1:end]
    filepath = os.path.join(os.path.dirname(LIB_RS), filename)
    with open(filepath, 'w') as f:
        f.writelines(section_lines)
    print(f"Extracted lines {start}-{end} -> {filename} ({len(section_lines)} lines)")

# Build the new lib.rs
skip_ranges = [(s-1, e) for s, e, _ in SECTIONS]

def should_skip(line_idx):
    for start, end in skip_ranges:
        if start <= line_idx <= end:
            return True
    return False

new_lines = []
for i, line in enumerate(original_lines):
    if should_skip(i):
        continue
    new_lines.append(line)

# Find insertion point (after "pub use future_views::...")
insert_idx = None
for i, line in enumerate(new_lines):
    if 'pub use future_views::' in line:
        insert_idx = i + 1
        break

if insert_idx is None:
    print("ERROR: Could not find insertion point")
    exit(1)

# Insert module declarations (each as a separate string in the list)
module_decls = [
    '\n',
    '// P1-13: extracted modules\n',
    'pub mod asset;\n',
    'pub mod error_boundary;\n',
    'pub mod knowledge;\n',
    'pub mod undo;\n',
    'pub mod window;\n',
    '\n',
    '// P1-13: re-exports for backward compatibility\n',
    'pub use asset::{AssetState, TokenValue, YggdrasilTokens};\n',
    'pub use error_boundary::{ComponentErrorState, ErrorBoundary};\n',
    'pub use knowledge::{AnnouncementPriority, KnowledgeFragment, KnowledgeId, KnowledgeState, MemoryLayer, Realm, TemporalEdge, TemporalNode};\n',
    'pub use undo::{UndoGroup, UndoManager};\n',
    'pub use window::{Window, WindowCloseAction, WindowConfig, WindowHandle, WindowId, WindowLevel};\n',
]

for j, line in enumerate(module_decls):
    new_lines.insert(insert_idx + j, line)

# Write the new lib.rs
with open(LIB_RS, 'w') as f:
    f.writelines(new_lines)

print(f"\nOriginal: {len(original_lines)} lines")
print(f"New: {len(new_lines)} lines")
print(f"Net change: {len(new_lines) - len(original_lines)} lines")
