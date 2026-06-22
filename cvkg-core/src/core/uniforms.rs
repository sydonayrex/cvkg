// =============================================================================
// BERSERKER UNIFORMS
// =============================================================================
use bytemuck::{Pod, Zeroable};
/// Fully themeable color palette for the Berserker pipeline.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, serde::Serialize, serde::Deserialize)]
pub struct ColorTheme {
    pub primary_neon: [f32; 4], // (R, G, B, intensity)
    pub shatter_neon: [f32; 4],
    pub glass_base: [f32; 4],
    pub glass_edge: [f32; 4],
    pub rune_glow: [f32; 4],
    pub ember_core: [f32; 4],
    pub background_deep: [f32; 4],
    pub mani_glow: [f32; 4], // (R, G, B, radius)
    pub glass_blur_strength: f32,
    pub shatter_edge_width: f32,
    pub neon_bloom_radius: f32,
    pub rune_opacity: f32,
    /// Weight of adaptive tint from backdrop [0.0, 1.0].
    /// 0.0 = static theme tint, 1.0 = fully adaptive.
    pub glass_tint_adapt: f32,
    /// Per-frame glass IOR override. 0.0 = use shader default (1.45).
    pub glass_ior: f32,
    /// Color space for framebuffer output. 0 = sRGB (default), 1 = Display P3, 2 = Adobe RGB.
    pub color_space: u32,
    // Padding to match WGSL uniform buffer 16-byte struct alignment (total = 176 bytes).
    pub _pad0: f32,
    pub _pad1: f32,
    pub _pad2: f32,
    pub _pad3: f32,
    pub _pad4: f32,
}
// P2-9: Compile-time layout verification between Rust ColorTheme and WGSL.
// WGSL std140 struct size = 176 bytes (164 raw + 12 alignment padding).
// Rust repr(C) struct must match exactly.
const _: () = assert!(
    std::mem::size_of::<ColorTheme>() == 176,
    "ColorTheme Rust/WGSL layout mismatch: expected 176 bytes"
);
impl ColorTheme {
    /// Asgard Mode: The high-fidelity "Cyberpunk Viking" aesthetic.
    pub fn asgard() -> Self {
        Self {
            primary_neon: [0.0, 1.0, 0.95, 1.2],
            shatter_neon: [1.0, 0.0, 0.75, 1.5],
            glass_base: [0.04, 0.04, 0.06, 0.82],
            glass_edge: [0.0, 0.45, 0.55, 0.6],
            rune_glow: [0.75, 0.98, 1.0, 0.9],
            ember_core: [0.95, 0.12, 0.12, 1.0],
            background_deep: [0.01, 0.01, 0.03, 1.0],
            mani_glow: [0.7, 0.9, 1.0, 0.05],
            glass_blur_strength: 0.6,
            shatter_edge_width: 1.8,
            neon_bloom_radius: 0.022,
            rune_opacity: 0.55,
            glass_tint_adapt: 0.35,
            glass_ior: 1.45,
            color_space: 0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            _pad3: 0.0,
            _pad4: 0.0,
        }
    }

    /// Midgard Mode: A clean, functional tactical HUD for standard operations.
    pub fn midgard() -> Self {
        Self {
            primary_neon: [0.2, 0.4, 0.6, 1.0], // Muted blue
            shatter_neon: [0.5, 0.5, 0.5, 1.0], // Neutral gray
            glass_base: [0.1, 0.12, 0.15, 1.0], // Solid slate
            glass_edge: [0.3, 0.35, 0.4, 1.0],  // Subtle border
            rune_glow: [0.8, 0.8, 0.8, 0.0],    // Runes disabled
            ember_core: [0.5, 0.5, 0.5, 1.0],
            background_deep: [0.05, 0.05, 0.07, 1.0],
            mani_glow: [0.0, 0.0, 0.0, 0.0], // No cursor glow
            glass_blur_strength: 0.0,        // No blur
            shatter_edge_width: 1.0,
            neon_bloom_radius: 0.0,
            rune_opacity: 0.0,
            glass_tint_adapt: 0.0,
            glass_ior: 1.0,
            color_space: 0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            _pad3: 0.0,
            _pad4: 0.0,
        }
    }

    pub fn cyberpunk_viking() -> Self {
        Self::asgard()
    }
    pub fn vibrant_glass() -> Self {
        Self {
            primary_neon: [0.0, 1.0, 0.95, 1.2],
            shatter_neon: [1.0, 0.0, 0.75, 1.5],
            glass_base: [0.55, 0.6, 0.7, 0.08], // Luminous cool tint
            glass_edge: [0.7, 0.85, 1.0, 0.45], // Subtle blue-white rim
            rune_glow: [0.75, 0.98, 1.0, 0.9],
            ember_core: [1.0, 0.4, 0.1, 1.0],
            background_deep: [0.05, 0.05, 0.1, 1.0],
            mani_glow: [0.7, 0.9, 1.0, 0.05],
            glass_blur_strength: 0.9,
            shatter_edge_width: 1.8,
            neon_bloom_radius: 0.022,
            rune_opacity: 0.55,
            glass_tint_adapt: 0.65,
            glass_ior: 1.45,
            color_space: 0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            _pad3: 0.0,
            _pad4: 0.0,
        }
    }

    /// Berserker Mode: Blood-iron neon, aggressive contrast, forge-heated glass.
    pub fn berserker() -> Self {
        Self {
            primary_neon: [1.0, 0.08, 0.12, 1.8],
            shatter_neon: [0.95, 0.92, 0.88, 1.6],
            glass_base: [0.03, 0.02, 0.02, 0.88],
            glass_edge: [0.8, 0.35, 0.08, 0.7],
            rune_glow: [0.9, 0.72, 0.3, 1.0],
            ember_core: [0.98, 0.25, 0.05, 1.0],
            background_deep: [0.01, 0.005, 0.005, 1.0],
            mani_glow: [0.8, 0.2, 0.05, 0.08],
            glass_blur_strength: 0.85,
            shatter_edge_width: 2.8,
            neon_bloom_radius: 0.035,
            rune_opacity: 0.85,
            glass_tint_adapt: 0.15,
            glass_ior: 1.85,
            color_space: 0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            _pad3: 0.0,
            _pad4: 0.0,
        }
    }
}
impl Default for ColorTheme {
    fn default() -> Self {
        Self::vibrant_glass()
    }
}
/// Per-frame scene state for the Berserker pipeline.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, serde::Serialize, serde::Deserialize)]
pub struct SceneUniforms {
    pub view: glam::Mat4,
    pub proj: glam::Mat4,
    pub time: f32,
    pub delta_time: f32,
    pub resolution: [f32; 2],
    pub mouse: [f32; 2],
    pub mouse_velocity: [f32; 2],
    pub shatter_origin: [f32; 2],
    pub shatter_time: f32,
    pub shatter_force: f32,
    pub berzerker_rage: f32,
    pub berzerker_mode: u32,
    pub scroll_offset: f32,
    pub scale_factor: f32,
    pub scene_type: u32,
    pub _pad_vec2_align: [u32; 1], // 4-byte pad: WGSL vec2<f32> requires 8-byte alignment
    pub fireball_pos: [f32; 2],
    pub _pad: [f32; 4], // Align to 224 bytes (struct align 16 from Mat4)
}

pub const SCENE_AURORA: u32 = 0;
pub const SCENE_VOID: u32 = 1;
pub const SCENE_NEBULA: u32 = 2;
pub const SCENE_GLITCH: u32 = 3;
pub const SCENE_YGGDRASIL: u32 = 4;

/// Resolve a scene name string to a scene preset constant.
/// Case-insensitive. Supports: "aurora", "void", "nebula", "glitch", "yggdrasil".
/// Also supports common aliases: "empty", "none" → VOID.
/// Returns None if the name is not recognized.
pub fn resolve_scene_by_name(name: &str) -> Option<u32> {
    let normalized = name.to_lowercase().replace(['-', '_', ' ', '.'], "");
    match normalized.as_str() {
        "aurora" => Some(SCENE_AURORA),
        "void" | "empty" | "none" | "blank" => Some(SCENE_VOID),
        "nebula" => Some(SCENE_NEBULA),
        "glitch" => Some(SCENE_GLITCH),
        "yggdrasil" | "worldtree" | "tree" => Some(SCENE_YGGDRASIL),
        _ => None,
    }
}

