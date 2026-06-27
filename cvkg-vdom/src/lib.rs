//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     -- State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     -- Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     -- Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    -- Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     -- Read the target, its surrounding context, and its full call graph
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     -- Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   -- Check every tool call / command for progress every 30 seconds.
//!                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//!                      and move to unblocked work. Never silently accept a broken state.
//!
//! Sources:
//!   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//!   CVKG Extended: Section 2 of the CVKG Design Specification

//! Virtual DOM implementation for CVKG

pub mod accesskit_bridge;
pub mod animated;
pub mod diff;
pub mod physics;
pub mod signals;
pub mod vnode;

pub use cvkg_core::KvasirId;
use cvkg_core::Renderer;
use std::collections::HashMap;

// Public re-exports to ensure zero breaking changes for callers
pub use accesskit_bridge::A11yNodeEntry;
pub use diff::VDomPatch;
pub use vnode::{
    AriaProps, DecorativeCmd, EventHandlerMap, LayoutRect, NodeEventHandlerMap, NodeId, VNode,
};

/// The root container for the Virtual DOM state.
pub struct VDom {
    /// The root node ID
    pub root: Option<NodeId>,
    /// Flattened map of all nodes currently in the VDOM
    pub nodes: HashMap<NodeId, VNode>,
    /// Parent mapping for O(1) event bubbling
    pub parents: HashMap<NodeId, NodeId>,
    /// Currently focused node for keyboard events
    pub focused_node: std::sync::Mutex<Option<NodeId>>,
    /// Currently captured node for pointer events
    pub captured_node: std::sync::Mutex<Option<NodeId>>,
    /// Currently hovered node for pointer events
    pub hovered_node: std::sync::Mutex<Option<NodeId>>,
    /// Centralized event handlers for efficient delegation
    pub event_handlers: NodeEventHandlerMap,
}

impl Default for VDom {
    fn default() -> Self {
        Self::new()
    }
}

impl VDom {
    /// Create a new empty VDom.
    pub fn new() -> Self {
        Self {
            root: None,
            nodes: HashMap::new(),
            parents: HashMap::new(),
            focused_node: std::sync::Mutex::new(None),
            captured_node: std::sync::Mutex::new(None),
            hovered_node: std::sync::Mutex::new(None),
            event_handlers: HashMap::new(),
        }
    }

    /// Build a VDom tree from a view by performing a virtual render pass.
    pub fn build<V: cvkg_core::View>(view: &V, rect: cvkg_core::Rect) -> Self {
        let mut renderer = VNodeRenderer::new();
        view.render(&mut renderer, rect);
        renderer.into_vdom()
    }

    /// Phase 4.4: Prepare this VDom to receive a new frame's nodes by clearing
    /// data while retaining allocated capacity in all HashMaps.
    ///
    /// Call this at the start of a frame rebuild (before rebuilding) to reuse the
    /// previous frame's heap allocations. This avoids repeated allocator churn
    /// when the VDom is rebuilt every frame.
    pub fn clear_and_retain_capacity(&mut self) {
        self.root = None;
        self.nodes.clear();
        self.parents.clear();
        self.event_handlers.clear();
        // Note: focused_node, captured_node, hovered_node are Mutex<Option<NodeId>>
        // and are not cleared here -- they carry state across frames.
    }

    /// Apply a set of patches to the host's DOM environment.
    pub fn apply_to_dom(&self, patches: &[VDomPatch]) {
        // This is a bridge to the platform-specific accessibility tree (ShieldWall).
        log::debug!("Applying {} patches to host ShieldWall", patches.len());
        for patch in patches {
            match patch {
                VDomPatch::Create(node) => log::debug!("ShieldWall: Create node {}", node.id.0),
                VDomPatch::Update { id, .. } => log::debug!("ShieldWall: Update node {}", id.0),
                VDomPatch::Remove(id) => log::debug!("ShieldWall: Remove node {}", id.0),
                VDomPatch::Replace { id, .. } => log::debug!("ShieldWall: Replace node {}", id.0),
                VDomPatch::Move { id, .. } => log::debug!("ShieldWall: Move node {}", id.0),
                VDomPatch::SetRoot(id) => log::debug!("ShieldWall: SetRoot {:?}", id),
                VDomPatch::ClearHandlers { id } => {
                    log::debug!("ShieldWall: ClearHandlers for node {}", id.0)
                }
            }
        }
    }

    /// Check if the VDOM and a SceneGraph are in perfect synchronization.
    /// Returns Err if corruption is detected, signaling a full rebuild is required.
    pub fn validate_sync(&self, scene: &cvkg_scene::SceneGraph) -> Result<(), String> {
        let _span = tracing::info_span!("vdom_validate_sync").entered();

        // 1. Root parity
        match (self.root, scene.root) {
            (None, None) => return Ok(()),
            (Some(vr), Some(sr)) if vr.0 == sr.0 => {}
            _ => return Err("Root node mismatch".to_string()),
        }

        // 2. Node count parity (approximate check for performance)
        if self.nodes.len() != scene.nodes.len() {
            return Err(format!(
                "Node count mismatch: VDom({}) vs SceneGraph({})",
                self.nodes.len(),
                scene.nodes.len()
            ));
        }

        // 3. Hierarchical Consistency Check (DFS)
        if let Some(root_id) = self.root {
            self.validate_node_sync(root_id, scene)?;
        }

        Ok(())
    }

