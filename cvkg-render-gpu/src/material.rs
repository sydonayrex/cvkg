//! Material graph -- composable shader generation.
//!
//! Replaces the mode-based `if/else` dispatch in shapes.wgsl with
//! composable material graphs that compile to WGSL at startup.
//!
//! # Architecture
//!
//! - `MaterialGraph` is a DAG of `MaterialNode`s connected by typed sockets.
//! - `MaterialCompiler` topologically sorts nodes and emits a WGSL fragment function.
//! - Built-in materials (rounded rect, glass, text, etc.) are pre-compiled at renderer init.
//! - User materials compile on first use and are cached by hash.

use std::collections::HashMap;

/// A socket type on a material node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialSocket {
    Color, // vec4<f32>
    Float, // f32
    Vec2,  // vec2<f32>
    Vec3,  // vec3<f32>
    Mask,  // f32 (0..1 coverage)
}

/// An operation node in the material graph.
#[derive(Debug, Clone)]
pub enum MaterialOp {
    /// Input: base color from vertex.
    /// Output: Color
    InputColor,

    /// Output: constant color from uniform.
    /// Parameters: rgba
    /// Output: Color
    ConstantColor {
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    },

    /// Input: UV from vertex.
    /// Output: sample result Color
    SampleTexture {
        tex_index: u32,
    },

    /// Premultiplied alpha blend (for font atlas).
    /// Inputs: color (Color), alpha (Float from texture)
    /// Output: Color
    PremultipliedBlend,

    /// SDF rounded rectangle mask.
    /// Inputs: none (reads vertex logical, size, radius)
    /// Output: Mask
    SDFRoundRect,

    /// SDF ellipse mask.
    /// Output: Mask
    SDFEllipse,

    /// Linear gradient between two colors.
    /// Input: t (Float, typically UV-based)
    /// Output: Color
    LinearGradient {
        start: [f32; 4],
        end: [f32; 4],
    },

    /// Radial gradient.
    /// Input: dist (Float)
    /// Output: Color
    RadialGradient {
        start: [f32; 4],
        end: [f32; 4],
    },

    /// Neon glow effect.
    /// Input: dist (Float), color (Color)
    /// Output: Color
    NeonGlow {
        radius: f32,
        intensity: f32,
    },

    /// Glass fresnel refraction.
    /// Inputs: uv (Vec2), blur_mip (Float)
    /// Output: Color
    GlassBlur,

    /// Layer two inputs with a blend mode.
    /// Inputs: bottom (Color), top (Color), opacity (Float)
    /// Output: Color
    LayerBlend {
        mode: BlendMode,
    },

    /// PBR lighting.
    /// Input: normal (Vec3), metallic (Float), roughness (Float), opacity (Float)
    /// Output: Color
    PBRLighting,

    /// Drop shadow.
    /// Inputs: uv (Vec2), size (Vec2), radius (Float)
    /// Output: Mask
    DropShadow,

    /// 9-slice UV remapping.
    /// Input: uv (Vec2)
    /// Output: Vec2
    NineSlice,

    /// Heatmap palette lookup.
    /// Input: value (Float)
    /// Output: Color
    Heatmap,

    /// Raymarched SDF shape.
    /// Output: Color
    Raymarch {
        shape: RaymarchShape,
    },

    Lightning,
    RuneGlow,
    RaymarchReflections,
    Stroke,
    DashedStroke,
}

#[derive(Debug, Clone, Copy)]
pub enum BlendMode {
    Add,
    Screen,
    Multiply,
    Overlay,
}

#[derive(Debug, Clone, Copy)]
pub enum RaymarchShape {
    Sphere,
    Box,
}

/// Connection between two nodes.
#[derive(Debug, Clone)]
pub struct MaterialEdge {
    pub from_node: u32,
    pub from_socket: MaterialSocket,
    pub to_node: u32,
    pub to_socket: MaterialSocket,
}

/// Index into the material graph's node list.
pub type MatNodeId = u32;

/// A directed acyclic graph of material operations.
#[derive(Debug, Clone)]
pub struct MaterialGraph {
    pub nodes: Vec<(MatNodeId, MaterialOp)>,
    pub edges: Vec<MaterialEdge>,
    pub output: Option<MatNodeId>,
}