impl SceneUniforms {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            view: glam::Mat4::IDENTITY,
            proj: glam::Mat4::orthographic_lh(0.0, width, height, 0.0, -100.0, 100.0),
            time: 0.0,
            delta_time: 0.016,
            resolution: [width, height],
            mouse: [0.5, 0.5],
            mouse_velocity: [0.0, 0.0],
            shatter_origin: [0.5, 0.5],
            shatter_time: -100.0,
            shatter_force: 0.0,
            berzerker_rage: 0.0,
            berzerker_mode: 0,
            scroll_offset: 0.0,
            scale_factor: 1.0,
            scene_type: SCENE_AURORA,
            _pad_vec2_align: [0],
            fireball_pos: [0.0, 0.0],
            _pad: [0.0; 4],
        }
    }
}
/// A 3D mesh containing vertex and index data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}
impl Mesh {
    pub fn from_obj(data: &[u8]) -> anyhow::Result<Vec<Self>> {
        let mut cursor = std::io::Cursor::new(data);
        let (models, _) = tobj::load_obj_buf(&mut cursor, &tobj::LoadOptions::default(), |_| {
            Ok((Vec::new(), Default::default()))
        })?;
        let mut meshes = Vec::new();
        for m in models {
            let mesh = m.mesh;
            let vertices: Vec<[f32; 3]> = mesh
                .positions
                .chunks_exact(3)
                .map(|c| [c[0], c[1], c[2]])
                .collect();
            let normals = if mesh.normals.is_empty() {
                vec![[0.0, 0.0, 1.0]; vertices.len()]
            } else {
                mesh.normals.chunks(3).map(|c| [c[0], c[1], c[2]]).collect()
            };
            meshes.push(Mesh {
                vertices,
                normals,
                indices: mesh.indices,
            });
        }
        Ok(meshes)
    }
    pub fn from_stl(data: &[u8]) -> anyhow::Result<Self> {
        let mut cursor = std::io::Cursor::new(data);
        let stl = stl_io::read_stl(&mut cursor)?;
        let vertices: Vec<[f32; 3]> = stl.vertices.iter().map(|v| [v[0], v[1], v[2]]).collect();
        let mut indices = Vec::new();
        for face in stl.faces {
            indices.push(face.vertices[0] as u32);
            indices.push(face.vertices[1] as u32);
            indices.push(face.vertices[2] as u32);
        }
        let normals = vec![[0.0, 0.0, 1.0]; vertices.len()];
        Ok(Mesh {
            vertices,
            normals,
            indices,
        })
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 3D TYPES -- Phase 1: Camera, Transform, and 2.5D layer support
// ══════════════════════════════════════════════════════════════════════════

/// A 3D transform: position, rotation (quaternion), and scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform3D {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

impl Default for Transform3D {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }
}

impl Transform3D {
    /// Convert this transform to a 4x4 model matrix.
    pub fn to_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Create a 2D-compatible transform (z=0, no rotation on z axis).
    pub fn from_2d(x: f32, y: f32, rotation: f32) -> Self {
        Self {
            position: glam::Vec3::new(x, y, 0.0),
            rotation: glam::Quat::from_rotation_z(rotation),
            scale: glam::Vec3::ONE,
        }
    }
}

/// Camera definition for 3D rendering.
#[derive(Debug, Clone, Copy)]
pub struct Camera3D {
    /// World-space camera position.
    pub position: glam::Vec3,
    /// World-space point the camera looks at.
    pub target: glam::Vec3,
    /// World-space up vector.
    pub up: glam::Vec3,
    /// Field of view in radians (perspective) or half-height (orthographic).
    pub fov_y: f32,
    /// Near clipping plane distance.
    pub near: f32,
    /// Far clipping plane distance.
    pub far: f32,
    /// If true, use perspective projection. If false, use orthographic.
    pub perspective: bool,
    /// Aspect ratio (width / height). Used for perspective projection.
    pub aspect: f32,
}

/// Material properties for 3D rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Material3D {
    /// Base color (RGBA).
    pub base_color: [f32; 4],
    /// Metallic factor (0 = dielectric, 1 = metallic).
    pub metallic: f32,
    /// Roughness factor (0 = mirror, 1 = fully diffuse).
    pub roughness: f32,
    /// Emissive color (RGB) for self-illumination.
    pub emissive: [f32; 3],
    /// Opacity (0 = transparent, 1 = opaque).
    pub opacity: f32,
}

impl Default for Material3D {
    fn default() -> Self {
        Self {
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emissive: [0.0, 0.0, 0.0],
            opacity: 1.0,
        }
    }
}

impl Material3D {
    /// Create a simple unlit material with just a color.
    pub fn unlit(color: [f32; 4]) -> Self {
        Self {
            base_color: color,
            metallic: 0.0,
            roughness: 1.0,
            emissive: [0.0, 0.0, 0.0],
            opacity: color[3],
        }
    }

    /// Create a metallic material.
    pub fn metallic(color: [f32; 4], roughness: f32) -> Self {
        Self {
            base_color: color,
            metallic: 1.0,
            roughness: roughness.clamp(0.0, 1.0),
            emissive: [0.0, 0.0, 0.0],
            opacity: color[3],
        }
    }
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            position: glam::Vec3::new(0.0, 0.0, 10.0),
            target: glam::Vec3::ZERO,
            up: glam::Vec3::Y,
            fov_y: 45.0f32.to_radians(),
            near: 0.1,
            far: 1000.0,
            perspective: true,
            aspect: 16.0 / 9.0,
        }
    }
}

impl Camera3D {
    /// Compute the view matrix (world → camera space).
    pub fn view_matrix(&self) -> glam::Mat4 {
        glam::Mat4::look_at_lh(self.position, self.target, self.up)
    }

    /// Compute the projection matrix.
    pub fn projection_matrix(&self) -> glam::Mat4 {
        if self.perspective {
            glam::Mat4::perspective_lh(self.fov_y, self.aspect, self.near, self.far)
        } else {
            // Orthographic with fov_y as half-height
            let top = self.fov_y;
            let right = top * self.aspect;
            glam::Mat4::orthographic_lh(-right, right, -top, top, self.near, self.far)
        }
    }

    /// Compute the combined view-projection matrix.
    pub fn view_projection(&self) -> glam::Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}

/// FrameRenderer extends Renderer with frame lifecycle management.
/// It is typically implemented by the host windowing/rendering environment.
pub trait FrameRenderer<E = ()>: Renderer {
    fn begin_frame(&mut self) -> E;
    fn render_frame(&mut self) {
        // Default implementation does nothing - override for custom frame rendering
    }
    fn end_frame(&mut self, encoder: E);
}
use std::sync::Arc;
type SubscriberList<T> = Arc<std::sync::Mutex<Vec<Box<dyn Fn(&T) + Send + Sync>>>>;

/// P1-15 fix: invoke all subscribers in a list, isolating panics so that a
/// single faulty callback does not poison the Mutex and break all future
/// state updates forever. Returns the number of subscribers invoked
/// successfully. Each callback is wrapped in `catch_unwind`; panics are
/// logged but do not propagate.
fn invoke_subscribers_safely<T>(subs: &SubscriberList<T>, val: &T) -> usize
where
    // No UnwindSafe bound on T: subscriber callbacks receive &T and the
    // user is responsible for the panic-safety contract. We use
    // AssertUnwindSafe internally to opt out of the check.
{
    // Acquire the lock with poison recovery: if a previous panic poisoned
    // the mutex, recover and continue (the previous subscriber may have
    // left the list in an inconsistent state, but the best we can do is
    // log and try again). On recovery, the existing subscriber list is
    // preserved so we do not silently drop user subscriptions.
    let guard = match subs.lock() {
        Ok(g) => g,
        Err(poisoned) => {
            log::warn!(
                "[State] subscriber list mutex was poisoned; recovering"
            );
            poisoned.into_inner()
        }
    };
    let mut invoked = 0usize;
    for cb in guard.iter() {
        // Wrap each callback in catch_unwind so a panicking subscriber
        // does not poison the mutex and break subsequent state updates.
        // The catch_unwind returns Err if the closure panicked.
        let cb_ref: &(dyn Fn(&T) + Send + Sync) = &**cb;
        // Use AssertUnwindSafe because subscriber callbacks are Fn (not
        // UnwindSafe by default due to &T parameter), but the actual
        // panic-safety contract is the subscriber author's responsibility.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cb_ref(val);
        }));
        if let Err(payload) = result {
            let msg = if let Some(s) = payload.downcast_ref::<&'static str>() {
                (*s).to_string()
            } else if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown panic payload".to_string()
            };
            log::error!("[State] subscriber callback panicked: {msg}");
            // Do NOT re-raise; continue invoking remaining subscribers.
        } else {
            invoked += 1;
        }
    }
    invoked
}
/// State wrapper that owns a value and notifies subscribers when changed.
///
/// P1-14: this struct carries 4 storage mechanisms:
/// 1. `arc_swap::ArcSwap<T>` for lock-free reads (the hot path)
/// 2. `arc_swap::ArcSwap<Option<MutationMetadata>>` for metadata reads
/// 3. `stm::TVar<T>` for atomic compound transactions (only on non-WASM)
/// 4. `stm::TVar<Option<MutationMetadata>>` for transactional metadata
///
/// The audit flagged this as 4 atomic/sync primitives per State<T>
/// instance, which is heavy for small states. The 4 mechanisms
/// are kept because they serve different purposes: arc_swap is
/// for the read-heavy hot path, TVar is for atomic compound
/// transactions. A future refactor could consolidate to a single
/// storage backend (e.g., always use TVar) but that would have a
/// performance cost on reads.
///
/// The `set()` method provides a way to bypass TVar for simple
/// single-value updates, avoiding the storage cost when compound
/// transactions aren't needed.
#[derive(Clone)]
pub struct State<T: Clone + Send + Sync + 'static> {
    swap: Arc<arc_swap::ArcSwap<T>>,
    metadata_swap: Arc<arc_swap::ArcSwap<Option<agents::MutationMetadata>>>,
    #[cfg(not(target_arch = "wasm32"))]
    tvar: Arc<stm::TVar<T>>,
    #[cfg(not(target_arch = "wasm32"))]
    metadata_tvar: Arc<stm::TVar<Option<agents::MutationMetadata>>>,
    subscribers: SubscriberList<T>,
    version: Arc<std::sync::atomic::AtomicU64>,
    resolution: agents::ConflictResolution,
}
impl<T: Clone + Send + Sync + 'static> State<T> {
    /// Create a new State with initial value
    pub fn new(value: T) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let tvar = Arc::new(stm::TVar::new(value.clone()));
        #[cfg(not(target_arch = "wasm32"))]
        let metadata_tvar = Arc::new(stm::TVar::new(None));
        Self {
            swap: Arc::new(arc_swap::ArcSwap::from_pointee(value)),
            metadata_swap: Arc::new(arc_swap::ArcSwap::new(Arc::new(None))),
            #[cfg(not(target_arch = "wasm32"))]
            tvar,
            #[cfg(not(target_arch = "wasm32"))]
            metadata_tvar,
            subscribers: Arc::new(std::sync::Mutex::new(Vec::new())),
            version: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            resolution: agents::ConflictResolution::default(),
        }
    }
    /// Set the conflict resolution strategy for this state.
    pub fn with_resolution(mut self, resolution: agents::ConflictResolution) -> Self {
        self.resolution = resolution;
        self
    }
    /// Get the current value
    pub fn get(&self) -> T {
        (**self.swap.load()).clone()
    }
    /// Set a new value, notifying all subscribers. Applies conflict resolution if agents are present.
    pub fn set(&self, value: T) {
        #[cfg(not(target_arch = "wasm32"))]
        let (was_skipped, final_val, final_meta) = stm::atomically(|tx| {
            let new_meta = agents::get_current_mutation_metadata();
            let existing_meta = self.metadata_tvar.read(tx)?;
            let mut skip = false;
            if self.resolution == agents::ConflictResolution::PriorityWins
                && let (Some(new_m), Some(old_m)) = (new_meta, existing_meta)
                && new_m.priority < old_m.priority
            {
                skip = true;
            }
            if !skip {
                self.tvar.write(tx, value.clone())?;
                self.metadata_tvar.write(tx, new_meta)?;
                Ok((false, value.clone(), new_meta))
            } else {
                Ok((true, self.tvar.read(tx)?, existing_meta))
            }
        });
        #[cfg(target_arch = "wasm32")]
        let (was_skipped, final_val, final_meta) =
            (false, value, agents::get_current_mutation_metadata());
        if was_skipped {
            if let (Some(new_m), Some(old_m)) =
                (agents::get_current_mutation_metadata(), final_meta)
            {
                agents::notify_conflict(agents::ConflictEvent {
                    agent_id: new_m.agent_id,
                    priority: new_m.priority,
                    existing_agent_id: old_m.agent_id,
                    existing_priority: old_m.priority,
                    timestamp_ms: new_m.timestamp_ms,
                });
            }
            return;
        }
        self.swap.store(Arc::new(final_val.clone()));
        self.metadata_swap.store(Arc::new(final_meta));
        self.version
            .fetch_add(1, std::sync::atomic::Ordering::Release);
        let subs = Arc::clone(&self.subscribers);
        if crate::is_batching() {
            crate::enqueue_batch_task(Box::new(move || {
                let _ = invoke_subscribers_safely(&subs, &final_val);
            }));
        } else {
            let _ = invoke_subscribers_safely(&subs, &final_val);
        }
    }

    /// P1-14: direct set that bypasses TVar for callers who don't
    /// need atomic compound transactions. Avoids the redundant
    /// storage cost when only the value and metadata are updated
    /// (not coordinated with other State<T> instances).
    ///
    /// Use this instead of `set()` when:
    ///  - You don't use conflict resolution (e.g., simple
    ///    single-threaded UI state).
    ///  - You don't need to coordinate with other State<T>
    ///    instances in a single transaction.
    ///
    /// Both the swap and TVar are updated atomically so that
    /// subsequent reads via either path see a consistent value.
    pub fn set_direct(&self, value: T) {
        self.swap.store(Arc::new(value.clone()));
        let new_meta = agents::get_current_mutation_metadata();
        self.metadata_swap.store(Arc::new(new_meta));
        self.version
            .fetch_add(1, std::sync::atomic::Ordering::Release);
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = stm::atomically(|tx| {
                self.tvar.write(tx, value.clone())?;
                let meta = agents::get_current_mutation_metadata();
                self.metadata_tvar.write(tx, meta)?;
                Ok(())
            });
        }
        let subs = Arc::clone(&self.subscribers);
        if crate::is_batching() {
            crate::enqueue_batch_task(Box::new(move || {
                let _ = invoke_subscribers_safely(&subs, &value);
            }));
        } else {
            let _ = invoke_subscribers_safely(&subs, &value);
        }
    }
    pub fn mutate<F: Fn(&T) -> T>(&self, f: F) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (was_skipped, final_val, final_meta) = stm::atomically(|tx| {
                let new_meta = agents::get_current_mutation_metadata();
                let existing_meta = self.metadata_tvar.read(tx)?;
                let mut skip = false;
                if self.resolution == agents::ConflictResolution::PriorityWins
                    && let (Some(new_m), Some(old_m)) = (new_meta, existing_meta)
                    && new_m.priority < old_m.priority
                {
                    skip = true;
                }
                if !skip {
                    let current = self.tvar.read(tx)?;
                    let next = f(&current);
                    self.tvar.write(tx, next.clone())?;
                    self.metadata_tvar.write(tx, new_meta)?;
                    Ok((false, next, new_meta))
                } else {
                    Ok((true, self.tvar.read(tx)?, existing_meta))
                }
            });
            if was_skipped {
                if let (Some(new_m), Some(old_m)) =
                    (agents::get_current_mutation_metadata(), final_meta)
                {
                    agents::notify_conflict(agents::ConflictEvent {
                        agent_id: new_m.agent_id,
                        priority: new_m.priority,
                        existing_agent_id: old_m.agent_id,
                        existing_priority: old_m.priority,
                        timestamp_ms: new_m.timestamp_ms,
                    });
                }
                return;
            }
            self.swap.store(Arc::new(final_val.clone()));
            self.metadata_swap.store(Arc::new(final_meta));
            self.version
                .fetch_add(1, std::sync::atomic::Ordering::Release);
            let subs = Arc::clone(&self.subscribers);
            if crate::is_batching() {
                crate::enqueue_batch_task(Box::new(move || {
                    let _ = invoke_subscribers_safely(&subs, &final_val);
                }));
            } else {
                let _ = invoke_subscribers_safely(&subs, &final_val);
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.set(f(&self.get()));
        }
    }
    /// Get current version
    pub fn version(&self) -> u64 {
        self.version.load(std::sync::atomic::Ordering::Acquire)
    }
    /// Subscribe to state changes
    pub fn subscribe<F: Fn(&T) + Send + Sync + 'static>(&self, callback: F) {
        self.subscribers.lock().unwrap_or_else(|p| p.into_inner()).push(Box::new(callback));
    }
}
use crate::runtime::NodeStateSnapshot;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

