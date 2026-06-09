/// Identifies a built-in stitchable effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectId {
    Aurora,
    Glitch,
    Holographic,
    WaterRipple,
    Pixelate,
    ColorInvert,
    /// Frosted glass: standalone render pass using bcs_frosted fragment shader.
    /// Parameters: [frost_amount, grain_size, clear_radius, clear_softness]
    Frosted,
    Custom(u64), // For future dynamic extensions
}

/// A node in a post-process effect chain
#[derive(Debug, Clone)]
pub struct EffectNode {
    pub id: EffectId,
    pub parameters: Vec<f32>,
}

/// An effect chain that can be compiled into a single WGSL post-process pass.
#[derive(Default, Debug, Clone)]
pub struct EffectChain {
    pub nodes: Vec<EffectNode>,
}

impl EffectChain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, id: EffectId, params: &[f32]) {
        self.nodes.push(EffectNode {
            id,
            parameters: params.to_vec(),
        });
    }

    /// Hashes the chain to reuse compiled pipelines
    pub fn hash_code(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for node in &self.nodes {
            node.id.hash(&mut hasher);
            for p in &node.parameters {
                p.to_bits().hash(&mut hasher);
            }
        }
        hasher.finish()
    }
}

/// Registry that handles compiling EffectChains into executable WGSL fragment shaders.
pub struct EffectRegistry;

impl EffectRegistry {
    pub fn compile_chain(chain: &EffectChain) -> String {
        let mut wgsl = String::new();
        wgsl.push_str("// --- Auto-generated Stitchable Effect Chain ---\n");
        wgsl.push_str(
            "fn apply_effects(uv: vec2<f32>, col: vec4<f32>, time: f32) -> vec4<f32> {\n",
        );
        wgsl.push_str("    var c = col;\n");
        wgsl.push_str("    var u = uv;\n");

        for node in &chain.nodes {
            match node.id {
                EffectId::Aurora => {
                    wgsl.push_str("    // Aurora Effect (Placeholder)\n");
                    wgsl.push_str("    c.r += sin(u.x * 10.0 + time) * 0.1;\n");
                    wgsl.push_str("    c.g += cos(u.y * 10.0 + time) * 0.1;\n");
                }
                EffectId::Glitch => {
                    let intensity = node.parameters.first().unwrap_or(&1.0);
                    wgsl.push_str(&format!(
                        "    // Glitch Effect\n    u.x += sin(u.y * 50.0 + time * 10.0) * 0.01 * {:.3};\n",
                        intensity
                    ));
                }
                EffectId::Holographic => {
                    wgsl.push_str("    // Holographic Scanlines\n");
                    wgsl.push_str("    let scanline = sin(u.y * 200.0 - time * 5.0) * 0.1;\n");
                    wgsl.push_str("    c = c + vec4<f32>(scanline, scanline, scanline, 0.0);\n");
                }
                EffectId::ColorInvert => {
                    wgsl.push_str("    c = vec4<f32>(1.0 - c.rgb, c.a);");
                }
                EffectId::Frosted => {
                    // bcs_frosted is a @fragment shader, not a stitchable function.
                    // It runs as a separate fullscreen pass, not in compile_chain.
                    wgsl.push_str("    // Frosted: handled by separate pass\n");
                }
                _ => {
                    wgsl.push_str("    // Unimplemented effect bypass\n");
                }
            }
        }

        wgsl.push_str("    return c;\n");
        wgsl.push_str("}\n");
        wgsl
    }
}