impl MaterialGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            output: None,
        }
    }

    pub fn add_node(&mut self, op: MaterialOp) -> MatNodeId {
        let id = self.nodes.len() as MatNodeId;
        self.nodes.push((id, op));
        id
    }

    pub fn connect(
        &mut self,
        from: MatNodeId,
        from_socket: MaterialSocket,
        to: MatNodeId,
        to_socket: MaterialSocket,
    ) {
        self.edges.push(MaterialEdge {
            from_node: from,
            from_socket,
            to_node: to,
            to_socket,
        });
    }

    pub fn set_output(&mut self, node: MatNodeId) {
        self.output = Some(node);
    }

    /// Validate the graph using default (unrestricted) config.
    pub fn validate(&self) -> Result<(), MaterialError> {
        self.validate_with_config(&MaterialValidationConfig::default())
    }

    /// Validate the graph with strict limitations (e.g. for AI-generated graphs).
    pub fn validate_with_config(
        &self,
        config: &MaterialValidationConfig,
    ) -> Result<(), MaterialError> {
        if self.output.is_none() {
            return Err(MaterialError::NoOutput);
        }
        if self.nodes.len() > config.max_nodes {
            return Err(MaterialError::TooManyNodes(
                self.nodes.len(),
                config.max_nodes,
            ));
        }
        // P1-4 fix: also bound the edge count. Without this, a graph
        // with 1024 nodes but 100K edges (very dense) could cause
        // memory pressure and slow validation. The check is O(1).
        if self.edges.len() > config.max_edges {
            return Err(MaterialError::TooManyEdges(
                self.edges.len(),
                config.max_edges,
            ));
        }
        // Cycle detection via DFS
        let mut visited = vec![false; self.nodes.len()];
        let mut in_stack = vec![false; self.nodes.len()];

        for &(id, _) in &self.nodes {
            if !visited[id as usize] {
                self.dfs_check(id, &mut visited, &mut in_stack)?;
            }
        }

        // P2-10: Reachability check -- ensure every node is reachable from the output.
        // A node that is connected via an edge but whose input chain never reaches
        // the output would produce incomplete WGSL.
        if let Some(output_id) = self.output {
            let mut reachable = vec![false; self.nodes.len()];
            self.dfs_reachable(output_id, &mut reachable);
            for &(id, _) in &self.nodes {
                if !reachable[id as usize] {
                    return Err(MaterialError::UnreachableNode(id));
                }
            }
        }
        Ok(())
    }

    fn dfs_check(
        &self,
        node: MatNodeId,
        visited: &mut [bool],
        in_stack: &mut [bool],
    ) -> Result<(), MaterialError> {
        let idx = node as usize;
        if in_stack[idx] {
            return Err(MaterialError::Cycle);
        }
        if visited[idx] {
            return Ok(());
        }
        visited[idx] = true;
        in_stack[idx] = true;

        // Find all edges where this node is the consumer (to_node)
        for edge in &self.edges {
            if edge.to_node == node {
                self.dfs_check(edge.from_node, visited, in_stack)?;
            }
        }

        in_stack[idx] = false;
        Ok(())
    }

    /// P2-10: DFS backwards from the output node to find all reachable nodes.
    /// Edges go from producer (from_node) to consumer (to_node), so we walk
    /// backwards from to_node to from_node.
    fn dfs_reachable(&self, node: MatNodeId, reachable: &mut [bool]) {
        let idx = node as usize;
        if reachable[idx] {
            return;
        }
        reachable[idx] = true;
        // Find all edges where this node is the consumer (to_node)
        for edge in &self.edges {
            if edge.to_node == node {
                self.dfs_reachable(edge.from_node, reachable);
            }
        }
    }
}

impl Default for MaterialGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum MaterialError {
    NoOutput,
    Cycle,
    DisconnectedInput {
        node: MatNodeId,
        socket: MaterialSocket,
    },
    TypeMismatch {
        from: MaterialSocket,
        to: MaterialSocket,
    },
    CompileError(String),
    TooManyNodes(usize, usize),
    UnsupportedNodeType(String),
    /// P1-4 fix: graph has more edges than the configured limit.
    TooManyEdges(usize, usize),
    /// P2-10: node is not reachable from the output (dead subgraph).
    UnreachableNode(MatNodeId),
}

pub struct MaterialValidationConfig {
    pub max_nodes: usize,
    /// P1-4 fix: max number of edges in the material graph. Limits
    /// the complexity of the graph and prevents memory pressure from
    /// graphs with very high node-to-edge ratios. The default of
    /// 4096 corresponds to a max_nodes of 1024 with an average
    /// degree of 4, which is a reasonable upper bound for typical
    /// authoring tools.
    pub max_edges: usize,
}

impl Default for MaterialValidationConfig {
    fn default() -> Self {
        // P1-4: default to 4 edges per node as a reasonable upper
        // bound for typical material graphs. AI-generated or
        // untrusted graphs should use a stricter config (e.g.,
        // 512 nodes, 1024 edges) via validate_with_config.
        Self { max_nodes: 1024, max_edges: 4096 }
    }
}

impl std::error::Error for MaterialError {}

impl std::fmt::Display for MaterialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoOutput => write!(f, "material graph has no output node"),
            Self::Cycle => write!(f, "material graph contains a cycle"),
            Self::DisconnectedInput { node, socket } => {
                write!(f, "node {:?} missing input {:?}", node, socket)
            }
            Self::TypeMismatch { from, to } => {
                write!(f, "type mismatch: {:?} -> {:?}", from, to)
            }
            Self::CompileError(msg) => write!(f, "WGSL compilation error: {}", msg),
            Self::TooManyNodes(count, max) => write!(f, "too many nodes: {} (max {})", count, max),
            Self::UnsupportedNodeType(kind) => write!(f, "unsupported node type: {}", kind),
            Self::TooManyEdges(count, max) => write!(f, "too many edges: {} (max {})", count, max),
            Self::UnreachableNode(id) => write!(f, "unreachable node: {:?}", id),
        }
    }
}

/// Compiled material -- a WGSL function that can be included in the main shader.
#[derive(Debug, Clone)]
pub struct CompiledMaterial {
    /// The WGSL function body (everything between the `{` and `}` of the fragment function).
    pub wgsl_fn: String,
    /// The function name (unique per material).
    pub fn_name: String,
}

impl CompiledMaterial {
    pub fn hash_code(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.wgsl_fn.hash(&mut hasher);
        hasher.finish()
    }
}

/// Compiles MaterialGraph → WGSL fragment function.
pub struct MaterialCompiler;