    fn validate_node_sync(&self, id: NodeId, scene: &cvkg_scene::SceneGraph) -> Result<(), String> {
        let vnode = self
            .nodes
            .get(&id)
            .ok_or_else(|| format!("Node {} missing in VDom", id.0))?;
        let snode = scene
            .nodes
            .get(&cvkg_core::KvasirId(id.0))
            .ok_or_else(|| format!("Node {} missing in SceneGraph", id.0))?;

        // Check child count and IDs
        if vnode.children.len() != snode.children.len() {
            return Err(format!("Child count mismatch for node {}", id.0));
        }

        for (v_child, s_child) in vnode.children.iter().zip(snode.children.iter()) {
            if v_child.0 != s_child.0 {
                return Err(format!(
                    "Child ID mismatch in node {}: {} != {}",
                    id.0, v_child.0, s_child.0
                ));
            }
            self.validate_node_sync(*v_child, scene)?;
        }

        // Check visual bounds (within tolerance)
        let tolerance = 0.5;
        if (vnode.layout.x - snode.world_rect.x).abs() > tolerance
            || (vnode.layout.y - snode.world_rect.y).abs() > tolerance
        {
            return Err(format!("Spatial drift detected in node {}", id.0));
        }

        Ok(())
    }

    /// Mutate the Virtual DOM state by applying a sequence of patches.
    #[tracing::instrument(skip(self, patches))]
    pub fn apply_patches(&mut self, patches: Vec<VDomPatch>) {
        if !patches.is_empty() {
            println!("VDom: Applying {} patches", patches.len());
        }
        let _span = tracing::info_span!("vdom_apply_patches").entered();
        for patch in patches {
            match patch {
                VDomPatch::Create(node) => {
                    for child_id in &node.children {
                        self.parents.insert(*child_id, node.id);
                    }
                    self.nodes.insert(node.id, node);
                }
                VDomPatch::Update {
                    id,
                    props,
                    layout,
                    aria_props,
                    aria_role,
                    children,
                    handlers,
                    sdf_shape,
                } => {
                    if let Some(node) = self.nodes.get_mut(&id) {
                        if let Some(p) = props {
                            node.props = p;
                        }
                        if let Some(l) = layout {
                            node.layout = l;
                        }
                        if let Some(ap) = aria_props {
                            node.aria_props = ap;
                        }
                        if let Some(ar) = aria_role {
                            node.aria_role = ar;
                        }
                        if let Some(c) = children {
                            // Update children and parents map
                            for child_id in &node.children {
                                self.parents.remove(child_id);
                            }
                            node.children = c;
                            for child_id in &node.children {
                                self.parents.insert(*child_id, id);
                            }
                        }
                        if let Some(h) = handlers {
                            self.event_handlers.insert(id, h);
                        }
                        if let Some(s) = sdf_shape {
                            node.sdf_shape = Some(s);
                        }
                    }
                }
                VDomPatch::Remove(id) => {
                    if let Some(node) = self.nodes.remove(&id) {
                        for child_id in &node.children {
                            self.parents.remove(child_id);
                        }
                    }
                    self.parents.remove(&id);
                }
                VDomPatch::Replace { id, node } => {
                    let is_root = self.root == Some(id);
                    let new_id = node.id;

                    // Cleanup old children from parents map
                    if let Some(old_node) = self.nodes.get(&id) {
                        for child_id in &old_node.children {
                            self.parents.remove(child_id);
                        }
                    }
                    for child_id in &node.children {
                        self.parents.insert(*child_id, new_id);
                    }

                    // Update nodes map. We use the new_id as the key to keep it consistent.
                    self.nodes.remove(&id);
                    self.nodes.insert(new_id, node);

                    if is_root {
                        self.root = Some(new_id);
                    }

                    // Migrate capture and focus state
                    if let Ok(mut capture) = self.captured_node.lock()
                        && *capture == Some(id)
                    {
                        *capture = Some(new_id);
                    }
                    if let Ok(mut focus) = self.focused_node.lock()
                        && *focus == Some(id)
                    {
                        *focus = Some(new_id);
                    }
                }
                VDomPatch::Move { id, new_index } => {
                    if let Some(&p_id) = self.parents.get(&id)
                        && let Some(parent) = self.nodes.get_mut(&p_id)
                        && let Some(old_pos) = parent.children.iter().position(|&x| x == id)
                    {
                        parent.children.remove(old_pos);
                        let target_pos = new_index.min(parent.children.len());
                        parent.children.insert(target_pos, id);
                    }
                }
                VDomPatch::SetRoot(id) => {
                    self.root = id;
                }
                VDomPatch::ClearHandlers { id } => {
                    // P0-6 fix: explicitly clear handlers for this node
                    self.event_handlers.remove(&id);
                }
            }
        }
    }

