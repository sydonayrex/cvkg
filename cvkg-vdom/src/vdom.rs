//! VDom — The root container for the Virtual DOM state.

use crate::accesskit_bridge::A11yNodeEntry;
use crate::diff::VDomPatch;
use crate::vnode::{NodeId, VNode};
use cvkg_core::Renderer;
use std::collections::HashMap;

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
    pub event_handlers: crate::vnode::NodeEventHandlerMap,
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
        let mut renderer = crate::vnode::VNodeRenderer::new();
        view.render(&mut renderer, rect);
        renderer.into_vdom()
    }

    /// Phase 4.4: Prepare this VDom to receive a new frame's nodes by clearing
    /// data while retaining allocated capacity in all HashMaps.
    pub fn clear_and_retain_capacity(&mut self) {
        self.root = None;
        self.nodes.clear();
        self.parents.clear();
        self.event_handlers.clear();
    }

    /// Apply a set of patches to the host's DOM environment.
    pub fn apply_to_dom(&self, patches: &[VDomPatch]) {
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
    pub fn validate_sync(&self, scene: &cvkg_scene::SceneGraph) -> Result<(), String> {
        let _span = tracing::info_span!("vdom_validate_sync").entered();
        match (self.root, scene.root) {
            (None, None) => return Ok(()),
            (Some(vr), Some(sr)) if vr.0 == sr.0 => {}
            _ => return Err("Root node mismatch".to_string()),
        }
        if self.nodes.len() != scene.nodes.len() {
            return Err(format!(
                "Node count mismatch: VDom({}) vs SceneGraph({})",
                self.nodes.len(),
                scene.nodes.len()
            ));
        }
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
                        if let Some(p) = props { node.props = p; }
                        if let Some(l) = layout { node.layout = l; }
                        if let Some(ap) = aria_props { node.aria_props = ap; }
                        if let Some(ar) = aria_role { node.aria_role = ar; }
                        if let Some(c) = children {
                            for child_id in &node.children { self.parents.remove(child_id); }
                            node.children = c;
                            for child_id in &node.children { self.parents.insert(*child_id, id); }
                        }
                        if let Some(h) = handlers { self.event_handlers.insert(id, h); }
                        if let Some(s) = sdf_shape { node.sdf_shape = Some(s); }
                    }
                }
                VDomPatch::Remove(id) => {
                    if let Some(node) = self.nodes.remove(&id) {
                        for child_id in &node.children { self.parents.remove(child_id); }
                    }
                    self.parents.remove(&id);
                }
                VDomPatch::Replace { id, node } => {
                    let is_root = self.root == Some(id);
                    let new_id = node.id;
                    if let Some(old_node) = self.nodes.get(&id) {
                        for child_id in &old_node.children { self.parents.remove(child_id); }
                    }
                    for child_id in &node.children { self.parents.insert(*child_id, new_id); }
                    self.nodes.remove(&id);
                    self.nodes.insert(new_id, node);
                    if is_root { self.root = Some(new_id); }
                    if let Ok(mut capture) = self.captured_node.lock() && *capture == Some(id) {
                        *capture = Some(new_id);
                    }
                    if let Ok(mut focus) = self.focused_node.lock() && *focus == Some(id) {
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
                VDomPatch::SetRoot(id) => { self.root = id; }
                VDomPatch::ClearHandlers { id } => { self.event_handlers.remove(&id); }
            }
        }
    }

    fn sdf_distance(
        shape: Option<&cvkg_core::layout::SdfShape>,
        layout: &crate::vnode::LayoutRect,
        x: f32,
        y: f32,
    ) -> f32 {
        let shape = shape.copied().unwrap_or(cvkg_core::layout::SdfShape::Rect(
            cvkg_core::layout::Rect { x: layout.x, y: layout.y, width: layout.width, height: layout.height }
        ));
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
    pub fn hit_test(&self, x: f32, y: f32, pointer_precision: f32) -> Option<(NodeId, f32)> {
        self.root.and_then(|root_id| self.hit_test_recursive(root_id, x, y, pointer_precision))
    }

    fn hit_test_recursive(&self, node_id: NodeId, x: f32, y: f32, pointer_precision: f32) -> Option<(NodeId, f32)> {
        let node = self.nodes.get(&node_id)?;
        let dist = Self::sdf_distance(node.sdf_shape.as_ref(), &node.layout, x, y);
        let proximity_limit = pointer_precision.max(0.0);
        let proximity = if dist <= 0.0 {
            1.0
        } else if proximity_limit > 0.0 {
            (1.0 - (dist / proximity_limit)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        if proximity > 0.0 {
            let mut best_child_hit: Option<(NodeId, f32)> = None;
            let mut children_to_test = node.children.clone();
            if y >= 28.0 {
                if let Some(pos) = children_to_test.iter().position(|&cid| {
                    self.nodes.get(&cid).map_or(false, |n| n.component_type == "DropdownOverlay")
                }) {
                    let overlay_id = children_to_test.remove(pos);
                    children_to_test.insert(0, overlay_id);
                }
            }
            for child_id in children_to_test.iter().rev() {
                if let Some((hit, hit_prox)) = self.hit_test_recursive(*child_id, x, y, pointer_precision) {
                    if hit_prox >= 1.0 { return Some((hit, hit_prox)); }
                    if best_child_hit.is_none() || hit_prox > best_child_hit.unwrap().1 {
                        best_child_hit = Some((hit, hit_prox));
                    }
                }
            }
            if let Some(bh) = best_child_hit { return Some(bh); }
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
        let captured_target = self.captured_node.lock().ok().and_then(|captured| *captured);
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
                    (captured_target, 1.0)
                } else {
                    let (id, proximity) = match self.hit_test(x, y, event.pointer_precision()) {
                        Some((i, p)) => (Some(i), p),
                        None => (None, 0.0),
                    };
                    (id, proximity)
                };
                if let cvkg_core::Event::PointerMove { ref mut proximity_field, .. } = event {
                    *proximity_field = proximity;
                }
                if let cvkg_core::Event::PointerDown { ref mut proximity_field, .. } = event {
                    *proximity_field = proximity;
                }
                if let cvkg_core::Event::PointerDown { .. } = event {
                    if let Ok(mut focus) = self.focused_node.lock() { *focus = id; }
                    if let Ok(mut capture) = self.captured_node.lock() { *capture = id; }
                }
                if let cvkg_core::Event::PointerUp { .. } = event
                    && let Ok(mut capture) = self.captured_node.lock()
                { *capture = None; }
                id
            }
            _ => {
                let (id, _) = match self.hit_test(0.0, 0.0, 0.0) {
                    Some((i, p)) => (Some(i), p),
                    None => (None, 0.0),
                };
                id
            }
        };
        if let Some(target) = target_id {
            if let Some(handlers) = self.event_handlers.get(&target) {
                if let Some(handler) = handlers.get(&event_name) {
                    return handler(event);
                }
            }
        }
        cvkg_core::EventResponse::Ignored
    }
}