impl MaterialCompiler {
    /// Compile a material graph into a WGSL function.
    ///
    /// The emitted function has the signature:
    ///
    /// ```text
    /// fn material_<id>(in: VertexOutput, col: vec4<f32>) -> vec4<f32>
    /// ```
    ///
    /// where `in` provides UV/position/size/etc. from the vertex output,
    /// and `col` is the base vertex color.
    pub fn compile(graph: &MaterialGraph) -> Result<CompiledMaterial, MaterialError> {
        graph.validate()?;

        // Topological sort
        let order = Self::topo_sort(graph)?;

        // Generate WGSL for each node in order
        let mut lines: Vec<String> = Vec::new();
        let mut var_names: HashMap<(MatNodeId, MaterialSocket), String> = HashMap::new();
        let mut next_var = 0;

        let mut mk_var = |prefix: &str| -> String {
            let v = format!("{}_{}", prefix, next_var);
            next_var += 1;
            v
        };

        for &node_id in &order {
            let (_, op) = &graph.nodes[node_id as usize];
            let result_var = mk_var("v");

            let expr = match op {
                MaterialOp::InputColor => {
                    "col".to_string()
                }
                MaterialOp::ConstantColor { r, g, b, a } => {
                    format!("vec4<f32>({:.6}, {:.6}, {:.6}, {:.6})", r, g, b, a)
                }
                MaterialOp::SampleTexture { tex_index } => {
                    format!(
                        "textureSample(t_diffuse[{}u], s_diffuse, in.uv)",
                        tex_index
                    )
                }
                MaterialOp::PremultipliedBlend => {
                    let color_var = Self::find_input(&var_names, node_id, MaterialSocket::Color, graph)
                        .unwrap_or_else(|| "col".to_string());
                    // Read alpha from a separate texture sample -- for fonts this is the single channel
                    let alpha_var = Self::find_input(&var_names, node_id, MaterialSocket::Float, graph)
                        .unwrap_or_else(|| "1.0".to_string());
                    format!(
                        "vec4<f32>(({}).rgb, ({}).a * ({}))",
                        color_var, color_var, alpha_var
                    )
                }
                MaterialOp::SDFRoundRect => {
                    let half = "in.size * 0.5";
                    format!(
                        r#"
    let _d = sd_round_rect(in.logical - {0}, {0} - in.radius, in.radius);
    let _aa = fwidth(_d);
    __RESULT__ = vec4<f32>(col.rgb, col.a * (1.0 - smoothstep(0.0, _aa, _d)));"#,
                        half
                    ).trim().to_string()
                }
                MaterialOp::SDFEllipse => {
                    let half = "in.size * 0.5";
                    format!(
                        r#"
    let _sh = max({0}, vec2<f32>(0.001));
    let _d = length((in.logical - {0}) / _sh) - 1.0;
    let _aa = fwidth(_d);
    __RESULT__ = vec4<f32>(col.rgb, col.a * (1.0 - smoothstep(0.0, _aa, _d)));"#,
                        half
                    ).trim().to_string()
                }
                MaterialOp::LinearGradient { start, end } => {
                    let t_var = Self::find_input(&var_names, node_id, MaterialSocket::Float, graph)
                        .unwrap_or_else(|| "in.uv.x".to_string());
                    format!(
                        "mix(vec4<f32>({:.6},{:.6},{:.6},{:.6}), vec4<f32>({:.6},{:.6},{:.6},{:.6}), clamp({}, 0.0, 1.0))",
                        start[0], start[1], start[2], start[3],
                        end[0], end[1], end[2], end[3],
                        t_var
                    )
                }
                MaterialOp::RadialGradient { start, end } => {
                    format!(
                        r#"
    let _dist = length(in.uv - 0.5) * 2.0;
    __RESULT__ = mix(vec4<f32>({:.6},{:.6},{:.6},{:.6}), vec4<f32>({:.6},{:.6},{:.6},{:.6}), clamp(_dist, 0.0, 1.0));"#,
                        start[0], start[1], start[2], start[3],
                        end[0], end[1], end[2], end[3],
                    ).trim().to_string()
                }
                MaterialOp::NeonGlow { radius, intensity } => {
                    let dist_var = Self::find_input(&var_names, node_id, MaterialSocket::Float, graph)
                        .unwrap_or_else(|| "length(in.logical - in.size * 0.5) / max(in.size.x, in.size.y)".to_string());
                    format!(
                        "vec4<f32>(col.rgb * exp(-{} * {:.6}), col.a)",
                        dist_var, intensity / radius.max(0.001)
                    )
                }
                MaterialOp::GlassBlur => {
                    r#"
    let uv = clamp(in.uv, vec2<f32>(0.0), vec2<f32>(1.0));
    let local = in.logical / in.size;
    let centered = local - vec2<f32>(0.5, 0.5);
    let lens_dir = normalize(centered + vec2<f32>(1e-5, 1e-5));
    let lens_dist = length(centered);
    let fresnel = pow(lens_dist * 1.8, 2.5);
    let lens = lens_dir * lens_dist * 0.08;
    let blur_mip = theme.glass_blur_strength;
    let env_base = textureSampleLevel(t_env, s_env, uv, blur_mip).rgb;
    let brightness = dot(env_base, vec3<f32>(0.299, 0.587, 0.114));
    var distortion = lens * 1.2;
    distortion *= (1.0 + brightness * 0.7);
    distortion *= 2.0;
    let ab_offset = distortion * 0.04;
    let r_sample = textureSampleLevel(t_env, s_env, uv + distortion + ab_offset * 1.2, blur_mip).r;
    let g_sample = textureSampleLevel(t_env, s_env, uv + distortion, blur_mip).g;
    let b_sample = textureSampleLevel(t_env, s_env, uv + distortion - ab_offset * 1.2, blur_mip).b;
    let refracted = vec3<f32>(r_sample, g_sample, b_sample);
    let tint = vec3<f32>(0.85, 0.9, 1.0);
    var final_rgb = refracted * tint;
    final_rgb += (brightness * 0.2) * (0.9 + vnoise(uv * 20.0 + scene.time * 3.0) * 0.1);
    let half_size = in.size * 0.5;
    let p_sdf = in.logical - half_size;
    let q_sdf = abs(p_sdf) - (half_size - in.radius);
    let d_sdf = length(max(q_sdf, vec2(0.0))) + min(max(q_sdf.x, q_sdf.y), 0.0) - in.radius;
    let d_norm = clamp(-d_sdf / 20.0, 0.0, 1.0);
    let flicker = 0.9 + vnoise(uv * 20.0 + scene.time * 3.0) * 0.1;
    final_rgb += smoothstep(1.0, 0.96, d_norm) * 0.25 * flicker * vec3<f32>(0.7, 1.0, 1.3);
    final_rgb -= smoothstep(0.96, 0.88, d_norm) * 0.15;
    let light_dir_h = normalize(vec2<f32>(-0.4, -0.8));
    let l = dot(uv, light_dir_h);
    final_rgb += smoothstep(0.45, 0.55, l) * 0.12;
    __RESULT__ = vec4<f32>(final_rgb, 0.02 + fresnel * 0.15) * (1.0 - smoothstep(-length(vec2(dpdx(in.logical.x), dpdy(in.logical.y))), length(vec2(dpdx(in.logical.x), dpdy(in.logical.y))), d_sdf));"#.trim().to_string()
                }
                MaterialOp::LayerBlend { mode } => {
                    let bottom = Self::find_input(&var_names, node_id, MaterialSocket::Color, graph)
                        .unwrap_or_else(|| "col".to_string());
                    let top = Self::find_input_map(&var_names, node_id, MaterialSocket::Color, graph, 1)
                        .unwrap_or_else(|| "col".to_string());
                    let opacity = Self::find_input(&var_names, node_id, MaterialSocket::Float, graph)
                        .unwrap_or_else(|| "1.0".to_string());
                    match mode {
                        BlendMode::Add => {
                            format!("mix({}, {}, {})", bottom, top, opacity)
                        }
                        BlendMode::Screen => {
                            format!("mix({}, 1.0 - (1.0 - {}) * (1.0 - {}), {})", bottom, bottom, top, opacity)
                        }
                        BlendMode::Multiply => {
                            format!("mix({}, {} * {}, {})", bottom, bottom, top, opacity)
                        }
                        BlendMode::Overlay => {
                            format!("mix({}, select(2.0 * {} * {}, 1.0 - 2.0 * (1.0 - {}) * (1.0 - {}), step(vec4<f32>(0.5), {})), {})", bottom, bottom, top, bottom, top, bottom, opacity)
                        }
                    }
                }
                MaterialOp::PBRLighting => {
                    r#"
    let _n = normalize(in.normal);
    let _metallic = in.slice.x;
    let _roughness = in.slice.y;
    let _opacity = in.slice.z;
    let _ld = normalize(vec3<f32>(0.5, 0.8, 0.6));
    let _lc = vec3<f32>(1.0, 0.95, 0.9);
    let _ndl = max(dot(_n, _ld), 0.0);
    let _diffuse = _ndl * _lc;
    let _vd = vec3<f32>(0.0, 0.0, 1.0);
    let _hd = normalize(_ld + _vd);
    let _ndh = max(dot(_n, _hd), 0.0);
    let _shiny = mix(8.0, 256.0, 1.0 - _roughness);
    let _spec = pow(_ndh, _shiny) * _lc;
    let _f0 = mix(vec3<f32>(0.04), col.rgb, _metallic);
    let _fresnel = _f0 + (vec3<f32>(1.0) - _f0) * pow(1.0 - max(dot(_n, -_vd), 0.0), 5.0);
    let _amb = vec3<f32>(0.06, 0.07, 0.1);
    var _lit = col.rgb * (_amb + _diffuse);
    _lit += _spec * mix(vec3<f32>(1.0), col.rgb, _metallic) * _fresnel;
    let _depth = in.clip_position.z;
    let _fog = clamp(1.0 - _depth * 0.0005, 0.7, 1.0);
    _lit *= _fog;
    __RESULT__ = vec4<f32>(_lit, col.a * _opacity);"#.trim().to_string()
                }
                MaterialOp::DropShadow => {
                    r#"
    let margin = in.uv.x;
    let blur = max(in.uv.y, 1.0);
    let original_size = in.size - 2.0 * margin;
    let half_size = original_size * 0.5;
    let p = in.logical - margin - half_size;
    let d_sdf = sd_round_rect(p, half_size - in.radius, in.radius);
    __RESULT__ = vec4<f32>(col.rgb, col.a * smoothstep(blur, 0.0, d_sdf));"#.trim().to_string()
                }
                MaterialOp::NineSlice => {
                    "col".to_string() // Passthrough: 9-slice UV remapping is resolved on CPU
                }
                MaterialOp::Heatmap => {
                    let val_var = Self::find_input(&var_names, node_id, MaterialSocket::Float, graph)
                        .unwrap_or_else(|| "textureSample(t_diffuse[0], s_diffuse, in.uv).r".to_string());
                    format!("vec4<f32>(heatmap_palette({}), col.a)", val_var)
                }
                MaterialOp::Raymarch { shape } => {
                    match shape {
                        RaymarchShape::Box => {
                            r#"
    let uv = (in.uv - 0.5) * 2.0;
    let ro = vec3<f32>(0.0, 0.0, -2.5);
    let rd = normalize(vec3<f32>(uv.x, uv.y, 1.5));
    let m = rotX(in.slice.x) * rotY(in.slice.y) * rotZ(in.slice.z);
    var t = 0.0;
    var hit = false;
    var d = 0.0;
    for (var i = 0; i < 40; i++) {
        let p = m * (ro + rd * t);
        d = sd_box_3d(p, vec3(0.5, 0.5, 0.5));
        if d < 0.001 {
            hit = true;
            break;
        }
        t += d;
        if t > 5.0 { break; }
    }
    if hit {
        let p = m * (ro + rd * t);
        let eps = vec2(0.001, 0.0);
        let n = normalize(vec3(
            sd_box_3d(p + eps.xyy, vec3(0.5)) - sd_box_3d(p - eps.xyy, vec3(0.5)),
            sd_box_3d(p + eps.yxy, vec3(0.5)) - sd_box_3d(p - eps.yxy, vec3(0.5)),
            sd_box_3d(p + eps.yyx, vec3(0.5)) - sd_box_3d(p - eps.yyx, vec3(0.5))
        ));
        let light_dir = normalize(vec3(1.0, 1.0, -2.0));
        let diff = max(dot(n, light_dir), 0.1);
        let rim = pow(1.0 - max(dot(n, -rd), 0.0), 3.0) * 0.5;
        __RESULT__ = vec4<f32>(col.rgb * diff + rim, col.a);
    } else {
        discard;
    }"#.trim().to_string()
                        }
                        RaymarchShape::Sphere => {
                            r#"
    let ro = vec3<f32>(in.uv * 2.0 - 1.0, -2.0);
    let rd = normalize(vec3<f32>(0.0, 0.0, 1.0));
    var t = 0.0;
    var hit = false;
    for (var i = 0; i < 32; i++) {
        let p = ro + rd * t;
        let d = length(p) - 1.0;
        if d < 0.01 { hit = true; break; }
        t += d;
    }
    if hit {
        let p = ro + rd * t;
        let n = normalize(p);
        let ld = normalize(vec3<f32>(1.0, 1.0, -1.0));
        let diff = max(dot(n, ld), 0.0);
        __RESULT__ = vec4<f32>(col.rgb * diff, col.a);
    } else {
        discard;
    }"#.trim().to_string()
                        }
                    }
                }
                MaterialOp::Lightning => {
                    r#"
    let d = length((in.uv - 0.5) * vec2<f32>(1.0, 4.0));
    __RESULT__ = theme.primary_neon * neon_glow(d, 0.01, 0.2);"#.trim().to_string()
                }
                MaterialOp::RuneGlow => {
                    r#"
    let p = (in.uv - 0.5) * 2.0;
    let d = min(sd_segment(p, vec2(-0.5, -0.8), vec2(0.5, 0.8)), sd_segment(p, vec2(0.5, -0.8), vec2(-0.5, 0.8)));
    __RESULT__ = theme.rune_glow * neon_glow(d, 0.02, 0.15) * theme.rune_opacity;"#.trim().to_string()
                }
                MaterialOp::RaymarchReflections => {
                    r#"
    let ro = vec3<f32>(in.uv.x - 0.5, in.uv.y - 0.5, -2.0);
    let rd = normalize(vec3<f32>(in.uv.x - 0.5, in.uv.y - 0.5, 1.0));
    let t = ray_march(ro, rd);
    if t > 0.0 {
        let p = ro + rd * t;
        let n = calc_normal(p);
        let light_dir = normalize(vec3<f32>(1.0, 1.0, -1.0));
        let diff = max(dot(n, light_dir), 0.2);
        let ref_rd = reflect(rd, n);
        let ref_t = ray_march(p + n * 0.01, ref_rd);
        var reflection_color = vec3<f32>(0.05, 0.05, 0.1);
        if ref_t > 0.0 { reflection_color = mix(theme.primary_neon.rgb, theme.shatter_neon.rgb, 0.5); }
        __RESULT__ = vec4<f32>(mix(col.rgb * diff, reflection_color, 0.3), 1.0);
    } else { discard; }"#.trim().to_string()
                }
                MaterialOp::Stroke => {
                    r#"
    let half_size = in.size * 0.5;
    let d = sd_round_rect(in.logical - half_size, half_size - in.radius, in.radius);
    let thickness = max(in.slice.x, 1.0);
    let fw = length(vec2(dpdx(in.logical.x), dpdy(in.logical.y)));
    __RESULT__ = vec4<f32>(col.rgb, col.a * (1.0 - smoothstep(-fw, fw, abs(d + thickness * 0.5) - thickness * 0.5)));"#.trim().to_string()
                }
                MaterialOp::DashedStroke => {
                    r#"
    let half_size = in.size * 0.5;
    let d = sd_round_rect(in.logical - half_size, half_size - in.radius, in.radius);
    let thickness = max(in.slice.x, 1.0);
    let perimeter = (in.uv.x + in.uv.y) * max(in.size.x, in.size.y);
    var alpha = 1.0 - smoothstep(-length(vec2(dpdx(in.logical.x), dpdy(in.logical.y))), length(vec2(dpdx(in.logical.x), dpdy(in.logical.y))), abs(d + thickness * 0.5) - thickness * 0.5);
    if (perimeter + scene.time * 20.0) % (max(in.slice.y, 1.0) + max(in.slice.z, 1.0)) > max(in.slice.y, 1.0) { alpha = 0.0; }
    __RESULT__ = vec4<f32>(col.rgb, col.a * alpha);"#.trim().to_string()
                }
            };

            if expr.contains("__RESULT__") {
                lines.push(format!("    var {}: vec4<f32>;", result_var));
                lines.push("    {".to_string());
                lines.push(expr.replace("__RESULT__", &result_var));
                lines.push("    }".to_string());
            } else {
                lines.push(format!("    var {} = {};", result_var, expr));
            }
            var_names.insert((node_id, MaterialSocket::Color), result_var);
        }

        let body = lines.join("\n");
        let out_id = graph.output.ok_or(MaterialError::NoOutput)?;
        let fn_name = "material_entry".to_string();

        let wgsl_fn = format!(
            "fn {}(in: VertexOutput, col: vec4<f32>) -> vec4<f32> {{\n{}\n    return v_{};\n}}",
            fn_name, body, out_id
        );

        Ok(CompiledMaterial { wgsl_fn, fn_name })
    }