    fn sdf_distance(
        shape: Option<&cvkg_core::layout::SdfShape>,
        layout: &LayoutRect,
        x: f32,
        y: f32,
    ) -> f32 {
        let shape =
            shape
                .copied()
                .unwrap_or(cvkg_core::layout::SdfShape::Rect(cvkg_core::layout::Rect {
                    x: layout.x,
                    y: layout.y,
                    width: layout.width,
                    height: layout.height,
                }));
        match shape {
            cvkg_core::layout::SdfShape::Rect(r) => {
                let dx = (r.x - x).max(x - (r.x + r.width)).max(0.0);
                let dy = (r.y - y).max(y - (r.y + r.height)).max(0.0);
                if dx == 0.0 && dy == 0.0 {
                    let in_x = (x - r.x).min(r.x + r.width - x);
                    let in_y = (y - r.y).min(r.y + r.height - y);
                    -in_x.min(in_y)
                } else {
                    (dx * dx + dy * dy).sqrt()
                }
            }
            cvkg_core::layout::SdfShape::RoundedRect { rect: r, radius } => {
                let hw = r.width / 2.0;
                let hh = r.height / 2.0;
                let cx = r.x + hw;
                let cy = r.y + hh;
                let dx = (x - cx).abs() - hw + radius;
                let dy = (y - cy).abs() - hh + radius;
                dx.max(0.0).hypot(dy.max(0.0)) + dx.max(dy).min(0.0) - radius
            }
            cvkg_core::layout::SdfShape::Circle { center, radius } => {
                ((x - center[0]).powi(2) + (y - center[1]).powi(2)).sqrt() - radius
            }
        }
    }

    /// Perform hit testing to find the front-most node at the given coordinates.
    ///
    /// WHY: Looks up node at coordinates.
    /// CONTRACT: Uses pointer_precision to define the maximum expansion radius for hit matching.
    pub fn hit_test(&self, x: f32, y: f32, pointer_precision: f32) -> Option<(NodeId, f32)> {
        self.root
            .and_then(|root_id| self.hit_test_recursive(root_id, x, y, pointer_precision))
    }

    /// Perform recursive hit-testing down the VDOM tree.
    fn hit_test_recursive(
        &self,
        node_id: NodeId,
        x: f32,
        y: f32,
        pointer_precision: f32,
    ) -> Option<(NodeId, f32)> {
        let node = self.nodes.get(&node_id)?;

        let dist = Self::sdf_distance(node.sdf_shape.as_ref(), &node.layout, x, y);

        // Scale proximity limit based on the precision of the pointer device.
        let proximity_limit = pointer_precision.max(0.0);
        let proximity = if dist <= 0.0 {
            1.0
        } else if proximity_limit > 0.0 {
            (1.0 - (dist / proximity_limit)).clamp(0.0, 1.0)
        } else {
            0.0
        };

        if proximity > 0.0 {
            // Search children in reverse (front-to-back) to maintain proper Z-ordering.
            let mut best_child_hit: Option<(NodeId, f32)> = None;

            // Hit test policy: if the click is outside the menu bar (y >= 28.0),
            // evaluate DropdownOverlay last so it doesn't block sibling interactive elements.
            let mut children_to_test = node.children.clone();
            if y >= 28.0
                && let Some(pos) = children_to_test.iter().position(|&cid| {
                    self.nodes
                        .get(&cid)
                        .is_some_and(|n| n.component_type == "DropdownOverlay")
                })
            {
                let overlay_id = children_to_test.remove(pos);
                children_to_test.insert(0, overlay_id);
            }

            for child_id in children_to_test.iter().rev() {
                if let Some((hit, hit_prox)) =
                    self.hit_test_recursive(*child_id, x, y, pointer_precision)
                {
                    // Direct hit (point inside child's SDF): return immediately
                    if hit_prox >= 1.0 {
                        return Some((hit, hit_prox));
                    }
                    // Track best partial hit among children
                    if best_child_hit.is_none() || hit_prox > best_child_hit.unwrap().1 {
                        best_child_hit = Some((hit, hit_prox));
                    }
                }
            }

            // If any child matched (even partially), prefer the best child hit
            if let Some(bh) = best_child_hit {
                return Some(bh);
            }

            // No child matched at all -- return this node if it's interactive
            if dist <= 0.0 || self.event_handlers.contains_key(&node_id) {
                return Some((node_id, proximity));
            }
        }

        None
    }