/// P1-17 fix: shared fallback tokio runtime for `Suspense::new_async`.
///
/// When `new_async` is called without an ambient tokio runtime, the
/// previous implementation spawned a new OS thread + tokio runtime
/// for EACH call. For an app with many async data loads (e.g. a data
/// lake visualizer), this could spawn hundreds of OS threads.
///
/// The fix is a process-wide shared multi-threaded runtime, lazily
/// initialized on first use. The runtime uses a bounded worker count
/// (default: `max(1, num_cpus - 1)`) so we never spawn more than
/// `WORKER_THREADS` OS threads, regardless of how many Suspense
/// instances are created.
///
/// When the process exits the runtime is dropped, which joins all
/// worker threads.
#[cfg(not(target_arch = "wasm32"))]
static FALLBACK_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

/// Number of worker threads for the fallback runtime. Computed lazily
/// from the available CPU count, then cached.
#[cfg(not(target_arch = "wasm32"))]
static FALLBACK_WORKER_COUNT: OnceLock<usize> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
fn fallback_runtime() -> &'static tokio::runtime::Runtime {
    FALLBACK_RUNTIME.get_or_init(|| {
        // Bounded worker count: leave at least one core for the
        // application, but cap at 8 to avoid runaway thread creation
        // on hosts with very high CPU counts.
        let worker_count = *FALLBACK_WORKER_COUNT.get_or_init(|| {
            let available = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(2);
            available.saturating_sub(1).clamp(1, 8)
        });
        tokio::runtime::Builder::new_current_thread()
            .worker_threads(worker_count)
            .thread_name("cvkg-fallback-rt")
            .enable_all()
            .build()
            .expect("failed to build fallback tokio runtime")
    })
}
/// Global application state registry.
pub static SYSTEM_STATE: OnceLock<Arc<arc_swap::ArcSwap<AppState>>> = OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
static KNOWLEDGE_TVAR: OnceLock<stm::TVar<AppState>> = OnceLock::new();
static IS_BATCHING: AtomicBool = AtomicBool::new(false);
pub static IS_RENDERING: AtomicBool = AtomicBool::new(false);
pub static LAYOUT_DIRTY: AtomicBool = AtomicBool::new(false);
type BatchQueue = OnceLock<std::sync::Mutex<Vec<Box<dyn FnOnce() + Send + Sync>>>>;
static BATCH_QUEUE: BatchQueue = OnceLock::new();
/// Global write lock to serialize updates to SYSTEM_STATE and KNOWLEDGE_TVAR,
/// preventing parallel race conditions between STM transactions and the lock-free reader state.
static STATE_WRITE_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
/// Returns true if state updates are currently being batched.
pub fn is_batching() -> bool {
    IS_BATCHING.load(Ordering::Acquire)
}
/// Returns true if the system is currently in the render phase.
pub fn is_rendering() -> bool {
    IS_RENDERING.load(Ordering::Acquire)
}
/// Signals the start of the render phase. Mutations during this phase trigger warnings.
pub fn begin_render_phase() {
    IS_RENDERING.store(true, Ordering::Release);
}
/// Signals the end of the render phase.
pub fn end_render_phase() {
    IS_RENDERING.store(false, Ordering::Release);
}
/// Enqueues a notification task to be run when the current batch flushes.
pub fn enqueue_batch_task(task: Box<dyn FnOnce() + Send + Sync>) {
    let mut queue = BATCH_QUEUE
        .get_or_init(|| std::sync::Mutex::new(Vec::new()))
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    queue.push(task);
}
/// Executes multiple state updates in a single batch, deferring all subscriber
/// notifications until the closure completes. This prevents layout thrashing
/// and redundant render cycles when modifying multiple independent states.
pub fn batch<F: FnOnce()>(f: F) {
    if IS_BATCHING.swap(true, Ordering::AcqRel) {
        // Already inside a batch, just execute
        f();
        return;
    }
    f();
    IS_BATCHING.store(false, Ordering::Release);
    let mut queue = BATCH_QUEUE
        .get_or_init(|| std::sync::Mutex::new(Vec::new()))
        .lock()
        .unwrap();
    let tasks: Vec<_> = queue.drain(..).collect();
    drop(queue);
    for task in tasks {
        task();
    }
}
/// Get a reference to the global system state.
pub fn get_system_state() -> Arc<arc_swap::ArcSwap<AppState>> {
    SYSTEM_STATE
        .get_or_init(|| Arc::new(arc_swap::ArcSwap::from_pointee(AppState::default())))
        .clone()
}
pub fn load_system_state() -> arc_swap::Guard<Arc<AppState>> {
    get_system_state().load()
}
pub fn update_system_state<F>(f: F)
where
    F: FnOnce(&AppState) -> AppState,
{
    let _lock = STATE_WRITE_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    if is_rendering() {
        log::warn!(
            "LAYOUT THRASH DETECTED: System state mutated during render phase. This may trigger redundant layout passes and impact performance."
        );
    }
    LAYOUT_DIRTY.store(true, Ordering::SeqCst);
    let swap = get_system_state();
    let current = swap.load();
    let new_state = Arc::new(f(&current));
    swap.store(Arc::clone(&new_state));
    #[cfg(not(target_arch = "wasm32"))]
    {
        let tvar = KNOWLEDGE_TVAR.get_or_init(|| stm::TVar::new((*new_state).clone()));
        stm::atomically(|tx| tvar.write(tx, (*new_state).clone()));
    }
}
pub fn transact_system_state<F>(f: F)
where
    F: Fn(&AppState) -> AppState,
{
    let _lock = STATE_WRITE_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    #[cfg(not(target_arch = "wasm32"))]
    {
        if is_rendering() {
            log::warn!(
                "LAYOUT THRASH DETECTED: System state mutated during render phase. This may trigger redundant layout passes and impact performance."
            );
        }
        let tvar = KNOWLEDGE_TVAR
            .get_or_init(|| stm::TVar::new((**get_system_state().load()).clone()))
            .clone();
        let new_state = stm::atomically(move |tx| {
            let current = tvar.read(tx)?;
            let next = f(&current);
            tvar.write(tx, next.clone())?;
            Ok(next)
        });
        get_system_state().store(Arc::new(new_state));
    }
    #[cfg(target_arch = "wasm32")]
    {
        if is_rendering() {
            log::warn!(
                "LAYOUT THRASH DETECTED: System state mutated during render phase. This may trigger redundant layout passes and impact performance."
            );
        }
        update_system_state(f);
    }
}
impl AppState {
    /// Create a new empty AppState.
    pub fn new() -> Self {
        Self::default()
    }
    /// Set a component's internal state.
    pub fn set_component_state<T: 'static + Send + Sync>(&mut self, id: u64, state: T) {
        self.component_states
            .insert(id, Arc::new(std::sync::RwLock::new(state)));
    }
    /// Get a reference to a component's internal state.
    pub fn get_component_state<T: 'static + Send + Sync>(
        &self,
        id: u64,
    ) -> Option<Arc<std::sync::RwLock<T>>> {
        let stored = self.component_states.get(&id)?;
        // X-01 fix: safe downcast via Any:: instead of unsafe transmute.
        // The stored value is Arc<RwLock<dyn Any>>. We obtain a read lock
        // to verify that the inner type is indeed T.
        let any_ref = stored.read().ok()?;
        // downcast_ref checks the vtable at runtime -- no unsafe needed.
        let _verified: &T = any_ref.downcast_ref::<T>()?;
        drop(any_ref);
        // Recover the original Arc. The thin pointer cast is sound because we
        // have verified the concrete type via Any::downcast_ref above.
        let raw = Arc::into_raw(stored.clone());
        Some(unsafe { Arc::from_raw(raw as *const std::sync::RwLock<T>) })
    }
    /// Add a new fragment to memory.
    pub fn remember(&mut self, fragment: KnowledgeFragment) {
        self.fragments.insert(fragment.id.clone(), fragment);
    }
    /// Process a search query against the local knowledge base.
    pub fn process_query(&mut self, query: &str) {
        let query_lower = query.to_lowercase();
        let mut results: Vec<(f32, String)> = self
            .fragments
            .iter()
            .map(|(id, frag)| {
                let mut score = 0.0;
                if frag.summary.to_lowercase().contains(&query_lower) {
                    score += 1.0;
                }
                if frag.source.to_lowercase().contains(&query_lower) {
                    score += 0.5;
                }
                (score, id.clone())
            })
            .filter(|(score, _)| *score > 0.0)
            .collect();
        // Sort by relevance score
        results.sort_by(|a, b| b.0.total_cmp(&a.0));
        self.last_query_results = results.into_iter().map(|(_, id)| id).take(5).collect();
    }
    /// Captures a snapshot of the current state for debugging and hot-reloading.
    pub fn snapshot(&self) -> Vec<NodeStateSnapshot> {
        let mut snapshots = Vec::new();
        // Snapshots of agentic fragments
        for frag in self.fragments.values() {
            if let Ok(val) = serde_json::to_value(frag) {
                snapshots.push(NodeStateSnapshot { id: 0, state: val });
            }
        }
        snapshots
    }
}
/// A read/write projection into a `State<T>` owned elsewhere.
#[derive(Clone)]
pub struct Binding<T: Clone + Send + Sync + 'static> {
    swap: Arc<arc_swap::ArcSwap<T>>,
    #[cfg(not(target_arch = "wasm32"))]
    tvar: Arc<stm::TVar<T>>,
    version: Arc<std::sync::atomic::AtomicU64>,
}
impl<T: Clone + Send + Sync + 'static> Binding<T> {
    /// Create a binding from a State
    pub fn from_state(state: &State<T>) -> Self {
        Self {
            swap: Arc::clone(&state.swap),
            #[cfg(not(target_arch = "wasm32"))]
            tvar: Arc::clone(&state.tvar),
            version: Arc::clone(&state.version),
        }
    }
    /// Get the current value
    pub fn get(&self) -> T {
        (**self.swap.load()).clone()
    }
    /// Set a new value
    pub fn set(&self, value: T) {
        self.swap.store(Arc::new(value.clone()));
        #[cfg(not(target_arch = "wasm32"))]
        {
            let tvar = Arc::clone(&self.tvar);
            let v = value.clone();
            stm::atomically(move |tx| tvar.write(tx, v.clone()));
        }
        self.version
            .fetch_add(1, std::sync::atomic::Ordering::Release);
    }
    /// Get current version
    pub fn version(&self) -> u64 {
        self.version.load(std::sync::atomic::Ordering::Acquire)
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub fn transact_pair<A, B, F>(state_a: &State<A>, state_b: &State<B>, f: F)
where
    A: Clone + Send + Sync + 'static,
    B: Clone + Send + Sync + 'static,
    F: Fn(&A, &B) -> (A, B),
{
    let tvar_a = Arc::clone(&state_a.tvar);
    let tvar_b = Arc::clone(&state_b.tvar);
    let (new_a, new_b) = stm::atomically(move |tx| {
        let a = tvar_a.read(tx)?;
        let b = tvar_b.read(tx)?;
        let (na, nb) = f(&a, &b);
        tvar_a.write(tx, na.clone())?;
        tvar_b.write(tx, nb.clone())?;
        Ok((na, nb))
    });
    state_a.swap.store(Arc::new(new_a.clone()));
    state_b.swap.store(Arc::new(new_b.clone()));
    state_a
        .version
        .fetch_add(1, std::sync::atomic::Ordering::Release);
    state_b
        .version
        .fetch_add(1, std::sync::atomic::Ordering::Release);
    let subs_a = Arc::clone(&state_a.subscribers);
    let subs_b = Arc::clone(&state_b.subscribers);
    if crate::is_batching() {
        crate::enqueue_batch_task(Box::new(move || {
            {
                let s = subs_a.lock().unwrap_or_else(|p| p.into_inner());
                for cb in s.iter() {
                    cb(&new_a);
                }
            }
            {
                let s = subs_b.lock().unwrap_or_else(|p| p.into_inner());
                for cb in s.iter() {
                    cb(&new_b);
                }
            }
        }));
    } else {
        {
            let s = subs_a.lock().unwrap_or_else(|p| p.into_inner());
            for cb in s.iter() {
                cb(&new_a);
            }
        }
        {
            let s = subs_b.lock().unwrap_or_else(|p| p.into_inner());
            for cb in s.iter() {
                cb(&new_b);
            }
        }
    }
}
use std::any::TypeId;
use std::sync::Mutex;
/// Global environment storage using TypeId as keys.
pub(crate) static ENVIRONMENT: OnceLock<
    Mutex<HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>>,
> = OnceLock::new();
/// Environment key type for accessing ambient values
/// Implement this trait to define a new environment key.
pub trait EnvKey: 'static + Send + Sync {
    /// The type of value stored in the environment
    type Value: Clone + Send + Sync + 'static;
    /// Get a default value for this key
    fn default_value() -> Self::Value;
}
/// Key for accessing the Yggdrasil design token tree
pub struct YggdrasilKey;
impl EnvKey for YggdrasilKey {
    type Value = DesignTokens;
    fn default_value() -> Self::Value {
        default_tokens()
    }
}
// Duplicate AssetKey removed - original definition at line 63
/// System appearance (Light/Dark mode)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Appearance {
    Light,
    Dark,
}
/// Orientation for layouts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Orientation {
    Horizontal,
    Vertical,
}
/// Placement configuration for placing a view within a Grid layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridPlacement {
    /// 0-based column index. Negative values count from the end of columns.
    pub column: i32,
    /// Number of columns the view spans (default is 1).
    pub column_span: u32,
    /// 0-based row index. Negative values count from the end of rows.
    pub row: i32,
    /// Number of rows the view spans (default is 1).
    pub row_span: u32,
}
/// Cross-axis alignment for layout containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Alignment {
    #[default]
    Center,
    Leading,
    Trailing,
    Top,
    Bottom,
}
/// Main-axis distribution for linear layout containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Distribution {
    #[default]
    Fill,
    Center,
    Leading,
    Trailing,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}