    fn find_input(
        names: &HashMap<(MatNodeId, MaterialSocket), String>,
        node: MatNodeId,
        socket: MaterialSocket,
        graph: &MaterialGraph,
    ) -> Option<String> {
        for edge in &graph.edges {
            if edge.to_node == node && edge.to_socket == socket {
                return names.get(&(edge.from_node, edge.from_socket)).cloned();
            }
        }
        None
    }

    fn find_input_map(
        names: &HashMap<(MatNodeId, MaterialSocket), String>,
        node: MatNodeId,
        socket: MaterialSocket,
        graph: &MaterialGraph,
        offset: usize,
    ) -> Option<String> {
        let mut matches = graph
            .edges
            .iter()
            .filter(|e| e.to_node == node && e.to_socket == socket);
        let edge = matches.nth(offset)?;
        names.get(&(edge.from_node, edge.from_socket)).cloned()
    }

    fn topo_sort(graph: &MaterialGraph) -> Result<Vec<MatNodeId>, MaterialError> {
        let n = graph.nodes.len();
        let mut in_degree = vec![0u32; n];
        let mut adj: Vec<Vec<MatNodeId>> = vec![Vec::new(); n];

        for edge in &graph.edges {
            adj[edge.from_node as usize].push(edge.to_node);
            in_degree[edge.to_node as usize] += 1;
        }

        let mut queue: std::collections::VecDeque<MatNodeId> = std::collections::VecDeque::new();
        for (i, &deg) in in_degree.iter().enumerate() {
            if deg == 0 {
                queue.push_back(i as MatNodeId);
            }
        }

        let mut order = Vec::with_capacity(n);
        while let Some(node) = queue.pop_front() {
            order.push(node);
            for &next in &adj[node as usize] {
                in_degree[next as usize] -= 1;
                if in_degree[next as usize] == 0 {
                    queue.push_back(next);
                }
            }
        }

        if order.len() != n {
            return Err(MaterialError::Cycle);
        }

        Ok(order)
    }
}