    /// Dispatch an event to the VDOM by performing a hit test and calling the handler.
    pub fn dispatch_event(&self, mut event: cvkg_core::Event) -> cvkg_core::EventResponse {
        let _span = tracing::info_span!("vdom_dispatch_event").entered();
        let event_name = event.name();

        log::trace!("[VDOM] DISPATCH: {} (root={:?})", event_name, self.root);

        let captured_target = self
            .captured_node
            .lock()
            .ok()
            .and_then(|captured| *captured);
        let target_id = match event {
            cvkg_core::Event::PointerDown { x, y, .. }
            | cvkg_core::Event::PointerUp { x, y, .. }
            | cvkg_core::Event::PointerMove { x, y, .. }
            | cvkg_core::Event::PointerClick { x, y, .. }
            | cvkg_core::Event::PointerWheel { x, y, .. }
            | cvkg_core::Event::PointerDoubleClick { x, y, .. }
            | cvkg_core::Event::DragStart { x, y, .. }
            | cvkg_core::Event::DragMove { x, y, .. }
            | cvkg_core::Event::DragEnd { x, y, .. }
            | cvkg_core::Event::FileDrop { x, y, .. } => {
                let use_capture = matches!(
                    event,
                    cvkg_core::Event::PointerUp { .. }
                        | cvkg_core::Event::PointerClick { .. }
                        | cvkg_core::Event::DragMove { .. }
                        | cvkg_core::Event::DragEnd { .. }
                ) && captured_target.is_some();

                let (id, proximity) = if use_capture {
                    let captured = captured_target;
                    log::trace!(
                        "[VDOM] Using captured target for {}: {:?}",
                        event_name,
                        captured
                    );
                    (captured, 1.0)
                } else {
                    log::trace!(
                        "[VDOM] Hit testing at ({}, {}) with precision {}",
                        x,
                        y,
                        event.pointer_precision()
                    );
                    let (id, proximity) = match self.hit_test(x, y, event.pointer_precision()) {
                        Some((i, p)) => (Some(i), p),
                        None => (None, 0.0),
                    };
                    log::trace!("[VDOM] Hit test result: {:?}, proximity: {}", id, proximity);
                    (id, proximity)
                };

                if let cvkg_core::Event::PointerMove {
                    ref mut proximity_field,
                    ..
                } = event
                {
                    *proximity_field = proximity;
                }
                if let cvkg_core::Event::PointerDown {
                    ref mut proximity_field,
                    ..
                } = event
                {
                    *proximity_field = proximity;
                }

                // Update focus/capture/hover state
                if let cvkg_core::Event::PointerDown { .. } = event {
                    if let Ok(mut focus) = self.focused_node.lock() {
                        *focus = id;
                    }
                    if let Ok(mut capture) = self.captured_node.lock() {
                        *capture = id;
                    }
                }
                if let cvkg_core::Event::PointerUp { .. } = event
                    && let Ok(mut capture) = self.captured_node.lock()
                {
                    *capture = None;
                }

                // Handle hover transitions
                if let cvkg_core::Event::PointerMove { .. } = event {
                    let old_hover = if let Ok(mut hover) = self.hovered_node.lock() {
                        let prev = *hover;
                        *hover = id;
                        prev
                    } else {
                        None
                    };

                    if old_hover != id {
                        if let Some(old_id) = old_hover {
                            self.bubble_event(old_id, &cvkg_core::Event::PointerLeave);
                        }
                        if let Some(new_id) = id {
                            self.bubble_event(new_id, &cvkg_core::Event::PointerEnter);
                        }
                    }
                }

                id
            }
            cvkg_core::Event::FocusIn | cvkg_core::Event::FocusOut => {
                self.focused_node.lock().ok().and_then(|f| *f)
            }
            _ => self.focused_node.lock().ok().and_then(|f| *f),
        };

        if let Some(id) = target_id {
            log::trace!(
                "[VDOM] Dispatching {} to node {:?} ({})",
                event_name,
                id,
                self.nodes
                    .get(&id)
                    .map(|n| n.component_type.as_str())
                    .unwrap_or("UNKNOWN")
            );
            self.bubble_event_response(id, event)
        } else {
            log::trace!("[VDOM] No hit for event {} at {:?}", event_name, event);
            cvkg_core::EventResponse::Ignored
        }
    }

    /// Dispatch an event directly to a specific node and bubble upward from there.
    pub fn dispatch_event_to_target(
        &self,
        target: NodeId,
        event: cvkg_core::Event,
    ) -> cvkg_core::EventResponse {
        if matches!(event, cvkg_core::Event::PointerUp { .. } | cvkg_core::Event::PointerClick { .. })
            && let Ok(mut capture) = self.captured_node.lock()
        {
            *capture = None;
        }

        if self.nodes.contains_key(&target) {
            self.bubble_event_response(target, event)
        } else {
            cvkg_core::EventResponse::Ignored
        }
    }

    /// Bubble an event up the tree from a target node to the root.
    pub fn bubble_event(&self, target: NodeId, event: &cvkg_core::Event) -> bool {
        let event_name = event.name();
        let mut current_id = target;
        let mut processed = false;

        loop {
            if let Some(handlers) = self.event_handlers.get(&current_id)
                && let Some(handler) = handlers.get(event_name)
            {
                handler(event.clone());
                processed = true;
            }

            if let Some(parent_id) = self.parents.get(&current_id) {
                current_id = *parent_id;
            } else {
                break;
            }
        }

        processed
    }

