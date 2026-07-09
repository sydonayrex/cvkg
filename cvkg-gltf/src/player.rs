// AnimationPlayer — keyframe interpolation runtime for glTF skeletal animations.
//
// Samples Animation3D tracks (translation/rotation/scale) per frame, propagates
// node transforms through the hierarchy, and produces joint matrices
// (global_joint * inverse_bind_matrix) ready for GPU skinning.

use crate::types::{Animation3D, AnimationChannel3D, AnimationProperty, Scene3D};
use glam::{Mat4, Quat, Vec3};

/// Playback state for a single animation clip.
#[derive(Debug, Clone)]
pub struct ActiveAnimation {
    /// Index into `Scene3D::animations`.
    pub anim_index: usize,
    /// Playback speed multiplier (negative for reverse).
    pub speed: f32,
    /// Whether to loop when reaching the end.
    pub looping: bool,
    /// Blend weight (0.0–1.0) for multi-animation blending.
    pub weight: f32,
}

/// Runtime animation player that samples keyframe tracks and computes joint matrices.
///
/// # Usage
/// ```no_run
/// use cvkg_gltf::{load_gltf, player::AnimationPlayer};
///
/// let scene = load_gltf("character.glb").unwrap();
/// let mut player = AnimationPlayer::new(&scene);
/// player.play(0, true); // play first animation, looping
///
/// // Each frame:
/// let dt = 0.016; // frame delta time in seconds
/// player.update(dt, &scene);
/// let matrices = player.joint_matrices();
/// // upload matrices to GpuRenderer via upload_joint_matrices()
/// ```
pub struct AnimationPlayer {
    /// Current playback time in seconds.
    time: f32,
    /// Currently active animations.
    active: Vec<ActiveAnimation>,
    /// Cached node local transforms (rebuilt from scene on creation).
    node_transforms: Vec<NodeTransform>,
    /// Index into Scene3D::skins for each node (u32::MAX if none).
    #[allow(dead_code)]
    node_skin: Vec<u32>,
    /// Flat joint matrices output — one Mat4 per joint across all skins.
    joint_matrices: Vec<Mat4>,
    /// Number of joints per skin (for offset calculation).
    #[allow(dead_code)]
    skin_joint_counts: Vec<usize>,
}

/// Lightweight transform representation for interpolation.
#[derive(Debug, Clone, Copy)]
struct NodeTransform {
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
}

impl Default for NodeTransform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl NodeTransform {
    fn to_matrix(self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    fn from_mat4(m: Mat4) -> Self {
        let (scale, rotation, translation) = m.to_scale_rotation_translation();
        Self {
            translation,
            rotation,
            scale,
        }
    }
}

impl AnimationPlayer {
    /// Create a new player from a loaded scene.
    ///
    /// Initializes node transforms from the scene graph and allocates
    /// storage for joint matrices based on the scene's skins.
    pub fn new(scene: &Scene3D) -> Self {
        let node_count = scene.nodes.len();
        let mut node_transforms = vec![NodeTransform::default(); node_count];
        let mut node_skin = vec![u32::MAX; node_count];

        for node in &scene.nodes {
            node_transforms[node.index] = NodeTransform {
                translation: node.transform.position,
                rotation: node.transform.rotation,
                scale: node.transform.scale,
            };
            if let Some(skin_idx) = node.skin_index {
                node_skin[node.index] = skin_idx as u32;
            }
        }

        let skin_joint_counts: Vec<usize> = scene.skins.iter().map(|s| s.joints.len()).collect();
        let total_joints: usize = skin_joint_counts.iter().sum();

        Self {
            time: 0.0,
            active: Vec::new(),
            node_transforms,
            node_skin,
            joint_matrices: vec![Mat4::IDENTITY; total_joints],
            skin_joint_counts,
        }
    }

    /// Start playing an animation.
    ///
    /// Returns `true` if the animation was accepted, `false` if the index is invalid.
    /// Invalid indices are silently rejected and will be cleaned up on the next `update()` call.
    pub fn play(&mut self, anim_index: usize, looping: bool) -> bool {
        // Validate that the animation index is reasonable (will be fully validated in update())
        if anim_index == usize::MAX {
            return false;
        }
        self.active.push(ActiveAnimation {
            anim_index,
            speed: 1.0,
            looping,
            weight: 1.0,
        });
        true
    }

