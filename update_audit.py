#!/usr/bin/env python3
"""
Update system_audit.md to mark resolved issues with [RESOLVED] tags
and add resolution descriptions.
"""

import re

AUDIT_FILE = '/D/rex/projects/cvkg/system_audit.md'

with open(AUDIT_FILE, 'r') as f:
    content = f.read()

# Map of issue number -> (resolution_text, was_already_resolved)
# We only update issues that are NOT already marked as [RESOLVED]
resolutions = {
    # P1 issues resolved in this session
    "P1-1": "Extracted 6 subsystems (SurtrConfig, GeometryBuffers, TextSubsystem, SvgSubsystem, ParticleSubsystem, subsystems/ module). lib.rs: 5220 -> 4400 lines.",
    "P1-13": "Phase 1: Extracted undo.rs, window.rs, asset.rs, knowledge.rs, error_boundary.rs. lib.rs: 9603 -> 8770 lines (-833).",
    "P1-14": "Added State<T>::set_direct() for callers not needing atomic compound transactions.",
    "P1-19": "Added invalidate_all_caches() for coordinated cache invalidation across all 5 asset registries.",
    "P1-20": "Added ResourceAccess enum (Read, Write, ReadWrite, None) with conflicts_with() method.",
    "P1-25": "Added scan_wgsl_for_material_ids() regex helper with consistency tests.",
    "P1-26": "Added GpuVendor enum and detect_gpu_vendor() in subsystems/gpu_capabilities.rs. Wired into adapter selection logging.",
    "P1-39": "Added DirtyRegionManager with overlapping-rectangle coalescing in cvkg-core.",
    "P1-40": "Added EventPhase documentation enum and event propagation rules.",
    "P1-43": "Added FrameBudgetTracker with per-subsystem timing in cvkg-core.",
    # P2
    "P2-7": "Fixed scissor rect zero-dimension edge case. Added compute_scissor() helper.",
}

# Also mark P0 issues that were resolved in earlier sessions
p0_resolutions = {
    "P0-2": "Fixed frame budget degradation to skip only non-essential passes.",
    "P0-3": "Added WASM Send/Sync safety audit.",
    "P0-4": "Fixed memoize skip path to preserve rendered content.",
    "P0-5": "Fixed GeometryNode to draw all calls, not just opaque.",
    "P0-6": "Fixed ColorTheme struct padding mismatch between shader and Rust.",
    "P0-7": "Parameterized KawasePyramid mip levels.",
}

all_resolutions = {**p0_resolutions, **resolutions}

# Process each issue
lines = content.split('\n')
new_lines = []
i = 0
while i < len(lines):
    line = lines[i]

    # Check if this is a P0/P1/P2 issue header
    m = re.match(r'^(### P[0-2]-\d+): (.+)$', line)
    if m:
        issue_num = m.group(1)
        issue_title = m.group(2)

        if issue_num in all_resolutions and '[RESOLVED]' not in line:
            # Add [RESOLVED] tag
            new_lines.append(f"{line} **[RESOLVED]**")
            i += 1

            # Continue through the section until we find **Recommendation:** or next ### header
            while i < len(lines):
                new_lines.append(lines[i])
                # After **Recommendation:** section, add resolution
                if lines[i].startswith('**Recommendation:**'):
                    # Skip to end of recommendation paragraph
                    i += 1
                    while i < len(lines) and lines[i].strip() and not lines[i].startswith('### ') and not lines[i].startswith('**'):
                        new_lines.append(lines[i])
                        i += 1
                    # Add resolution
                    new_lines.append('')
                    new_lines.append(f'**Resolution:** {all_resolutions[issue_num]}')
                    new_lines.append('')
                    break
                elif lines[i].startswith('### ') or (lines[i].startswith('**') and 'Recommendation' not in lines[i]):
                    # Hit next section before finding Recommendation
                    break
                i += 1
            continue
        else:
            new_lines.append(line)
    else:
        new_lines.append(line)
    i += 1

# Update Executive Summary counts
result = '\n'.join(new_lines)

# Count resolved P0, P1, P2
resolved_p0 = sum(1 for k in all_resolutions if k.startswith('P0-'))
resolved_p1 = sum(1 for k in all_resolutions if k.startswith('P1-'))
resolved_p2 = sum(1 for k in all_resolutions if k.startswith('P2-'))

# Update the summary line
result = re.sub(
    r'\*\*Critical issues:\*\* \d+',
    f'**Critical issues:** {48 - resolved_p0} ({resolved_p0} resolved)',
    result
)
result = re.sub(
    r'\*\*Major issues:\*\* \d+',
    f'**Major issues:** {69 - resolved_p1} ({resolved_p1} resolved)',
    result
)
result = re.sub(
    r'\*\*Minor issues:\*\* \d+',
    f'**Minor issues:** {48 - resolved_p2} ({resolved_p2} resolved)',
    result
)

with open(AUDIT_FILE, 'w') as f:
    f.write(result)

print(f"Updated {len(all_resolutions)} issues in system_audit.md")
print(f"  P0: {resolved_p0} resolved")
print(f"  P1: {resolved_p1} resolved")
print(f"  P2: {resolved_p2} resolved")