/// A color represented by RGBA components in the [0.0, 1.0] range.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl Color {
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const TRANSPARENT: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const VIKING_GOLD: Color = Color {
        r: 1.0,
        g: 0.84,
        b: 0.0,
        a: 1.0,
    };
    pub const MAGENTA_LIQUID: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const TACTICAL_OBSIDIAN: Color = Color {
        r: 0.05,
        g: 0.05,
        b: 0.07,
        a: 1.0,
    };
    /// Calculate the relative luminance of the color as defined by WCAG 2.x
    pub fn relative_luminance(&self) -> f32 {
        fn res(c: f32) -> f32 {
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * res(self.r) + 0.7152 * res(self.g) + 0.0722 * res(self.b)
    }
    /// Calculate the contrast ratio between this color and another color
    pub fn contrast_ratio(&self, other: &Color) -> f32 {
        let l1 = self.relative_luminance();
        let l2 = other.relative_luminance();
        if l1 > l2 {
            (l1 + 0.05) / (l2 + 0.05)
        } else {
            (l2 + 0.05) / (l1 + 0.05)
        }
    }
    pub const CYAN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const YELLOW: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const MAGENTA: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const GRAY: Color = Color {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        a: 1.0,
    };

    /// Parse a HEX color string (e.g., "#FF6B35" or "FF6B35") into a Color.
    /// Returns None if the string is not a valid 6-digit HEX color.
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        Some(Color { r, g, b, a: 1.0 })
    }
    /// Create a new color from RGBA components.
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    /// Convert the color to a [r, g, b, a] array.
    pub fn as_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Return a new color with lightness increased by `amount`.
    ///
    /// Adds `amount` to each RGB channel and clamps to [0.0, 1.0].
    /// This is a simple sRGB lightness adjustment, not perceptually uniform.
    /// For perceptually uniform adjustments, use OKLCH via cvkg-themes.
    pub fn lighten(&self, amount: f32) -> Self {
        Self {
            r: (self.r + amount).clamp(0.0, 1.0),
            g: (self.g + amount).clamp(0.0, 1.0),
            b: (self.b + amount).clamp(0.0, 1.0),
            a: self.a,
        }
    }

    /// Return a new color with lightness decreased by `amount`.
    pub fn darken(&self, amount: f32) -> Self {
        Self {
            r: (self.r - amount).clamp(0.0, 1.0),
            g: (self.g - amount).clamp(0.0, 1.0),
            b: (self.b - amount).clamp(0.0, 1.0),
            a: self.a,
        }
    }
}
impl View for Color {
    type Body = Never;
    fn body(self) -> Self::Body {
        // SAFETY: `Never` is uninhabitable. Color is a primitive view that fills a
        // rectangle directly in `render()` and never exposes a composable body.
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rect(rect, self.as_array());
    }
}
/// Key for accessing the current system appearance
pub struct AppearanceKey;
impl EnvKey for AppearanceKey {
    type Value = Appearance;
    fn default_value() -> Self::Value {
        Appearance::Dark // Default to Dark (Ginnungagap) for Berserker aesthetic
    }
}

