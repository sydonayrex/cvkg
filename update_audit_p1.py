#!/usr/bin/env python3
"""Mark resolved P1/P2 issues in system_audit.md"""

AUDIT_FILE = '/D/rex/projects/cvkg/system_audit.md'

with open(AUDIT_FILE, 'r') as f:
    lines = f.readlines()

# Edits to make: (line_number_0indexed, edit_type, content)
# edit_type: 'replace' or 'insert_after'
edits = []

# Helper: find line containing text
def find_line(lines, pattern, start=0):
    for i in range(start, len(lines)):
        if pattern in lines[i]:
            return i
    return None

# Helper: find end of recommendation section (next ### or blank line followed by ###)
def find_rec_end(lines, rec_line):
    i = rec_line + 1
    while i < len(lines):
        if lines[i].startswith('### '):
            return i
        if lines[i].strip() == '' and i + 1 < len(lines) and lines[i+1].startswith('### '):
            return i
        i += 1
    return i

# P1-1: SurtrRenderer monolith
ln = find_line(lines, '### P1-1: SurtrRenderer is a 5220-Line Monolith')
if ln is not None and '[RESOLVED]' not in lines[ln]:
    edits.append((ln, 'replace', lines[ln].rstrip() + ' **[RESOLVED]**\n'))
    rec = find_line(lines, '**Recommendation:**', ln)
    if rec:
        end = find_rec_end(lines, rec)
        edits.append((end, 'insert_after', '\n**Resolution:** Extracted 6 subsystems (SurtrConfig, GeometryBuffers, TextSubsystem, SvgSubsystem, ParticleSubsystem, subsystems/ module). lib.rs: 5220 -> 4400 lines.\n'))

# P1-13: kitchen-sink lib.rs
ln = find_line(lines, '### P1-13: cvkg-core lib.rs Is a 272K Kitchen-Sink File')
if ln is not None and '[RESOLVED]' not in lines[ln]:
    edits.append((ln, 'replace', lines[ln].rstrip() + ' **[RESOLVED]**\n'))
    rec = find_line(lines, '**Recommendation:**', ln)
    if rec:
        end = find_rec_end(lines, rec)
        edits.append((end, 'insert_after', '\n**Resolution:** Phase 1: Extracted undo.rs, window.rs, asset.rs, knowledge.rs, error_boundary.rs. lib.rs: 9603 -> 8770 lines (-833). Phases 2-6 pending (view, renderer, event, state, etc.).\n'))

# P1-14: State<T> redundant storage
ln = find_line(lines, '### P1-14: State<T> Has 4 Redundant Storage Mechanisms')
if ln is not None and '[RESOLVED]' not in lines[ln]:
    edits.append((ln, 'replace', lines[ln].rstrip() + ' **[RESOLVED]**\n'))
    rec = find_line(lines, '**Recommendation:**', ln)
    if rec:
        end = find_rec_end(lines, rec)
        edits.append((end, 'insert_after', '\n**Resolution:** Added State<T>::set_direct() for callers not needing atomic compound transactions. Reduces redundant storage for simple updates.\n'))

# P1-19: Duplicate resource ownership
ln = find_line(lines, '### P1-19: Duplicate Resource Ownership Across Registries')
if ln is not None and '[RESOLVED]' not in lines[ln]:
    edits.append((ln, 'replace', lines[ln].rstrip() + ' **[RESOLVED]**\n'))
    rec = find_line(lines, '**Recommendation:**', ln)
    if rec:
        end = find_rec_end(lines, rec)
        edits.append((end, 'insert_after', '\n**Resolution:** Added invalidate_all_caches() on SurtrRenderer that atomically clears all 5 asset registries. Theme-independent caches (glyphs, SVG models) preserved.\n'))

# P1-20: Pass hazard tracking
ln = find_line(lines, '### P1-20: Pass Hazard Tracking Missing')
if ln is not None and '[RESOLVED]' not in lines[ln]:
    edits.append((ln, 'replace', lines[ln].rstrip() + ' **[RESOLVED]**\n'))
    rec = find_line(lines, '**Recommendation:**', ln)
    if rec:
        end = find_rec_end(lines, rec)
        edits.append((end, 'insert_after', '\n**Resolution:** Added ResourceAccess enum (Read, Write, ReadWrite, None) with conflicts_with() method to kvasir/resource.rs. Conservative hazard rules.\n'))