    /// Internal helper that dispatches and converts to EventResponse.
    fn bubble_event_response(
        &self,
        mut current_id: NodeId,
        event: cvkg_core::Event,
    ) -> cvkg_core::EventResponse {
        let event_name = event.name();
        let mut processed = false;
        let target = current_id;

        loop {
            if let Some(handlers) = self.event_handlers.get(&current_id) {
                log::trace!("[VDOM] Checking node {:?} for handlers", current_id);
                if let Some(handler) = handlers.get(event_name) {
                    log::debug!(
                        "[VDOM] Executing handler for '{}' on node {:?}",
                        event_name,
                        current_id
                    );
                    handler(event.clone());
                    processed = true;
                }
            }

            if let Some(parent_id) = self.parents.get(&current_id) {
                current_id = *parent_id;
            } else {
                break;
            }
        }

        if !processed {
            let mut stack = vec![target];
            while let Some(node_id) = stack.pop() {
                if node_id != target
                    && let Some(handlers) = self.event_handlers.get(&node_id)
                    && let Some(handler) = handlers.get(event_name)
                {
                    log::debug!(
                        "[VDOM] Found descendant handler on {:?} for '{}'",
                        node_id,
                        event_name
                    );
                    handler(event.clone());
                    processed = true;
                    break;
                }
                if let Some(node) = self.nodes.get(&node_id) {
                    for child_id in &node.children {
                        stack.push(*child_id);
                    }
                }
            }
        }

        if processed {
            cvkg_core::EventResponse::Handled
        } else {
            cvkg_core::EventResponse::Ignored
        }
    }

    /// Set the focused node by ID.
    pub fn focus_node(&self, id: NodeId) {
        if let Ok(mut focus) = self.focused_node.lock() {
            *focus = Some(id);
        }
    }

    /// Clear the current focus.
    pub fn blur_node(&self) {
        if let Ok(mut focus) = self.focused_node.lock() {
            *focus = None;
        }
    }

    /// Build the document-order (DFS pre-order) list of node IDs.
    fn build_focus_order(&self) -> Vec<NodeId> {
        let mut order = Vec::new();
        if let Some(root_id) = self.root {
            self.dfs_pre_order(root_id, &mut order);
        }
        order
    }

    fn dfs_pre_order(&self, node_id: NodeId, order: &mut Vec<NodeId>) {
        order.push(node_id);
        if let Some(node) = self.nodes.get(&node_id) {
            for child_id in &node.children {
                self.dfs_pre_order(*child_id, order);
            }
        }
    }

    /// Move focus to the next node in document order (DFS pre-order).
    pub fn focus_next(&self) {
        let order = self.build_focus_order();
        if order.is_empty() {
            return;
        }

        let current = self.focused_node.lock().ok().and_then(|f| *f);
        match current {
            None => {
                if let Ok(mut focus) = self.focused_node.lock() {
                    *focus = Some(order[0]);
                }
            }
            Some(cur_id) => {
                if let Some(pos) = order.iter().position(|id| *id == cur_id) {
                    let next = (pos + 1) % order.len();
                    if let Ok(mut focus) = self.focused_node.lock() {
                        *focus = Some(order[next]);
                    }
                }
            }
        }
    }

    /// Move focus to the previous node in document order (DFS pre-order).
    pub fn focus_prev(&self) {
        let order = self.build_focus_order();
        if order.is_empty() {
            return;
        }

        let current = self.focused_node.lock().ok().and_then(|f| *f);
        match current {
            None => {
                if let Ok(mut focus) = self.focused_node.lock() {
                    *focus = Some(order[order.len() - 1]);
                }
            }
            Some(cur_id) => {
                if let Some(pos) = order.iter().position(|id| *id == cur_id) {
                    let prev = if pos == 0 { order.len() - 1 } else { pos - 1 };
                    if let Ok(mut focus) = self.focused_node.lock() {
                        *focus = Some(order[prev]);
                    }
                }
            }
        }
    }
}

/// A specialized renderer that captures the component hierarchy as a Virtual DOM.
pub struct VNodeRenderer {
    nodes: HashMap<NodeId, VNode>,
    event_handlers: NodeEventHandlerMap,
    next_id: u64,
    stack: Vec<NodeId>,
    clip_stack: Vec<cvkg_core::Rect>,
    root: Option<NodeId>,
    /// Phase 4.1: Accumulated decorative commands since last flush.
    decorative_batch: Vec<DecorativeCmd>,
    /// Phase 4.1: The node ID of the current batch VNode, if any.
    batch_node_id: Option<NodeId>,
    /// Phase 2 fix: long-lived text engine, constructed once per VNodeRenderer.
    text_engine: cvkg_runic_text::TextEngine,
}