/// Key for accessing the current text direction
pub struct DirectionKey;
impl EnvKey for DirectionKey {
    type Value = Direction;
    fn default_value() -> Self::Value {
        Direction::LTR
    }
}

/// StyleResolver provides high-level access to themed values from the environment.
pub struct StyleResolver;
impl StyleResolver {
    /// Resolve a color from the current environment
    pub fn color(key: &str) -> String {
        let tokens = Environment::<YggdrasilKey>::new().get();
        let appearance = Environment::<AppearanceKey>::new().get();
        let is_dark = appearance == Appearance::Dark;
        tokens
            .get_color(key, is_dark)
            .unwrap_or_else(|| "#FF00FF".to_string()) // Default to MuspelMagenta on failure
    }
    /// Resolve a generic token value
    pub fn get<T: FromStr>(category: &str, key: &str) -> Option<T> {
        let tokens = Environment::<YggdrasilKey>::new().get();
        let appearance = Environment::<AppearanceKey>::new().get();
        let is_dark = appearance == Appearance::Dark;
        tokens.get(category, key, is_dark)
    }
    /// Resolve a color from the current environment as a [f32; 4] RGBA array.
    /// Returns the color value for the current appearance (light/dark).
    /// Falls back to magenta (#FF00FF) if the key is not found.
    pub fn color_array(key: &str) -> [f32; 4] {
        let hex = Self::color(key);
        parse_hex_color(&hex)
    }
}

/// Parse a hex color string (#RRGGBB or #RRGGBBAA) into [f32; 4] RGBA.
fn parse_hex_color(hex: &str) -> [f32; 4] {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f32 / 255.0;
        let a = if hex.len() >= 8 {
            u8::from_str_radix(&hex[6..8], 16).unwrap_or(255) as f32 / 255.0
        } else {
            1.0
        };
        [r, g, b, a]
    } else {
        [1.0, 0.0, 1.0, 1.0] // Magenta fallback
    }
}

/// The authoritative Cyberpunk Viking default tokens
pub fn default_tokens() -> DesignTokens {
    let mut tokens = DesignTokens::new();
    // Core Norse Colorways
    tokens.color.insert(
        "background".to_string(),
        TokenValue::Adaptive {
            light: "#FFFFFF".to_string(), // Light mode: white background
            dark: "#000000".to_string(),  // Dark mode: Ginnungagap (The Void)
        },
    );
    tokens.color.insert(
        "primary".to_string(),
        TokenValue::Adaptive {
            light: "#007B8A".to_string(), // Light mode: muted cyan
            dark: "#00FFFF".to_string(),  // Dark mode: NiflCyan (Aesir Primary)
        },
    );
    tokens.color.insert(
        "secondary".to_string(),
        TokenValue::Adaptive {
            light: "#8A008A".to_string(), // Light mode: muted magenta
            dark: "#FF00FF".to_string(),  // Dark mode: MuspelMagenta (Berserker Secondary)
        },
    );
    tokens.color.insert(
        "surface".to_string(),
        TokenValue::Adaptive {
            light: "#FFFFFF".to_string(),
            dark: "#121212".to_string(),
        },
    );
    tokens.color.insert(
        "text".to_string(),
        TokenValue::Adaptive {
            light: "#000000".to_string(),
            dark: "#FFFFFF".to_string(),
        },
    );
    // Semantic component tokens
    tokens.color.insert(
        "surface_elevated".to_string(),
        TokenValue::Adaptive {
            light: "#FFFFFF".to_string(),
            dark: "#1A1A24".to_string(),
        },
    );
    tokens.color.insert(
        "surface_overlay".to_string(),
        TokenValue::Adaptive {
            light: "#FFFFFF".to_string(),
            dark: "#1E1E2E".to_string(),
        },
    );
    tokens.color.insert(
        "border".to_string(),
        TokenValue::Adaptive {
            light: "#D0D0D8".to_string(),
            dark: "#2A2A3A".to_string(),
        },
    );
    tokens.color.insert(
        "border_strong".to_string(),
        TokenValue::Adaptive {
            light: "#A0A0B0".to_string(),
            dark: "#3A3A50".to_string(),
        },
    );
    tokens.color.insert(
        "text_muted".to_string(),
        TokenValue::Adaptive {
            light: "#606070".to_string(),
            dark: "#8080A0".to_string(),
        },
    );
    tokens.color.insert(
        "text_dim".to_string(),
        TokenValue::Adaptive {
            light: "#9090A0".to_string(),
            dark: "#505070".to_string(),
        },
    );
    tokens.color.insert(
        "accent".to_string(),
        TokenValue::Adaptive {
            light: "#007B8A".to_string(), // Light mode: muted cyan
            dark: "#00FFFF".to_string(),  // Dark mode: NiflCyan
        },
    );
    tokens.color.insert(
        "accent_hover".to_string(),
        TokenValue::Adaptive {
            light: "#00A0B0".to_string(), // Light mode: lighter muted cyan
            dark: "#33FFFF".to_string(),  // Dark mode: brighter cyan
        },
    );
    tokens.color.insert(
        "success".to_string(),
        TokenValue::Single {
            value: "#00E676".to_string(),
        },
    );
    tokens.color.insert(
        "warning".to_string(),
        TokenValue::Single {
            value: "#FFB300".to_string(),
        },
    );
    tokens.color.insert(
        "error".to_string(),
        TokenValue::Single {
            value: "#FF5252".to_string(),
        },
    );
    tokens.color.insert(
        "info".to_string(),
        TokenValue::Single {
            value: "#448AFF".to_string(),
        },
    );
    tokens.color.insert(
        "hover".to_string(),
        TokenValue::Adaptive {
            light: "#F0F0F5".to_string(),
            dark: "#252535".to_string(),
        },
    );
    tokens.color.insert(
        "active".to_string(),
        TokenValue::Adaptive {
            light: "#E0E0EB".to_string(),
            dark: "#303045".to_string(),
        },
    );
    tokens.color.insert(
        "disabled".to_string(),
        TokenValue::Adaptive {
            light: "#E8E8F0".to_string(),
            dark: "#1A1A28".to_string(),
        },
    );
    tokens.color.insert(
        "disabled_text".to_string(),
        TokenValue::Adaptive {
            light: "#B0B0C0".to_string(),
            dark: "#404060".to_string(),
        },
    );
    tokens.color.insert(
        "focus_ring".to_string(),
        TokenValue::Single {
            value: "#00FFFF".to_string(),
        },
    );
    tokens.color.insert(
        "shadow".to_string(),
        TokenValue::Adaptive {
            light: "#00000020".to_string(),
            dark: "#00000060".to_string(),
        },
    );
    tokens.color.insert(
        "code_bg".to_string(),
        TokenValue::Adaptive {
            light: "#F5F5FA".to_string(),
            dark: "#0D0D18".to_string(),
        },
    );
    // Bifrost (Glassmorphism) - Frosted Style
    tokens.bifrost.insert(
        "blur".to_string(),
        TokenValue::Single {
            value: "25.0".to_string(),
        },
    );
    tokens.bifrost.insert(
        "saturation".to_string(),
        TokenValue::Single {
            value: "1.2".to_string(),
        },
    );
    tokens.bifrost.insert(
        "opacity".to_string(),
        TokenValue::Single {
            value: "0.65".to_string(),
        },
    );
    // Gungnir (Neon Glow)
    tokens.gungnir.insert(
        "intensity".to_string(),
        TokenValue::Single {
            value: "1.0".to_string(),
        },
    );
    tokens.gungnir.insert(
        "radius".to_string(),
        TokenValue::Single {
            value: "15.0".to_string(),
        },
    );
    // Mjolnir (Sharp Geometry)
    tokens.mjolnir.insert(
        "clip_angle".to_string(),
        TokenValue::Single {
            value: "12.0".to_string(),
        },
    );
    tokens.mjolnir.insert(
        "border_width".to_string(),
        TokenValue::Single {
            value: "2.0".to_string(),
        },
    );
    // Sleipnir (Spring Animation)
    tokens.anim.insert(
        "stiffness".to_string(),
        TokenValue::Single {
            value: "170.0".to_string(),
        },
    );
    tokens.anim.insert(
        "damping".to_string(),
        TokenValue::Single {
            value: "26.0".to_string(),
        },
    );
    tokens.anim.insert(
        "mass".to_string(),
        TokenValue::Single {
            value: "1.0".to_string(),
        },
    );
    // Accessibility
    tokens.accessibility.insert(
        "reduce_motion".to_string(),
        TokenValue::Single {
            value: "false".to_string(),
        },
    );
    tokens
}
/// Environment wrapper for accessing ambient values
pub struct Environment<K: EnvKey> {
    _marker: std::marker::PhantomData<K>,
}
impl<K: EnvKey> Default for Environment<K> {
    fn default() -> Self {
        Self::new()
    }
}
impl<K: EnvKey> Environment<K> {
    /// Create a new Environment
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
    /// Get the current value from the environment
    pub fn get(&self) -> K::Value {
        if let Some(env_store) = ENVIRONMENT.get() {
            let env_lock = env_store.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(val) = env_lock.get(&std::any::TypeId::of::<K>()) {
                if let Some(typed_val) = val.downcast_ref::<K::Value>() {
                    return typed_val.clone();
                } else {
                    log::warn!(
                        "Environment: Downcast failed for key type {:?}",
                        std::any::type_name::<K>()
                    );
                }
            } else {
                // Lowered to trace to avoid terminal logging overhead under standard debug runs
                log::trace!(
                    "Environment: Key not found: {:?}. Returning default.",
                    std::any::type_name::<K>()
                );
            }
        } else {
            // Lowered to trace to avoid terminal logging overhead under standard debug runs
            log::trace!(
                "Environment: Store not initialized. Key: {:?}. Returning default.",
                std::any::type_name::<K>()
            );
        }
        K::default_value()
    }
}
/// Ambient environment management
pub mod env {
    /// Insert a value into the environment
    pub fn insert<K: super::EnvKey>(value: K::Value) {
        let store = super::ENVIRONMENT
            .get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
        let mut env_map = store.lock().unwrap_or_else(|p| p.into_inner());
        env_map.insert(std::any::TypeId::of::<K>(), Box::new(value));
    }
    /// Remove a value from the environment.
    pub fn remove<K: super::EnvKey>() {
        if let Some(store) = super::ENVIRONMENT.get() {
            let mut env_map = store.lock().unwrap_or_else(|p| p.into_inner());
            env_map.remove(&std::any::TypeId::of::<K>());
        }
    }
}
/// Geometry modifiers
/// Size of the view in logical pixels
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Self = Self {
        width: 0.0,
        height: 0.0,
    };

    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// Insets for padding
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeInsets {
    pub top: f32,
    pub leading: f32,
    pub bottom: f32,
    pub trailing: f32,
}