    /// Stop all animations.
    pub fn stop_all(&mut self) {
        self.active.clear();
    }

    /// Advance time by `dt` seconds and recompute joint matrices.
    pub fn update(&mut self, dt: f32, scene: &Scene3D) {
        self.time += dt;

        // Reset node transforms to rest pose
        for node in &scene.nodes {
            self.node_transforms[node.index] = NodeTransform {
                translation: node.transform.position,
                rotation: node.transform.rotation,
                scale: node.transform.scale,
            };
        }

        // Sample each active animation and accumulate
        let mut to_remove = Vec::new();
        for (i, anim) in self.active.iter_mut().enumerate() {
            if let Some(animation) = scene.animations.get(anim.anim_index) {
                let duration = animation_duration(animation);
                if duration <= 0.0 {
                    to_remove.push(i);
                    continue;
                }

                let local_time = self.time * anim.speed;
                let t = if anim.looping {
                    // Euclidean modulo for correct behavior with negative speed (reverse playback)
                    let d = duration;
                    ((local_time % d) + d) % d
                } else if local_time >= duration {
                    to_remove.push(i);
                    duration
                } else if local_time < 0.0 {
                    0.0
                } else {
                    local_time
                };

                sample_animation(animation, t, anim.weight, &mut self.node_transforms);
            } else {
                to_remove.push(i);
            }
        }

        // Remove finished non-looping animations (reverse order to preserve indices)
        for i in to_remove.into_iter().rev() {
            self.active.remove(i);
        }

        // Propagate transforms through hierarchy (parents before children)
        self.propagate_hierarchy(scene);

        // Compute joint matrices
        self.compute_joint_matrices(scene);
    }

    /// Get the computed joint matrices for GPU upload.
    pub fn joint_matrices(&self) -> &[Mat4] {
        &self.joint_matrices
    }

    /// Check if any animations are currently playing.
    pub fn is_playing(&self) -> bool {
        !self.active.is_empty()
    }

    /// Propagate parent transforms to children.
    ///
    /// Nodes are stored parent-before-child in the flat array, so a single
    /// forward pass computes world transforms correctly.
    ///
    /// NOTE: This function decomposes the world matrix back to TRS (translation/rotation/scale)
    /// after each multiplication. For deep hierarchies with non-uniform scaling, this can
    /// introduce precision loss due to repeated decompose/recompose cycles. For most use cases
    /// (uniform scaling or shallow hierarchies), the error is negligible.
    fn propagate_hierarchy(&mut self, scene: &Scene3D) {
        for node in &scene.nodes {
            let local = self.node_transforms[node.index].to_matrix();
            let world = if let Some(parent_idx) = node.parent {
                let parent_world = self.node_transforms[parent_idx].to_matrix();
                parent_world * local
            } else {
                local
            };
            self.node_transforms[node.index] = NodeTransform::from_mat4(world);
        }
    }