/// Pre-built material graphs for the built-in modes.
/// These replace the if/else chains in shapes.wgsl.
pub mod builtins {
    use super::*;

    /// Build a rounded rectangle material (old mode 3).
    pub fn rounded_rect() -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let input = g.add_node(MaterialOp::InputColor);
        let sdf = g.add_node(MaterialOp::SDFRoundRect);
        // The SDF node reads vertex data directly; input color provides the base
        g.connect(input, MaterialSocket::Color, sdf, MaterialSocket::Color);
        g.set_output(sdf);
        g
    }

    /// Build a glass material (old mode 7).
    pub fn glass() -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let glass = g.add_node(MaterialOp::GlassBlur);
        g.set_output(glass);
        g
    }

    /// Build a solid color material (old mode 0 / default).
    pub fn solid() -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let input = g.add_node(MaterialOp::InputColor);
        g.set_output(input);
        g
    }

    /// Build a PBR material (old mode 13).
    pub fn pbr() -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let input = g.add_node(MaterialOp::InputColor);
        let pbr = g.add_node(MaterialOp::PBRLighting);
        g.connect(input, MaterialSocket::Color, pbr, MaterialSocket::Color);
        g.set_output(pbr);
        g
    }

    /// Build a text material (old mode 6) with premultiplied alpha.
    pub fn text(tex_index: u32) -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let input = g.add_node(MaterialOp::InputColor);
        let tex = g.add_node(MaterialOp::SampleTexture { tex_index });
        let blend = g.add_node(MaterialOp::PremultipliedBlend);
        g.connect(input, MaterialSocket::Color, blend, MaterialSocket::Color);
        g.connect(tex, MaterialSocket::Float, blend, MaterialSocket::Float);
        g.set_output(blend);
        g
    }

    /// Build a texture sample material (old mode 2).
    pub fn textured(tex_index: u32) -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let input = g.add_node(MaterialOp::InputColor);
        let tex = g.add_node(MaterialOp::SampleTexture { tex_index });
        let blend = g.add_node(MaterialOp::LayerBlend {
            mode: BlendMode::Multiply,
        });
        g.connect(input, MaterialSocket::Color, blend, MaterialSocket::Color);
        g.connect(tex, MaterialSocket::Color, blend, MaterialSocket::Color);
        g.set_output(blend);
        g
    }

    /// Build a neon glow material (old mode 8).
    pub fn neon_glow(radius: f32, intensity: f32) -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let input = g.add_node(MaterialOp::InputColor);
        let glow = g.add_node(MaterialOp::NeonGlow { radius, intensity });
        g.connect(input, MaterialSocket::Color, glow, MaterialSocket::Color);
        g.set_output(glow);
        g
    }

    /// Build a linear gradient material (old mode 15).
    pub fn linear_gradient(start: [f32; 4], end: [f32; 4]) -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let grad = g.add_node(MaterialOp::LinearGradient { start, end });
        g.set_output(grad);
        g
    }

    /// Build a radial gradient material (old mode 16).
    pub fn radial_gradient(start: [f32; 4], end: [f32; 4]) -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let grad = g.add_node(MaterialOp::RadialGradient { start, end });
        g.set_output(grad);
        g
    }

    /// Build an ellipse material (old mode 4).
    pub fn ellipse() -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let input = g.add_node(MaterialOp::InputColor);
        let sdf = g.add_node(MaterialOp::SDFEllipse);
        g.connect(input, MaterialSocket::Color, sdf, MaterialSocket::Color);
        g.set_output(sdf);
        g
    }

    /// Build a neon line material (old mode 1).
    pub fn neon_line() -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let color = g.add_node(MaterialOp::ConstantColor {
            r: 1.5,
            g: 1.5,
            b: 1.5,
            a: 1.0,
        });
        g.set_output(color);
        g
    }

    /// Build a heatmap material (old mode 12).
    pub fn heatmap(tex_index: u32) -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let tex = g.add_node(MaterialOp::SampleTexture { tex_index });
        let hm = g.add_node(MaterialOp::Heatmap);
        g.connect(tex, MaterialSocket::Float, hm, MaterialSocket::Float);
        g.set_output(hm);
        g
    }

    /// Build a 9-slice material (old mode 20).
    pub fn nine_slice(tex_index: u32) -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let input = g.add_node(MaterialOp::InputColor);
        let tex = g.add_node(MaterialOp::SampleTexture { tex_index });
        let blend = g.add_node(MaterialOp::LayerBlend {
            mode: BlendMode::Multiply,
        });
        g.connect(input, MaterialSocket::Color, blend, MaterialSocket::Color);
        g.connect(tex, MaterialSocket::Color, blend, MaterialSocket::Color);
        g.set_output(blend);
        g
    }

    /// Build a raymarched cube material (old mode 21).
    pub fn raymarch_cube() -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let input = g.add_node(MaterialOp::InputColor);
        let rm = g.add_node(MaterialOp::Raymarch {
            shape: RaymarchShape::Box,
        });
        g.connect(input, MaterialSocket::Color, rm, MaterialSocket::Color);
        g.set_output(rm);
        g
    }

    /// Build a stroke material (old mode 17).
    pub fn stroke() -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let input = g.add_node(MaterialOp::InputColor);
        let sdf = g.add_node(MaterialOp::Stroke);
        g.connect(input, MaterialSocket::Color, sdf, MaterialSocket::Color);
        g.set_output(sdf);
        g
    }

    /// Build a drop shadow material (old mode 18).
    pub fn drop_shadow() -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let input = g.add_node(MaterialOp::InputColor);
        let shadow = g.add_node(MaterialOp::DropShadow);
        g.connect(input, MaterialSocket::Color, shadow, MaterialSocket::Color);
        g.set_output(shadow);
        g
    }

    /// Build a dashed stroke material (old mode 19).
    pub fn dashed_stroke() -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let input = g.add_node(MaterialOp::InputColor);
        let sdf = g.add_node(MaterialOp::DashedStroke);
        g.connect(input, MaterialSocket::Color, sdf, MaterialSocket::Color);
        g.set_output(sdf);
        g
    }

    pub fn lightning() -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let l = g.add_node(MaterialOp::Lightning);
        g.set_output(l);
        g
    }

    pub fn rune_glow() -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let r = g.add_node(MaterialOp::RuneGlow);
        g.set_output(r);
        g
    }

    pub fn raymarch() -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let input = g.add_node(MaterialOp::InputColor);
        let rm = g.add_node(MaterialOp::RaymarchReflections);
        g.connect(input, MaterialSocket::Color, rm, MaterialSocket::Color);
        g.set_output(rm);
        g
    }
}