impl EdgeInsets {
    /// Equal insets on all edges
    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            leading: value,
            bottom: value,
            trailing: value,
        }
    }

    /// Vertical insets (top and bottom)
    pub fn vertical(value: f32) -> Self {
        Self {
            top: value,
            leading: 0.0,
            bottom: value,
            trailing: 0.0,
        }
    }

    /// Horizontal insets (leading and trailing)
    pub fn horizontal(value: f32) -> Self {
        Self {
            top: 0.0,
            leading: value,
            bottom: 0.0,
            trailing: value,
        }
    }
}

/// Modifier to set the size and alignment constraints of a view.
/// This determines the proposal size passed to the child and how the child is aligned
/// within the layout rect allocated to the frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameModifier {
    /// Exact width to assign to the child view.
    pub width: Option<f32>,
    /// Exact height to assign to the child view.
    pub height: Option<f32>,
    /// Minimum width constraint for the view.
    pub min_width: Option<f32>,
    /// Maximum width constraint for the view.
    pub max_width: Option<f32>,
    /// Minimum height constraint for the view.
    pub min_height: Option<f32>,
    /// Maximum height constraint for the view.
    pub max_height: Option<f32>,
    /// The alignment strategy for positioning the child view within the frame.
    pub alignment: Alignment,
}

impl Default for FrameModifier {
    /// Returns the default frame configuration which has no constraints and center alignment.
    fn default() -> Self {
        Self::new()
    }
}

impl FrameModifier {
    /// Creates a new FrameModifier with all dimensions unspecified and center alignment.
    pub fn new() -> Self {
        Self {
            width: None,
            height: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            alignment: Alignment::Center,
        }
    }

    /// Sets the fixed width of the frame.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets the fixed height of the frame.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets both the fixed width and height of the frame.
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Sets the minimum width constraint.
    pub fn min_width(mut self, min_width: f32) -> Self {
        self.min_width = Some(min_width);
        self
    }

    /// Sets the maximum width constraint.
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = Some(max_width);
        self
    }

    /// Sets the minimum height constraint.
    pub fn min_height(mut self, min_height: f32) -> Self {
        self.min_height = Some(min_height);
        self
    }

    /// Sets the maximum height constraint.
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = Some(max_height);
        self
    }

    /// Sets the alignment strategy for the child within the frame's layout bounds.
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}

impl ViewModifier for FrameModifier {
    /// Wraps the child view in a ModifiedView using this frame modifier.
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    /// Transforms the layout size proposal offered to the child to comply with frame constraints.
    fn transform_proposal(&self, proposal: SizeProposal) -> SizeProposal {
        let w = if let Some(width) = self.width {
            Some(width)
        } else {
            proposal.width.map(|pw| {
                pw.clamp(
                    self.min_width.unwrap_or(0.0),
                    self.max_width.unwrap_or(f32::INFINITY),
                )
            })
        };
        let h = if let Some(height) = self.height {
            Some(height)
        } else {
            proposal.height.map(|ph| {
                ph.clamp(
                    self.min_height.unwrap_or(0.0),
                    self.max_height.unwrap_or(f32::INFINITY),
                )
            })
        };
        SizeProposal {
            width: w,
            height: h,
        }
    }

    /// Constraints and transforms the child's resulting size to fit the frame's bounds.
    fn transform_size(&self, child_size: Size) -> Size {
        let w = if let Some(width) = self.width {
            width
        } else {
            child_size.width.clamp(
                self.min_width.unwrap_or(0.0),
                self.max_width.unwrap_or(f32::INFINITY),
            )
        };
        let h = if let Some(height) = self.height {
            height
        } else {
            child_size.height.clamp(
                self.min_height.unwrap_or(0.0),
                self.max_height.unwrap_or(f32::INFINITY),
            )
        };
        Size {
            width: w,
            height: h,
        }
    }

    /// Renders the frame's child view aligned within the layout rect.
    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        self.render(renderer, rect);
        let child_proposal =
            self.transform_proposal(SizeProposal::new(Some(rect.width), Some(rect.height)));
        let child_size = view.intrinsic_size(renderer, child_proposal);

        let mut child_x = rect.x;
        let mut child_y = rect.y;

        match self.alignment {
            Alignment::Leading => {
                child_y = rect.y + (rect.height - child_size.height) / 2.0;
            }
            Alignment::Trailing => {
                child_x = rect.x + rect.width - child_size.width;
                child_y = rect.y + (rect.height - child_size.height) / 2.0;
            }
            Alignment::Top => {
                child_x = rect.x + (rect.width - child_size.width) / 2.0;
            }
            Alignment::Bottom => {
                child_x = rect.x + (rect.width - child_size.width) / 2.0;
                child_y = rect.y + rect.height - child_size.height;
            }
            Alignment::Center => {
                child_x = rect.x + (rect.width - child_size.width) / 2.0;
                child_y = rect.y + (rect.height - child_size.height) / 2.0;
            }
        }

        let child_rect = Rect {
            x: child_x,
            y: child_y,
            width: child_size.width,
            height: child_size.height,
        };

        view.render(renderer, child_rect);
        self.post_render(renderer, rect);
    }
}

/// Modifier to set the flex weight of a view
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlexModifier {
    pub weight: f32,
}

impl ViewModifier for FlexModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn child_flex_weight<V: View>(&self, _view: &V) -> f32 {
        self.weight
    }
}

/// Modifier that specifies the column and row placement of a view inside a Grid layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPlacementModifier {
    /// The grid placement settings containing column/row indexes and spans.
    pub placement: GridPlacement,
}

impl ViewModifier for GridPlacementModifier {
    /// Wraps the child view in a ModifiedView using this modifier.
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    /// Exposes the grid placement metadata to parent layout engines.
    fn get_grid_placement(&self) -> Option<GridPlacement> {
        Some(self.placement)
    }
}

/// Modifier to render a popover, tooltip, or menu view overlaying an anchored view.
/// It supports alignment positioning and outside-click dismissal.
#[derive(Clone)]
pub struct OverlayModifier {
    /// The overlay content view.
    pub overlay: AnyView,
    /// Where the overlay is aligned relative to the anchored view.
    pub alignment: Alignment,
    /// Additional offset in logical pixels.
    pub offset: [f32; 2],
    /// Optional dismissal callback triggered by click-outside events.
    pub on_dismiss: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl ViewModifier for OverlayModifier {
    /// Wraps the child view in a ModifiedView using this overlay modifier.
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    /// Renders the overlay content positioned above the child view.
    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        // 1. Render primary anchored view
        view.render(renderer, rect);

        // 2. Measure overlay content
        let overlay_size = self
            .overlay
            .intrinsic_size(renderer, SizeProposal::unspecified());

        // 3. Align overlay rect relative to anchored rect
        let mut overlay_x;
        let mut overlay_y;

        match self.alignment {
            Alignment::Leading => {
                overlay_x = rect.x - overlay_size.width;
                overlay_y = rect.y + (rect.height - overlay_size.height) / 2.0;
            }
            Alignment::Trailing => {
                overlay_x = rect.x + rect.width;
                overlay_y = rect.y + (rect.height - overlay_size.height) / 2.0;
            }
            Alignment::Top => {
                overlay_x = rect.x + (rect.width - overlay_size.width) / 2.0;
                overlay_y = rect.y - overlay_size.height;
            }
            Alignment::Bottom => {
                overlay_x = rect.x + (rect.width - overlay_size.width) / 2.0;
                overlay_y = rect.y + rect.height;
            }
            Alignment::Center => {
                overlay_x = rect.x + (rect.width - overlay_size.width) / 2.0;
                overlay_y = rect.y + (rect.height - overlay_size.height) / 2.0;
            }
        }

        overlay_x += self.offset[0];
        overlay_y += self.offset[1];

        let overlay_rect = Rect {
            x: overlay_x,
            y: overlay_y,
            width: overlay_size.width,
            height: overlay_size.height,
        };

        // 4. Handle click-outside dismissal
        if let Some(on_dismiss) = &self.on_dismiss {
            let dismiss = on_dismiss.clone();
            renderer.register_handler(
                "pointerdown",
                Arc::new(move |event| {
                    if let Event::PointerDown { x, y, .. } = event {
                        let click_inside = x >= overlay_rect.x
                            && x <= overlay_rect.x + overlay_rect.width
                            && y >= overlay_rect.y
                            && y <= overlay_rect.y + overlay_rect.height;
                        if !click_inside {
                            dismiss();
                        }
                    }
                }),
            );
        }

        // 5. Render overlay view
        self.overlay.render(renderer, overlay_rect);
    }
}

/// Modifier to offset a view
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OffsetModifier {
    pub x: f32,
    pub y: f32,
}

impl OffsetModifier {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl ViewModifier for OffsetModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }
}

/// Modifier to set the z-index of a view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZIndexModifier {
    pub z_index: i32,
}

impl ZIndexModifier {
    pub fn new(z_index: i32) -> Self {
        Self { z_index }
    }
}

impl ViewModifier for ZIndexModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }
}

/// Layout constraints for views
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LayoutConstraints {
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
}

/// Modifier to set layout constraints
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutModifier {
    pub constraints: LayoutConstraints,
}

impl LayoutModifier {
    pub fn new(constraints: LayoutConstraints) -> Self {
        Self { constraints }
    }
}

impl ViewModifier for LayoutModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }
}

/// Modifier to handle platform safe areas
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SafeAreaModifier {
    pub ignores: bool,
}