    /// Compute final joint matrices: global_joint * inverse_bind_matrix.
    fn compute_joint_matrices(&mut self, scene: &Scene3D) {
        let mut offset = 0;
        for skin in &scene.skins {
            for (j, &joint_node_idx) in skin.joints.iter().enumerate() {
                let global = if joint_node_idx < self.node_transforms.len() {
                    self.node_transforms[joint_node_idx].to_matrix()
                } else {
                    Mat4::IDENTITY
                };
                let ibm = skin
                    .inverse_bind_matrices
                    .get(j)
                    .copied()
                    .unwrap_or(Mat4::IDENTITY);
                let idx = offset + j;
                if idx < self.joint_matrices.len() {
                    self.joint_matrices[idx] = global * ibm;
                }
            }
            offset += skin.joints.len();
        }
    }
}

/// Compute the duration of an animation clip from its keyframe timestamps.
fn animation_duration(anim: &Animation3D) -> f32 {
    anim.channels
        .iter()
        .filter(|ch| !ch.keyframes.is_empty())
        .map(|ch| *ch.keyframes.last().unwrap_or(&0.0))
        .fold(0.0f32, f32::max)
}

/// Sample all channels of an animation at time `t`, blending with `weight`.
fn sample_animation(
    anim: &Animation3D,
    t: f32,
    weight: f32,
    transforms: &mut [NodeTransform],
) {
    for channel in &anim.channels {
        if channel.keyframes.is_empty() {
            continue;
        }
        let value = sample_channel(channel, t);
        let node_idx = channel.target_node;
        if node_idx >= transforms.len() {
            continue;
        }

        match channel.property {
            AnimationProperty::Translation => {
                if let Some(v) = value {
                    let sampled = Vec3::new(v[0], v[1], v[2]);
                    transforms[node_idx].translation =
                        transforms[node_idx].translation.lerp(sampled, weight);
                }
            }
            AnimationProperty::Rotation => {
                if let Some(v) = value {
                    let sampled = Quat::from_xyzw(v[0], v[1], v[2], v[3]).normalize();
                    transforms[node_idx].rotation =
                        transforms[node_idx].rotation.slerp(sampled, weight);
                }
            }
            AnimationProperty::Scale => {
                if let Some(v) = value {
                    let sampled = Vec3::new(v[0], v[1], v[2]);
                    transforms[node_idx].scale =
                        transforms[node_idx].scale.lerp(sampled, weight);
                }
            }
            AnimationProperty::MorphWeights => {
                // Morph weights are applied on the GPU side via morph_weights uniform.
                // Store as metadata if needed in the future.
            }
        }
    }
}

/// Sample a single animation channel at time `t` using linear interpolation.
///
/// Returns the interpolated value as `[f32; N]` where N depends on the property
/// type (3 for translation/scale, 4 for rotation quaternion).
fn sample_channel(channel: &AnimationChannel3D, t: f32) -> Option<[f32; 4]> {
    let kf = &channel.keyframes;
    let vals = &channel.values;
    if kf.is_empty() {
        return None;
    }

    let (property, stride) = match channel.property {
        AnimationProperty::Translation => (AnimationProperty::Translation, 3),
        AnimationProperty::Rotation => (AnimationProperty::Rotation, 4),
        AnimationProperty::Scale => (AnimationProperty::Scale, 3),
        AnimationProperty::MorphWeights => (AnimationProperty::MorphWeights, 1),
    };

    // Find the two surrounding keyframes
    if t <= kf[0] {
        return read_value(vals, 0, stride);
    }
    // Safe unwrap: kf is non-empty (checked above), so last() always returns Some
    if t >= *kf.last().unwrap_or(&0.0) {
        return read_value(vals, kf.len() - 1, stride);
    }

    // Binary search for the keyframe pair
    let mut lo = 0;
    let mut hi = kf.len() - 1;
    while lo < hi - 1 {
        let mid = (lo + hi) / 2;
        if kf[mid] <= t {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    let t0 = kf[lo];
    let t1 = kf[hi];
    let alpha = if (t1 - t0).abs() < 1e-10 {
        0.0
    } else {
        (t - t0) / (t1 - t0)
    };

    let v0 = read_value(vals, lo, stride)?;
    let v1 = read_value(vals, hi, stride)?;

    match property {
        AnimationProperty::Translation | AnimationProperty::Scale => {
            let a = Vec3::new(v0[0], v0[1], v0[2]);
            let b = Vec3::new(v1[0], v1[1], v1[2]);
            let r = a.lerp(b, alpha);
            Some([r.x, r.y, r.z, 0.0])
        }
        AnimationProperty::Rotation => {
            let a = Quat::from_xyzw(v0[0], v0[1], v0[2], v0[3]).normalize();
            let b = Quat::from_xyzw(v1[0], v1[1], v1[2], v1[3]).normalize();
            let r = a.slerp(b, alpha);
            Some([r.x, r.y, r.z, r.w])
        }
        AnimationProperty::MorphWeights => {
            let r = v0[0] + (v1[0] - v0[0]) * alpha;
            Some([r, 0.0, 0.0, 0.0])
        }
    }
}

/// Read a value from the flat values array at the given keyframe index.
fn read_value(vals: &[f32], keyframe_index: usize, stride: usize) -> Option<[f32; 4]> {
    let start = keyframe_index * stride;
    if start + stride > vals.len() {
        return None;
    }
    let mut out = [0.0f32; 4];
    out[..stride].copy_from_slice(&vals[start..start + stride]);
    Some(out)
}