pub fn generate_builtins_wgsl() -> String {
    let mut out = String::new();
    out.push_str("// ── Auto-generated material functions (Runtime) ──\n\n");

    let builtins = vec![
        (0, "solid", builtins::solid()),
        (1, "neon_line", builtins::neon_line()),
        (2, "textured", builtins::textured(0)),
        (3, "rounded_rect", builtins::rounded_rect()),
        (4, "ellipse", builtins::ellipse()),
        (6, "text", builtins::text(0)),
        (7, "glass", builtins::glass()),
        (8, "neon_glow", builtins::neon_glow(1.0, 1.0)),
        (9, "lightning", builtins::lightning()),
        (10, "rune_glow", builtins::rune_glow()),
        (12, "heatmap", builtins::heatmap(0)),
        (13, "pbr", builtins::pbr()),
        (14, "raymarch", builtins::raymarch()),
        (
            15,
            "linear_grad",
            builtins::linear_gradient([0.0; 4], [0.0; 4]),
        ),
        (
            16,
            "radial_grad",
            builtins::radial_gradient([0.0; 4], [0.0; 4]),
        ),
        (17, "stroke", builtins::stroke()),
        (18, "drop_shadow", builtins::drop_shadow()),
        (19, "dashed", builtins::dashed_stroke()),
        (20, "nine_slice", builtins::nine_slice(0)),
        (21, "raymarch_cube", builtins::raymarch_cube()),
    ];

    let mut dispatch = String::new();
    dispatch.push_str(
        "fn dispatch_material(material_id: u32, in: VertexOutput, col: vec4<f32>) -> vec4<f32> {\n",
    );
    dispatch.push_str("    switch material_id {\n");

    for (id, name, graph) in builtins {
        let compiled = MaterialCompiler::compile(&graph).unwrap();
        let fn_name = format!("material_{}_{}", id, name);
        let fn_code = compiled.wgsl_fn.replace("material_entry", &fn_name);
        out.push_str(&fn_code);
        out.push_str("\n\n");

        dispatch.push_str(&format!(
            "        case {}u: {{ return {}(in, col); }}\n",
            id, fn_name
        ));
    }

    dispatch.push_str("        default: { return col; }\n");
    dispatch.push_str("    }\n}\n");

    out.push_str(&dispatch);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solid_material_compiles() {
        let graph = builtins::solid();
        let compiled = MaterialCompiler::compile(&graph).unwrap();
        assert!(compiled.wgsl_fn.contains("fn material_"));
        assert!(compiled.wgsl_fn.contains("col"));
    }

    #[test]
    fn test_rounded_rect_compiles() {
        let graph = builtins::rounded_rect();
        let compiled = MaterialCompiler::compile(&graph).unwrap();
        assert!(compiled.wgsl_fn.contains("sd_round_rect"));
    }

    #[test]
    fn test_pbr_compiles() {
        let graph = builtins::pbr();
        let compiled = MaterialCompiler::compile(&graph).unwrap();
        assert!(compiled.wgsl_fn.contains("PBRLighting") || compiled.wgsl_fn.contains("_n"));
    }

    #[test]
    fn test_graph_validation_no_output() {
        let mut g = MaterialGraph::new();
        g.add_node(MaterialOp::InputColor);
        assert!(g.validate().is_err());
    }

    #[test]
    fn test_graph_validation_cycle() {
        let mut g = MaterialGraph::new();
        let a = g.add_node(MaterialOp::InputColor);
        let b = g.add_node(MaterialOp::NeonGlow {
            radius: 1.0,
            intensity: 1.0,
        });
        g.connect(a, MaterialSocket::Color, b, MaterialSocket::Color);
        g.connect(b, MaterialSocket::Color, a, MaterialSocket::Color); // cycle!
        g.set_output(b);
        assert!(g.validate().is_err());
    }

    #[test]
    fn test_all_builtins_compile() {
        let graphs: Vec<MaterialGraph> = vec![
            builtins::solid(),
            builtins::rounded_rect(),
            builtins::glass(),
            builtins::pbr(),
            builtins::text(0),
            builtins::textured(0),
            builtins::neon_glow(4.0, 1.5),
            builtins::linear_gradient([1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]),
            builtins::radial_gradient([1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 1.0]),
            builtins::ellipse(),
            builtins::neon_line(),
            builtins::heatmap(0),
            builtins::nine_slice(0),
            builtins::raymarch_cube(),
            builtins::stroke(),
            builtins::drop_shadow(),
            builtins::dashed_stroke(),
        ];

        for (i, graph) in graphs.iter().enumerate() {
            match MaterialCompiler::compile(graph) {
                Ok(compiled) => {
                    assert!(
                        !compiled.wgsl_fn.is_empty(),
                        "graph {} produced empty WGSL",
                        i
                    );
                    assert!(
                        !compiled.fn_name.is_empty(),
                        "graph {} produced empty fn name",
                        i
                    );
                }
                Err(e) => {
                    panic!("graph {} failed to compile: {}", i, e);
                }
            }
        }
    }

    // =====================================================================
    // P1-4: Material graph complexity bounds (max edges)
    // =====================================================================

    #[test]
    fn p1_4_validate_rejects_too_many_edges() {
        // P1-4 regression: max_edges is enforced.
        let mut graph = MaterialGraph::new();
        // Set output
        graph.output = Some(0);
        // Add 3 nodes so we can add 2 edges.
        graph.add_node(MaterialOp::InputColor);
        graph.add_node(MaterialOp::InputColor);
        graph.add_node(MaterialOp::InputColor);
        // Add 2 edges.
        graph.connect(0, MaterialSocket::Color, 1, MaterialSocket::Color);
        graph.connect(1, MaterialSocket::Color, 2, MaterialSocket::Color);
        assert_eq!(graph.edges.len(), 2, "test setup: need 2 edges");
        // Configure max_edges=1, so 2 edges should be rejected.
        let config = MaterialValidationConfig { max_nodes: 1024, max_edges: 1 };
        let result = graph.validate_with_config(&config);
        assert!(matches!(result, Err(MaterialError::TooManyEdges(2, 1))),
                "expected TooManyEdges(2, 1), got {result:?}");
    }

    #[test]
    fn p1_4_default_config_has_max_edges() {
        // P1-4 regression: default config must have a non-zero
        // max_edges so the limit is actually enforced.
        let config = MaterialValidationConfig::default();
        assert!(config.max_edges > 0,
                "default max_edges must be > 0, got {}", config.max_edges);
    }

    #[test]
    fn p1_4_validate_accepts_graph_within_edge_limit() {
        // Small graph with edges under the default max_edges.
        let mut graph = MaterialGraph::new();
        graph.output = Some(0);
        graph.add_node(MaterialOp::InputColor);
        graph.add_node(MaterialOp::InputColor);
        graph.connect(0, MaterialSocket::Color, 1, MaterialSocket::Color);
        let result = graph.validate_with_config(&MaterialValidationConfig::default());
        // Should pass edge check (may fail other checks like NoOutput
        // if not all required connections are present, but should
        // not fail with TooManyEdges).
        if let Err(MaterialError::TooManyEdges(_, _)) = result {
            panic!("default config should accept 1 edge, got {result:?}");
        }
    }

    // P2-10: Test unreachable node detection
    #[test]
    fn p2_10_unreachable_node_detected() {
        let mut graph = MaterialGraph::new();
        let n0 = graph.add_node(MaterialOp::InputColor);
        let n1 = graph.add_node(MaterialOp::ConstantColor { r: 1.0, g: 0.0, b: 0.0, a: 1.0 });
        let n2 = graph.add_node(MaterialOp::ConstantColor { r: 0.0, g: 1.0, b: 0.0, a: 1.0 }); // unreachable
        graph.connect(n0, MaterialSocket::Color, n1, MaterialSocket::Color);
        graph.set_output(n1);
        // n2 is not connected to the output path
        let result = graph.validate();
        assert!(
            matches!(result, Err(MaterialError::UnreachableNode(id)) if id == n2),
            "expected UnreachableNode({n2}), got {result:?}"
        );
    }

    #[test]
    fn p2_10_all_reachable_passes() {
        let mut graph = MaterialGraph::new();
        let n0 = graph.add_node(MaterialOp::InputColor);
        let n1 = graph.add_node(MaterialOp::ConstantColor { r: 1.0, g: 0.0, b: 0.0, a: 1.0 });
        graph.connect(n0, MaterialSocket::Color, n1, MaterialSocket::Color);
        graph.set_output(n1);
        // Both nodes reachable from output
        assert!(graph.validate().is_ok(), "valid graph should pass");
    }
}