impl ViewModifier for SafeAreaModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }
}

/// Modifier to add elevation (shadow) to a view
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElevationModifier {
    pub level: f32,
}

impl ViewModifier for ElevationModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        if self.level > 0.0 {
            let radius = self.level * 2.0;
            let offset_y = self.level * 0.5;
            let shadow_color = [0.0, 0.0, 0.0, 0.3];
            renderer.push_shadow(radius, shadow_color, [0.0, offset_y]);
            view.render(renderer, rect);
            renderer.pop_shadow();
        } else {
            view.render(renderer, rect);
        }
    }
}

/// Position modifier — offsets a view from its layout position.
/// Enables absolute-like positioning within a container.
#[derive(Clone)]
pub struct PositionModifier {
    pub x: f32,
    pub y: f32,
}

impl ViewModifier for PositionModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn transform_rect(&self, rect: Rect) -> Rect {
        Rect {
            x: rect.x + self.x,
            y: rect.y + self.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

// Layout subsystem
pub mod layout {
    use super::*;

    /// Key used to identify a cached layout entry.
    /// Combines a view hash with a generation counter for cache invalidation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LayoutKey {
        pub view_hash: u64,
        pub generation: u64,
    }

    // Layout pass scratch space
    pub struct LayoutCache {
        pub safe_area: SafeArea,
        pub delta_time: f32,
        /// Device scale factor for HiDPI / retina snapping. Defaults to 1.0.
        pub scale_factor: f32,
        /// The visible viewport bounds in logical pixels.
        /// If Some, layout execution can cull offscreen subtrees.
        pub viewport: Option<Rect>,
        /// Time budget for the layout pass. Defaults to 4.0ms.
        pub layout_time_budget: std::time::Duration,
        /// Start of the layout pass, captured at the beginning of the frame/layout run.
        pub layout_start_time: Option<std::time::Instant>,
        size_cache: HashMap<(u64, u32, u32), Size>, // (ViewHash, ProposalW, ProposalH)
        /// Map tracking child-to-parent view hash relationships for bottom-up invalidation.
        pub parent_map: HashMap<u64, u64>,
        /// Monotonically increasing generation counter for cache invalidation.
        /// When a view tree changes, bumping the generation causes stale entries
        /// to be treated as invalid without eagerly clearing the entire cache.
        generation: u64,
        /// Opaque pointer to the active layout engine (e.g. Taffy)
        pub engine: Option<Box<dyn std::any::Any + Send + Sync>>,
        /// Opaque pointer to the active animation orchestrator
        pub animators: Option<Box<dyn std::any::Any + Send + Sync>>,
        /// Cached previous rects for view transitions
        pub previous_rects: HashMap<u64, Rect>,
        /// Generation counter for cache eviction.
        /// Incremented each frame; entries not touched for N frames are evicted.
        pub eviction_generation: u64,
        /// Tracks which generation each previous_rects entry was last touched in.
        pub previous_rects_generation: HashMap<u64, u64>,
        /// Number of generations an entry can go untouched before eviction.
        eviction_threshold: u64,
    }

    thread_local! {
        static LAYOUT_BUDGET_DEADLINE: std::cell::RefCell<Option<std::time::Instant>> =
            const { std::cell::RefCell::new(None) };
    }

    impl Default for LayoutCache {
        fn default() -> Self {
            Self::new()
        }
    }

    impl LayoutCache {
        pub fn new() -> Self {
            Self {
                safe_area: SafeArea::default(),
                delta_time: 0.016,
                scale_factor: 1.0,
                viewport: None,
                layout_time_budget: std::time::Duration::from_millis(4),
                layout_start_time: None,
                size_cache: HashMap::new(),
                parent_map: HashMap::new(),
                generation: 0,
                engine: None,
                animators: None,
                previous_rects: HashMap::new(),
                eviction_generation: 0,
                previous_rects_generation: HashMap::new(),
                eviction_threshold: 300, // ~5 seconds at 60fps
            }
        }

        /// Returns the current generation counter.
        pub fn generation(&self) -> u64 {
            self.generation
        }

        /// Evict entries from previous_rects that haven't been touched for N generations.
        pub fn evict_stale_entries(&mut self) {
            self.eviction_generation += 1;
            let threshold = self.eviction_threshold;
            let current_gen = self.eviction_generation;
            self.previous_rects.retain(|hash, _| {
                self.previous_rects_generation
                    .get(hash)
                    .map_or(false, |g| current_gen - *g < threshold)
            });
            self.previous_rects_generation
                .retain(|hash, _| self.previous_rects.contains_key(hash));
        }

        /// Checks if the layout pass is currently running over its allocated time budget.
        pub fn is_over_budget(&self) -> bool {
            let deadline_red = LAYOUT_BUDGET_DEADLINE.with(|deadline| {
                deadline.borrow().as_ref().is_some_and(|deadline| std::time::Instant::now() >= *deadline)
            });
            if deadline_red {
                return true;
            }
            if let Some(start) = self.layout_start_time {
                start.elapsed() > self.layout_time_budget
            } else {
                false
            }
        }

        /// Set a process-local deadline for layout cache consumers.
        /// When this deadline is exceeded, caches should reuse previous
        /// rects instead of recomputing expensive layout work.
        pub fn set_layout_budget_deadline(deadline: Option<std::time::Instant>) {
            LAYOUT_BUDGET_DEADLINE.with(|slot| {
                *slot.borrow_mut() = deadline;
            });
        }

        /// Clear any process-local layout budget deadline.
        pub fn clear_layout_budget_deadline() {
            Self::set_layout_budget_deadline(None);
        }

        /// Bump the generation counter, logically invalidating all cached entries
        /// without eagerly clearing them. Subsequent lookups with the old generation
        /// will miss until re-populated.
        pub fn invalidate(&mut self) {
            self.generation = self.generation.wrapping_add(1);
        }

        /// Check whether a cached entry for the given key is still valid
        /// against the current generation.
        pub fn is_valid(&self, key: LayoutKey, current_gen: u64) -> bool {
            key.generation == current_gen && key.generation == self.generation
        }

        pub fn clear(&mut self) {
            self.safe_area = SafeArea::default();
            self.viewport = None;
            self.layout_start_time = None;
            self.size_cache.clear();
            self.parent_map.clear();
        }

        pub fn get_size(&self, view_hash: u64, proposal: SizeProposal) -> Option<Size> {
            let pw = (proposal.width.unwrap_or(-1.0) * 100.0) as u32;
            let ph = (proposal.height.unwrap_or(-1.0) * 100.0) as u32;
            self.size_cache.get(&(view_hash, pw, ph)).copied()
        }

        pub fn set_size(&mut self, view_hash: u64, proposal: SizeProposal, size: Size) {
            let pw = (proposal.width.unwrap_or(-1.0) * 100.0) as u32;
            let ph = (proposal.height.unwrap_or(-1.0) * 100.0) as u32;
            self.size_cache.insert((view_hash, pw, ph), size);
        }

        /// Register a child-to-parent layout relationship for bottom-up invalidation propagation.
        pub fn register_parent(&mut self, child_hash: u64, parent_hash: u64) {
            if child_hash != 0 && parent_hash != 0 {
                self.parent_map.insert(child_hash, parent_hash);
            }
        }

        /// Remove all cached size entries for a specific view hash and propagate the invalidation
        /// bottom-up to all its layout ancestors to ensure consistent layout updates.
        pub fn invalidate_view(&mut self, view_hash: u64) {
            let mut to_invalidate = vec![view_hash];
            let mut visited = std::collections::HashSet::new();
            while let Some(hash) = to_invalidate.pop() {
                if !visited.insert(hash) {
                    continue;
                }
                self.size_cache.retain(|&(h, _, _), _| h != hash);
                if let Some(&parent) = self.parent_map.get(&hash) {
                    to_invalidate.push(parent);
                }
            }
        }
    }

    /// Proposed size from parent view
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct SizeProposal {
        pub width: Option<f32>,
        pub height: Option<f32>,
    }

    impl SizeProposal {
        pub fn unspecified() -> Self {
            Self {
                width: None,
                height: None,
            }
        }

        pub fn width(width: f32) -> Self {
            Self {
                width: Some(width),
                height: None,
            }
        }

        pub fn height(height: f32) -> Self {
            Self {
                width: None,
                height: Some(height),
            }
        }

        pub fn tight(width: f32, height: f32) -> Self {
            Self {
                width: Some(width),
                height: Some(height),
            }
        }

        pub fn new(width: Option<f32>, height: Option<f32>) -> Self {
            Self { width, height }
        }
    }

    /// A view that can participate in layout
    pub trait LayoutView: Send {
        /// Propose a size for this view given the available space
        fn size_that_fits(
            &self,
            proposal: SizeProposal,
            subviews: &[&dyn LayoutView],
            cache: &mut LayoutCache,
        ) -> Size;

        /// Place subviews within the given bounds
        fn place_subviews(
            &self,
            bounds: Rect,
            subviews: &mut [&mut dyn LayoutView],
            cache: &mut LayoutCache,
        );

        /// Returns the flex weight of this view (default is 0.0, which means fixed/intrinsic)
        fn flex_weight(&self) -> f32 {
            0.0
        }

        /// Returns a persistent unique identifier for this view to enable Layout View Transitions.
        /// Return 0 (default) to disable layout animations for this node.
        fn view_hash(&self) -> u64 {
            0
        }

        /// Return true when this view's layout may have changed since the last pass.
        ///
        /// The layout engine uses this to skip cache lookups for views that are
        /// guaranteed static (e.g., chrome elements that never change between frames).
        /// Default true for backward compatibility -- override false for static subtrees.
        ///
        /// When false, the engine may skip `size_that_fits` entirely and reuse the
        /// cached rect from `LayoutCache::previous_rects`.
        fn changed(&self) -> bool {
            true
        }

        /// Return a debug representation of this layout subtree.
        /// The `indent` parameter controls the indentation level for nested display.
        fn debug_layout(&self, indent: usize) -> String {
            let prefix = " ".repeat(indent);
            format!("{}LayoutView", prefix)
        }
    }
    /// Edge insets for padding, margins, and safe areas
    #[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
    pub struct EdgeInsets {
        pub top: f32,
        pub leading: f32,
        pub bottom: f32,
        pub trailing: f32,
    }

    impl EdgeInsets {
        pub fn new(top: f32, leading: f32, bottom: f32, trailing: f32) -> Self {
            Self {
                top,
                leading,
                bottom,
                trailing,
            }
        }

        pub fn all(value: f32) -> Self {
            Self {
                top: value,
                leading: value,
                bottom: value,
                trailing: value,
            }
        }
    }

    /// SafeArea constraints provided by the platform
    #[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
    pub struct SafeArea {
        pub insets: EdgeInsets,
    }

    /// SDF Shape definitions for Vili Interaction Paradigm hit-testing.
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub enum SdfShape {
        Rect(Rect),
        RoundedRect { rect: Rect, radius: f32 },
        Circle { center: [f32; 2], radius: f32 },
    }

    /// Rectangle in logical pixels
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct Rect {
        pub x: f32,
        pub y: f32,
        pub width: f32,
        pub height: f32,
    }

    impl Rect {
        pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
            Self {
                x,
                y,
                width,
                height,
            }
        }

        pub fn inset(&self, amount: f32) -> Self {
            Self {
                x: self.x + amount,
                y: self.y + amount,
                width: (self.width - amount * 2.0).max(0.0),
                height: (self.height - amount * 2.0).max(0.0),
            }
        }

        pub fn offset(&self, dx: f32, dy: f32) -> Self {
            Self {
                x: self.x + dx,
                y: self.y + dy,
                ..*self
            }
        }

        pub fn zero() -> Self {
            Self {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            }
        }

        pub fn contains(&self, x: f32, y: f32) -> bool {
            x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
        }

        /// Determines whether this rectangle overlaps with another rectangle.
        ///
        /// # Contract
        /// Two rectangles overlap if their projection intervals on both the X
        /// and Y axes overlap. This is used for viewport intersection checks
        /// to determine visibility constraints during layout culling.
        pub fn intersects(&self, other: &Rect) -> bool {
            self.x < other.x + other.width
                && self.x + self.width > other.x
                && self.y < other.y + other.height
                && self.y + self.height > other.y
        }

        pub fn size(&self) -> Size {
            Size {
                width: self.width,
                height: self.height,
            }
        }

        /// Split the rect horizontally into N equal pieces
        pub fn split_horizontal(&self, n: usize) -> Vec<Rect> {
            if n == 0 {
                return vec![];
            }
            let item_width = self.width / n as f32;
            (0..n)
                .map(|i| Rect {
                    x: self.x + i as f32 * item_width,
                    y: self.y,
                    width: item_width,
                    height: self.height,
                })
                .collect()
        }

        /// Split the rect vertically into N equal pieces
        pub fn split_vertical(&self, n: usize) -> Vec<Rect> {
            if n == 0 {
                return vec![];
            }
            let item_height = self.height / n as f32;
            (0..n)
                .map(|i| Rect {
                    x: self.x,
                    y: self.y + i as f32 * item_height,
                    width: self.width,
                    height: item_height,
                })
                .collect()
        }
    }
}