impl Default for VNodeRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl VNodeRenderer {
    /// Create a new VNodeRenderer.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            event_handlers: HashMap::new(),
            next_id: 1,
            stack: Vec::new(),
            clip_stack: Vec::new(),
            root: None,
            decorative_batch: Vec::new(),
            batch_node_id: None,
            text_engine: {
                let mut engine = cvkg_runic_text::TextEngine::new_light();
                engine.load_font_data(include_bytes!("../Fonts/Jupiteroid.ttf").to_vec());
                engine
            },
        }
    }

    /// Convert the captured nodes into a VDom instance.
    pub fn into_vdom(mut self) -> VDom {
        self.flush_decorative_batch();
        log::debug!("[VDOM] Built VDOM with {} nodes", self.nodes.len());
        let mut parents = HashMap::new();
        for (id, node) in &self.nodes {
            for child_id in &node.children {
                parents.insert(*child_id, *id);
            }
        }
        VDom {
            root: self.root,
            nodes: self.nodes,
            parents,
            focused_node: std::sync::Mutex::new(None),
            captured_node: std::sync::Mutex::new(None),
            hovered_node: std::sync::Mutex::new(None),
            event_handlers: self.event_handlers,
        }
    }

    fn next_id(&mut self) -> NodeId {
        let id = KvasirId(self.next_id);
        self.next_id += 1;
        id
    }

    fn stable_id_for(&self, component_type: &str, key: Option<&str>) -> Option<NodeId> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let key = key?;
        let mut hasher = DefaultHasher::new();
        component_type.hash(&mut hasher);
        key.hash(&mut hasher);
        let hash = hasher.finish();
        Some(KvasirId(
            0x8000_0000_0000_0000 | (hash & 0x7FFF_FFFF_FFFF_FFFF),
        ))
    }

    fn add_node(&mut self, mut node: VNode) -> NodeId {
        if let Some(stable_id) = self.stable_id_for(&node.component_type, node.key.as_deref()) {
            node.id = stable_id;
        }
        let id = node.id;
        log::trace!(
            "[VDOM] Adding node {:?} ({}): {:?}",
            id,
            node.component_type,
            node.layout
        );
        if let Some(parent_id) = self.stack.last() {
            if let Some(parent) = self.nodes.get_mut(parent_id) {
                parent.children.push(id);
            }
        } else if self.root.is_none() {
            self.root = Some(id);
        }
        self.nodes.insert(id, node);
        id
    }

    fn flush_decorative_batch(&mut self) {
        if self.decorative_batch.is_empty() {
            return;
        }
        if let Some(batch_id) = self.batch_node_id {
            if let Some(node) = self.nodes.get_mut(&batch_id) {
                node.props.insert(
                    "commands".to_string(),
                    serde_json::to_value(&self.decorative_batch).unwrap_or_default(),
                );
            }
            if self.stack.last() == Some(&batch_id) {
                self.stack.pop();
            }
        }
        self.decorative_batch.clear();
        self.batch_node_id = None;
    }

    fn begin_decorative(&mut self, rect: cvkg_core::Rect) {
        if self.batch_node_id.is_none() {
            if self.stack.last().is_none() {
                return;
            }
            let id = self.next_id();
            self.batch_node_id = Some(id);
            let batch_node = VNode {
                id,
                key: None,
                component_type: "Primitive::DecorativeBatch".to_string(),
                props: HashMap::new(),
                state: None,
                layout: LayoutRect {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                },
                children: Vec::new(),
                aria_role: "presentation".to_string(),
                aria_props: AriaProps::default(),
                portal_target: None,
                sdf_shape: None,
            };
            if let Some(parent_id) = self.stack.last()
                && let Some(parent) = self.nodes.get_mut(parent_id)
            {
                parent.children.push(id);
            }
            self.nodes.insert(id, batch_node);
        }
    }

    fn expand_batch_rect(&mut self, rect: cvkg_core::Rect) {
        if let Some(batch_id) = self.batch_node_id
            && let Some(node) = self.nodes.get_mut(&batch_id)
        {
            let new_left = node.layout.x.min(rect.x);
            let new_top = node.layout.y.min(rect.y);
            let new_right = (node.layout.x + node.layout.width).max(rect.x + rect.width);
            let new_bottom = (node.layout.y + node.layout.height).max(rect.y + rect.height);
            node.layout.x = new_left;
            node.layout.y = new_top;
            node.layout.width = new_right - new_left;
            node.layout.height = new_bottom - new_top;
        }
    }

    fn push_decorative_cmd(
        &mut self,
        cmd_type: &str,
        rect: cvkg_core::Rect,
        props: HashMap<String, serde_json::Value>,
    ) {
        self.expand_batch_rect(rect);
        self.decorative_batch.push(DecorativeCmd {
            cmd_type: cmd_type.to_string(),
            rect: LayoutRect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            },
            props,
        });
    }
}

impl cvkg_core::ElapsedTime for VNodeRenderer {
    fn delta_time(&self) -> f32 {
        0.0
    }

    fn elapsed_time(&self) -> f32 {
        0.0
    }
}

impl cvkg_core::Renderer for VNodeRenderer {
    fn fill_rect(&mut self, rect: cvkg_core::Rect, _color: [f32; 4]) {
        self.begin_decorative(rect);
        self.push_decorative_cmd("fill_rect", rect, HashMap::new());
    }

    fn shape_rich_text(
        &mut self,
        spans: &[cvkg_runic_text::TextSpan],
        max_width: Option<f32>,
        align: cvkg_runic_text::TextAlign,
        overflow: cvkg_runic_text::TextOverflow,
    ) -> Option<cvkg_runic_text::ShapedText> {
        self.text_engine
            .shape_layout(spans, max_width, align, overflow)
            .ok()
    }

    fn draw_shaped_text(&mut self, shaped: &cvkg_runic_text::ShapedText, x: f32, y: f32) {
        self.flush_decorative_batch();
        let id = self.next_id();
        let mut props = HashMap::new();
        let text = shaped
            .spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<&str>>()
            .join("");
        props.insert("text".to_string(), serde_json::Value::String(text.clone()));
        self.add_node(VNode {
            id,
            key: None,
            component_type: "Primitive::Text".to_string(),
            props,
            state: None,
            layout: LayoutRect {
                x,
                y,
                width: shaped.width,
                height: shaped.height,
            },
            children: Vec::new(),
            aria_role: "text".to_string(),
            aria_props: AriaProps {
                label: Some(text),
                ..Default::default()
            },
            portal_target: None,
            sdf_shape: None,
        });
    }