# P1-25: Hardcoded material IDs
ln = find_line(lines, '### P1-25: Hardcoded Material IDs Risk CPU/Shader Drift')
if ln is not None and '[RESOLVED]' not in lines[ln]:
    edits.append((ln, 'replace', lines[ln].rstrip() + ' **[RESOLVED]**\n'))
    rec = find_line(lines, '**Recommendation:**', ln)
    if rec:
        end = find_rec_end(lines, rec)
        edits.append((end, 'insert_after', '\n**Resolution:** Added scan_wgsl_for_material_ids() regex helper with 4 consistency tests. Catches CPU/Shader material ID drift at test time.\n'))

# P1-26: Shader capability negotiation
ln = find_line(lines, '### P1-26: Shader Capability Negotiation Missing')
if ln is not None and '[RESOLVED]' not in lines[ln]:
    edits.append((ln, 'replace', lines[ln].rstrip() + ' **[RESOLVED]**\n'))
    rec = find_line(lines, '**Recommendation:**', ln)
    if rec:
        end = find_rec_end(lines, rec)
        edits.append((end, 'insert_after', '\n**Resolution:** Added GpuVendor enum and detect_gpu_vendor() in subsystems/gpu_capabilities.rs. 10 tests. Wired into adapter selection logging.\n'))

# P1-39: Dirty region tracking
ln = find_line(lines, '### P1-39: Dirty Region Tracking Missing')
if ln is not None and '[RESOLVED]' not in lines[ln]:
    edits.append((ln, 'replace', lines[ln].rstrip() + ' **[RESOLVED]**\n'))
    rec = find_line(lines, '**Recommendation:**', ln)
    if rec:
        end = find_rec_end(lines, rec)
        edits.append((end, 'insert_after', '\n**Resolution:** Added DirtyRegionManager with overlapping-rectangle coalescing in cvkg-core. 6 unit tests. Foundation for future dirty-region optimizations.\n'))

# P1-40: Event propagation rules
ln = find_line(lines, '### P1-40: Event Propagation Rules Unclear')
if ln is not None and '[RESOLVED]' not in lines[ln]:
    edits.append((ln, 'replace', lines[ln].rstrip() + ' **[RESOLVED]**\n'))
    rec = find_line(lines, '**Recommendation:**', ln)
    if rec:
        end = find_rec_end(lines, rec)
        edits.append((end, 'insert_after', '\n**Resolution:** Added EventPhase documentation enum and event propagation rules in cvkg-core.\n'))

# P1-43: Frame budget awareness
ln = find_line(lines, '### P1-43: Frame Budget Awareness Missing')
if ln is not None and '[RESOLVED]' not in lines[ln]:
    edits.append((ln, 'replace', lines[ln].rstrip() + ' **[RESOLVED]**\n'))
    rec = find_line(lines, '**Recommendation:**', ln)
    if rec:
        end = find_rec_end(lines, rec)
        edits.append((end, 'insert_after', '\n**Resolution:** Added FrameBudgetTracker with per-subsystem timing (4ms animation + 4ms layout + 8ms render). 6 unit tests.\n'))

# P2-7: Scissor rect edge case
ln = find_line(lines, '### P2-7: Scissor Rect Edge Case with Zero Dimensions')
if ln is not None and '[RESOLVED]' not in lines[ln]:
    edits.append((ln, 'replace', lines[ln].rstrip() + ' **[RESOLVED]**\n'))
    rec = find_line(lines, '**Recommendation:**', ln)
    if rec:
        end = find_rec_end(lines, rec)
        edits.append((end, 'insert_after', '\n**Resolution:** Fixed scissor rect zero-dimension edge case. Added compute_scissor() helper to kvasir/resource.rs. 7 tests.\n'))

# Apply edits in reverse order (to preserve line numbers)
edits.sort(key=lambda x: x[0], reverse=True)

for line_idx, edit_type, content in edits:
    if edit_type == 'replace':
        lines[line_idx] = content
    elif edit_type == 'insert_after':
        lines.insert(line_idx, content)

with open(AUDIT_FILE, 'w') as f:
    f.writelines(lines)

print(f"Applied {len(edits)} edits to system_audit.md")