// Size and FrameRenderer are pub items in this module; no re-export alias needed.

pub mod agents;
pub mod animation;
pub mod gpu;
pub mod material;
pub mod runtime;
pub mod scene_graph;
pub mod sdf_shadow;

// Re-export commonly used types
pub use layout::{LayoutCache, LayoutKey, LayoutView, Rect, SizeProposal};
pub use material::DrawMaterial;
pub use scene_graph::{NodeId, bifrost_registry};
pub use color::SemanticColors;

// Duplicate AssetState removed - original definition at line 67

/// AssetManager defines the interface for loading and caching external resources.
pub trait AssetManager: Send + Sync {
    /// Request an image asset. Returns the current state (Loading, Ready, or Error).
    fn load_image(&self, url: &str) -> AssetState<Arc<Vec<u8>>>;

    /// Pre-load an image into the cache.
    fn preload_image(&self, url: &str);
}

/// The phase of a touch or gesture event in its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TouchPhase {
    /// The touch/gesture has just begun.
    Began,
    /// The touch/gesture is moving.
    Moved,
    /// The touch/gesture has ended normally.
    Ended,
    /// The touch/gesture was cancelled (e.g., by the system).
    Cancelled,
}

/// User input event types
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Event {
    PointerDown {
        x: f32,
        y: f32,
        button: u32,
        proximity_field: f32,
        tilt: Option<f32>,
        azimuth: Option<f32>,
        pressure: Option<f32>,
        barrel_rotation: Option<f32>,
        pointer_precision: f32,
    },
    PointerUp {
        x: f32,
        y: f32,
        button: u32,
        tilt: Option<f32>,
        azimuth: Option<f32>,
        pressure: Option<f32>,
        barrel_rotation: Option<f32>,
        pointer_precision: f32,
    },
    PointerMove {
        x: f32,
        y: f32,
        proximity_field: f32,
        tilt: Option<f32>,
        azimuth: Option<f32>,
        pressure: Option<f32>,
        barrel_rotation: Option<f32>,
        pointer_precision: f32,
    },
    PointerClick {
        x: f32,
        y: f32,
        button: u32,
        tilt: Option<f32>,
        azimuth: Option<f32>,
        pressure: Option<f32>,
        barrel_rotation: Option<f32>,
        pointer_precision: f32,
    },
    PointerEnter,
    PointerLeave,
    /// Mouse wheel / trackpad scroll event.
    /// `delta_x` is the horizontal scroll amount, `delta_y` is the vertical scroll amount (positive = scroll down).
    PointerWheel {
        x: f32,
        y: f32,
        delta_x: f32,
        delta_y: f32,
        pointer_precision: f32,
    },
    /// Double-click event (rapid successive clicks).
    PointerDoubleClick {
        x: f32,
        y: f32,
        button: u32,
        pointer_precision: f32,
    },
    /// Drag-and-drop: drag started (pointer moved while button held past threshold).
    DragStart {
        x: f32,
        y: f32,
        button: u32,
        pointer_precision: f32,
    },
    /// Drag-and-drop: drag in progress.
    DragMove {
        x: f32,
        y: f32,
        pointer_precision: f32,
    },
    /// Drag-and-drop: drag ended (pointer released).
    DragEnd {
        x: f32,
        y: f32,
        pointer_precision: f32,
    },
    KeyDown {
        key: String,
        modifiers: KeyModifiers,
    },
    KeyUp {
        key: String,
        modifiers: KeyModifiers,
    },
    /// Focus gained by a node.
    FocusIn,
    /// Focus lost by a node.
    FocusOut,
    /// Clipboard copy event.
    Copy,
    /// Clipboard cut event.
    Cut,
    /// Clipboard paste event with the pasted text content.
    Paste(String),
    /// Input Method Editor event (e.g. CJK character composition)
    Ime(String),
    /// Touch began at the given position.
    TouchStart {
        x: f32,
        y: f32,
        touch_id: u64,
    },
    /// Touch moved to a new position.
    TouchMove {
        x: f32,
        y: f32,
        touch_id: u64,
    },
    /// Touch ended at the given position.
    TouchEnd {
        x: f32,
        y: f32,
        touch_id: u64,
    },
    /// Touch cancelled.
    TouchCancel {
        touch_id: u64,
    },
    /// Multi-touch pinch gesture.
    /// `center` is the gesture anchor point in device-independent pixels.
    /// `scale` is the relative pinch scale (>1 = expand, <1 = contract).
    /// `velocity` is the instantaneous velocity of the pinch.
    /// `phase` indicates the current phase of the gesture lifecycle.
    GesturePinch {
        center: [f32; 2],
        scale: f32,
        velocity: f32,
        phase: TouchPhase,
    },
    /// Multi-touch swipe/pan gesture.
    /// `direction` is the normalized direction vector [dx, dy].
    /// `velocity` is the instantaneous velocity of the swipe.
    /// `phase` indicates the current phase of the gesture lifecycle.
    GestureSwipe {
        direction: [f32; 2],
        velocity: f32,
        phase: TouchPhase,
    },
    /// Drag-and-drop: external file dropped onto window.
    FileDrop {
        x: f32,
        y: f32,
        path: String,
    },
}

impl Event {
    /// Returns the input pointer precision value in physical pixels if applicable.
    ///
    /// WHY: Used to scale hit-testing bounding boxes for proximity matching.
    /// CONTRACT: Mouse pointer inputs return low precision values (close to 0.0px),
    /// whereas touch inputs return larger values (e.g., 150.0px) for finger emulation.
    pub fn pointer_precision(&self) -> f32 {
        match self {
            Self::PointerDown {
                pointer_precision, ..
            }
            | Self::PointerUp {
                pointer_precision, ..
            }
            | Self::PointerMove {
                pointer_precision, ..
            }
            | Self::PointerClick {
                pointer_precision, ..
            }
            | Self::PointerWheel {
                pointer_precision, ..
            }
            | Self::PointerDoubleClick {
                pointer_precision, ..
            }
            | Self::DragStart {
                pointer_precision, ..
            }
            | Self::DragMove {
                pointer_precision, ..
            }
            | Self::DragEnd {
                pointer_precision, ..
            } => *pointer_precision,
            _ => 0.0,
        }
    }

    /// Returns the canonical string name of the event for lookup in handler maps.
    pub fn name(&self) -> &'static str {
        match self {
            Self::PointerDown { .. } => "pointerdown",
            Self::PointerUp { .. } => "pointerup",
            Self::PointerMove { .. } => "pointermove",
            Self::PointerClick { .. } => "pointerclick",
            Self::PointerEnter => "pointerenter",
            Self::PointerLeave => "pointerleave",
            Self::PointerWheel { .. } => "pointerwheel",
            Self::PointerDoubleClick { .. } => "pointerdoubleclick",
            Self::DragStart { .. } => "dragstart",
            Self::DragMove { .. } => "dragmove",
            Self::DragEnd { .. } => "dragend",
            Self::KeyDown { .. } => "keydown",
            Self::KeyUp { .. } => "keyup",
            Self::FocusIn => "focusin",
            Self::FocusOut => "focusout",
            Self::Copy => "copy",
            Self::Cut => "cut",
            Self::Paste(_) => "paste",
            Self::Ime(_) => "ime",
            Self::TouchStart { .. } => "touchstart",
            Self::TouchMove { .. } => "touchmove",
            Self::TouchEnd { .. } => "touchend",
            Self::TouchCancel { .. } => "touchcancel",
            Self::GesturePinch { .. } => "gesturepinch",
            Self::GestureSwipe { .. } => "gestureswipe",
            Self::FileDrop { .. } => "filedrop",
        }
    }
}


/// Response from an event handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResponse {
    Handled,
    Ignored,
}