    fn push_vnode(&mut self, rect: cvkg_core::Rect, name: &'static str) {
        let id = self.next_id();
        let role = match name {
            "CornerButton" => "button",
            "BerserkerRoot" => "application",
            _ => "group",
        };

        self.add_node(VNode {
            id,
            key: None,
            component_type: name.to_string(),
            props: HashMap::new(),
            state: None,
            layout: LayoutRect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            },
            children: Vec::new(),
            aria_role: role.to_string(),
            aria_props: AriaProps::default(),
            portal_target: None,
            sdf_shape: None,
        });
        self.stack.push(id);
        self.flush_decorative_batch();
    }

    fn pop_vnode(&mut self) {
        self.stack.pop();
    }

    fn fill_rounded_rect(&mut self, rect: cvkg_core::Rect, radius: f32, _color: [f32; 4]) {
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert("radius".to_string(), serde_json::to_value(radius).unwrap());
        self.push_decorative_cmd("fill_rounded_rect", rect, props);
    }

    fn fill_ellipse(&mut self, rect: cvkg_core::Rect, _color: [f32; 4]) {
        self.begin_decorative(rect);
        self.push_decorative_cmd("fill_ellipse", rect, HashMap::new());
    }

    fn draw_3d_cube(&mut self, rect: cvkg_core::Rect, _color: [f32; 4], _rotation: [f32; 3]) {
        self.begin_decorative(rect);
        self.push_decorative_cmd("draw_3d_cube", rect, HashMap::new());
    }

    fn stroke_rect(&mut self, rect: cvkg_core::Rect, _color: [f32; 4], width: f32) {
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert("width".to_string(), serde_json::to_value(width).unwrap());
        self.push_decorative_cmd("stroke_rect", rect, props);
    }

    fn stroke_rounded_rect(
        &mut self,
        rect: cvkg_core::Rect,
        radius: f32,
        _color: [f32; 4],
        width: f32,
    ) {
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert("radius".to_string(), serde_json::to_value(radius).unwrap());
        props.insert("width".to_string(), serde_json::to_value(width).unwrap());
        self.push_decorative_cmd("stroke_rounded_rect", rect, props);
    }

    fn stroke_ellipse(&mut self, rect: cvkg_core::Rect, _color: [f32; 4], width: f32) {
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert("width".to_string(), serde_json::to_value(width).unwrap());
        self.push_decorative_cmd("stroke_ellipse", rect, props);
    }

    fn set_sdf_shape(&mut self, shape: cvkg_core::layout::SdfShape) {
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.sdf_shape = Some(shape);
        }
    }

    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: [f32; 4], width: f32) {
        let rect = cvkg_core::Rect {
            x: x1.min(x2),
            y: y1.min(y2),
            width: (x1 - x2).abs(),
            height: (y1 - y2).abs(),
        };
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert("x1".to_string(), serde_json::to_value(x1).unwrap());
        props.insert("y1".to_string(), serde_json::to_value(y1).unwrap());
        props.insert("x2".to_string(), serde_json::to_value(x2).unwrap());
        props.insert("y2".to_string(), serde_json::to_value(y2).unwrap());
        props.insert("color".to_string(), serde_json::to_value(color).unwrap());
        props.insert("width".to_string(), serde_json::to_value(width).unwrap());
        self.push_decorative_cmd("draw_line", rect, props);
    }

    fn draw_texture(&mut self, _id: u32, _rect: cvkg_core::Rect) {}

    fn draw_image(&mut self, name: &str, rect: cvkg_core::Rect) {
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert("src".to_string(), serde_json::to_value(name).unwrap());
        self.push_decorative_cmd("draw_image", rect, props);
    }

    fn load_image(&mut self, _name: &str, _data: &[u8]) {}
    fn push_clip_rect(&mut self, rect: cvkg_core::Rect) {
        self.clip_stack.push(rect);
    }
    fn pop_clip_rect(&mut self) {
        self.clip_stack.pop();
    }
    fn current_clip_rect(&self) -> cvkg_core::Rect {
        self.clip_stack
            .last()
            .copied()
            .unwrap_or(cvkg_core::Rect::new(-10000.0, -10000.0, 20000.0, 20000.0))
    }
    fn memoize(&mut self, _id: u64, _data_hash: u64, render_fn: &dyn Fn(&mut dyn Renderer)) {
        render_fn(self);
    }
    fn push_opacity(&mut self, _opacity: f32) {}
    fn pop_opacity(&mut self) {}
    fn bifrost(&mut self, _rect: cvkg_core::Rect, _blur: f32, _sat: f32, _op: f32) {}
    fn push_mjolnir_slice(&mut self, _angle: f32, _offset: f32) {}
    fn pop_mjolnir_slice(&mut self) {}

    fn mjolnir_shatter(&mut self, rect: cvkg_core::Rect, pieces: u32, force: f32, color: [f32; 4]) {
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert("pieces".to_string(), serde_json::to_value(pieces).unwrap());
        props.insert("force".to_string(), serde_json::to_value(force).unwrap());
        props.insert("color".to_string(), serde_json::to_value(color).unwrap());
        self.push_decorative_cmd("mjolnir_shatter", rect, props);
    }

    fn render_scene_node_3d(
        &mut self,
        position: [f32; 3],
        rotation: [f32; 4],
        scale: [f32; 3],
        color: [f32; 4],
        _meshes: &[cvkg_core::Mesh],
    ) {
        let rect = cvkg_core::Rect {
            x: position[0] - scale[0] / 2.0,
            y: position[1] - scale[1] / 2.0,
            width: scale[0],
            height: scale[1],
        };
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert(
            "position".to_string(),
            serde_json::to_value(position).unwrap(),
        );
        props.insert(
            "rotation".to_string(),
            serde_json::to_value(rotation).unwrap(),
        );
        props.insert("scale".to_string(), serde_json::to_value(scale).unwrap());
        props.insert("color".to_string(), serde_json::to_value(color).unwrap());
        self.push_decorative_cmd("render_scene_node_3d", rect, props);
    }

    fn draw_mjolnir_bolt(&mut self, _from: [f32; 2], _to: [f32; 2], _color: [f32; 4]) {}

    fn draw_linear_gradient(
        &mut self,
        rect: cvkg_core::Rect,
        start_color: [f32; 4],
        end_color: [f32; 4],
        angle: f32,
    ) {
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert(
            "start_color".to_string(),
            serde_json::to_value(start_color).unwrap(),
        );
        props.insert(
            "end_color".to_string(),
            serde_json::to_value(end_color).unwrap(),
        );
        props.insert("angle".to_string(), serde_json::to_value(angle).unwrap());
        self.push_decorative_cmd("draw_linear_gradient", rect, props);
    }

    fn draw_radial_gradient(
        &mut self,
        rect: cvkg_core::Rect,
        inner_color: [f32; 4],
        outer_color: [f32; 4],
    ) {
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert(
            "inner_color".to_string(),
            serde_json::to_value(inner_color).unwrap(),
        );
        props.insert(
            "outer_color".to_string(),
            serde_json::to_value(outer_color).unwrap(),
        );
        self.push_decorative_cmd("draw_radial_gradient", rect, props);
    }

    fn gungnir(&mut self, rect: cvkg_core::Rect, color: [f32; 4], radius: f32, intensity: f32) {
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert("radius".to_string(), serde_json::to_value(radius).unwrap());
        props.insert(
            "intensity".to_string(),
            serde_json::to_value(intensity).unwrap(),
        );
        props.insert("color".to_string(), serde_json::to_value(color).unwrap());
        self.push_decorative_cmd("gungnir", rect, props);
    }

    fn set_aria_role(&mut self, role: &str) {
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.aria_role = role.to_string();
        }
    }

    fn set_aria_label(&mut self, label: &str) {
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.aria_props.label = Some(label.to_string());
        }
    }

    fn set_aria_valuemin(&mut self, min: f32) {
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.aria_props.aria_valuemin = Some(min);
        }
    }

    fn set_aria_valuemax(&mut self, max: f32) {
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.aria_props.aria_valuemax = Some(max);
        }
    }

    fn set_aria_valuenow(&mut self, now: f32) {
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.aria_props.aria_valuenow = Some(now);
        }
    }

    fn register_shared_element(&mut self, _id: &str, _rect: cvkg_core::Rect) {}

    fn set_key(&mut self, key: &str) {
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.key = Some(key.to_string());
        }
    }

    fn register_handler(
        &mut self,
        event_type: &str,
        handler: std::sync::Arc<dyn Fn(cvkg_core::Event) + Send + Sync>,
    ) {
        self.flush_decorative_batch();
        if let Some(node_id) = self.stack.last() {
            log::trace!(
                "[VDOM] Registering handler '{}' on node {:?}",
                event_type,
                node_id
            );
            self.event_handlers
                .entry(*node_id)
                .or_default()
                .insert(event_type.to_string(), handler);
        }
    }
}

/// State management hook primitive for the Virtual DOM.
///
/// Returns a tuple containing:
/// 1. The current state value of type `T`.
/// 2. A setter `Arc<dyn Fn(T)>` to update the state.
pub fn use_state<T: Clone + Send + Sync + 'static>(
    id_hash: u64,
    default: T,
) -> (T, std::sync::Arc<dyn Fn(T) + Send + Sync>) {
    let current = {
        let s = cvkg_core::load_system_state();
        match s.get_component_state::<T>(id_hash) {
            Some(arc_val) => arc_val.read().unwrap_or_else(|e| e.into_inner()).clone(),
            None => {
                cvkg_core::update_system_state(|s| {
                    let mut s = s.clone();
                    s.set_component_state(id_hash, default.clone());
                    s
                });
                default.clone()
            }
        }
    };

    let setter = std::sync::Arc::new(move |new_val: T| {
        cvkg_core::update_system_state(move |s| {
            let mut s = s.clone();
            s.set_component_state(id_hash, new_val.clone());
            s
        });
    });

    (current, setter)
}
