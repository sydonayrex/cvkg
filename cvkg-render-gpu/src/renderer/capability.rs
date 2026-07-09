//! GPU capability detection. Replaces the compile-time constants in
//! `types.rs` (`MAX_VERTICES`, `MAX_INDICES`, `MAX_PARTICLES`) and the
//! hardcoded `texture_registry_capacity` / `texture_array_count` /
//! `mega_heim_size` / `gi_probe_count` numbers in `init.rs` with values
//! derived from the actual detected adapter at startup.

#[derive(Debug, Clone)]
pub struct GpuCapabilityTier {
    pub max_vertices: usize,
    pub max_indices: usize,
    pub max_particles: usize,
    pub texture_registry_capacity: usize,
    pub texture_array_count: u32,
    pub mega_heim_size: u32,
    pub gi_probe_count: usize,
    pub shadow_cascade_count: u32,
    pub enable_ssao: bool,
    pub enable_taa: bool,
    /// `required_limits` is what we hand to `adapter.request_device(...)` —
    /// intersected against `adapter.limits()` so request_device never panics
    /// on adapters that grant less than what we naively wrote before
    /// detection existed.
    pub required_limits: wgpu::Limits,
}

impl GpuCapabilityTier {
    /// Call once, right after the adapter is selected and before
    /// `adapter.request_device(...)`.
    pub fn detect(adapter: &wgpu::Adapter) -> Self {
        let info = adapter.get_info();
        let caps = adapter.limits();

        let is_low_power = matches!(
            info.device_type,
            wgpu::DeviceType::IntegratedGpu | wgpu::DeviceType::Cpu
        );
        // Storage buffer limit is the most reliable single signal for
        // "can this GPU handle the full deferred/GI/SSAO pipeline" —
        // low-end and older integrated GPUs frequently cap this well
        // below the 128MB a lot of discrete cards report.
        let storage_limited = caps.max_storage_buffer_binding_size < 64 * 1024 * 1024;

        let reduced = is_low_power || storage_limited;

        let (
            max_vertices,
            max_indices,
            max_particles,
            texture_registry_capacity,
            texture_array_count,
            mega_heim_size,
            gi_probe_count,
            shadow_cascade_count,
            enable_ssao,
            enable_taa,
        ) = if reduced {
            (
                25_000,
                37_500,
                8_192,
                15,
                16,
                2048,
                512,
                2,
                false,
                false,
            )
        } else {
            (
                100_000,
                150_000,
                65_536,
                31,
                32,
                4096,
                4096,
                4,
                true,
                true,
            )
        };

        // Intersect what we want against what the adapter actually supports.
        // request_device FAILS OUTRIGHT (not a silent clamp) if required_limits
        // exceeds adapter.limits() — this is what currently risks a hard panic
        // on weaker hardware via the unconditional .expect() at init.rs.
        let required_limits = wgpu::Limits {
            max_bindings_per_bind_group: caps.max_bindings_per_bind_group.min(256),
            max_binding_array_elements_per_shader_stage: caps
                .max_binding_array_elements_per_shader_stage
                .min(256),
            max_bind_groups: caps.max_bind_groups.min(8),
            max_storage_buffer_binding_size: caps.max_storage_buffer_binding_size,
            max_buffer_size: caps.max_buffer_size,
            ..wgpu::Limits::default()
        };

        tracing::info!(
            "[GPU] Capability tier: {} (device_type={:?}, storage_limit={}MB, max_bind_groups={})",
            if reduced { "reduced" } else { "full" },
            info.device_type,
            caps.max_storage_buffer_binding_size / (1024 * 1024),
            caps.max_bind_groups,
        );

        Self {
            max_vertices,
            max_indices,
            max_particles,
            texture_registry_capacity,
            texture_array_count,
            mega_heim_size,
            gi_probe_count,
            shadow_cascade_count,
            enable_ssao,
            enable_taa,
            required_limits,
        }
    }
}
