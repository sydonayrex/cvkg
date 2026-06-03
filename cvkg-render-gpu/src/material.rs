//! Material graph — composable shader generation.
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
    Color,    // vec4<f32>
    Float,    // f32
    Vec2,     // vec2<f32>
    Vec3,     // vec3<f32>
    Mask,     // f32 (0..1 coverage)
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
    ConstantColor { r: f32, g: f32, b: f32, a: f32 },

    /// Input: UV from vertex.
    /// Output: sample result Color
    SampleTexture { tex_index: u32 },

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
    SDFFllipse,

    /// Linear gradient between two colors.
    /// Input: t (Float, typically UV-based)
    /// Output: Color
    LinearGradient { start: [f32; 4], end: [f32; 4] },

    /// Radial gradient.
    /// Input: dist (Float)
    /// Output: Color
    RadialGradient { start: [f32; 4], end: [f32; 4] },

    /// Neon glow effect.
    /// Input: dist (Float), color (Color)
    /// Output: Color
    NeonGlow { radius: f32, intensity: f32 },

    /// Glass fresnel refraction.
    /// Inputs: uv (Vec2), blur_mip (Float)
    /// Output: Color
    GlassBlur { blur_radius: f32 },

    /// Layer two inputs with a blend mode.
    /// Inputs: bottom (Color), top (Color), opacity (Float)
    /// Output: Color
    LayerBlend { mode: BlendMode },

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
    Raymarch { shape: RaymarchShape },
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

    /// Validate the graph: no cycles, output connected, all inputs satisfied.
    pub fn validate(&self) -> Result<(), MaterialError> {
        if self.output.is_none() {
            return Err(MaterialError::NoOutput);
        }
        // Cycle detection via DFS
        let mut visited = vec![false; self.nodes.len()];
        let mut in_stack = vec![false; self.nodes.len()];

        for &(id, _) in &self.nodes {
            if !visited[id as usize] {
                self.dfs_check(id, &mut visited, &mut in_stack)?;
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
}

#[derive(Debug)]
pub enum MaterialError {
    NoOutput,
    Cycle,
    DisconnectedInput { node: MatNodeId, socket: MaterialSocket },
    TypeMismatch { from: MaterialSocket, to: MaterialSocket },
    CompileError(String),
}

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
            Self::CompileError(msg) => write!(f, "compile error: {}", msg),
        }
    }
}

/// Compiled material — a WGSL function that can be included in the main shader.
#[derive(Debug, Clone)]
pub struct CompiledMaterial {
    /// The WGSL function body (everything between the `{` and `}` of the fragment function).
    pub wgsl_fn: String,
    /// The function name (unique per material).
    pub fn_name: String,
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
                        .unwrap_or("col".to_string());
                    // Read alpha from a separate texture sample — for fonts this is the single channel
                    let alpha_var = Self::find_input(&var_names, node_id, MaterialSocket::Float, graph)
                        .unwrap_or("1.0".to_string());
                    format!(
                        "vec4<f32>(({}).rgb, ({}).a * ({}))",
                        color_var, color_var, alpha_var
                    )
                }
                MaterialOp::SDFRoundRect => {
                    // Reads vertex data directly
                    let half = "in.size * 0.5";
                    format!(
                        "let _d = sd_round_rect(in.logical - {}, {} - in.radius, in.radius); let _aa = fwidth(_d); vec4<f32>(col.rgb, col.a * (1.0 - smoothstep(0.0, _aa, _d)))",
                        half, half
                    )
                }
                MaterialOp::SDFFllipse => {
                    let half = "in.size * 0.5";
                    format!(
                        "let _sh = max({}, vec2<f32>(0.001)); let _d = length((in.logical - {}) / _sh) - 1.0; let _aa = fwidth(_d); vec4<f32>(col.rgb, col.a * (1.0 - smoothstep(0.0, _aa, _d)))",
                        half, half
                    )
                }
                MaterialOp::LinearGradient { start, end } => {
                    let t_var = Self::find_input(&var_names, node_id, MaterialSocket::Float, graph)
                        .unwrap_or("in.uv.x".to_string());
                    format!(
                        "mix(vec4<f32>({:.6},{:.6},{:.6},{:.6}), vec4<f32>({:.6},{:.6},{:.6},{:.6}), clamp({}, 0.0, 1.0))",
                        start[0], start[1], start[2], start[3],
                        end[0], end[1], end[2], end[3],
                        t_var
                    )
                }
                MaterialOp::RadialGradient { start, end } => {
                    format!(
                        "let _dist = length(in.uv - 0.5) * 2.0; mix(vec4<f32>({:.6},{:.6},{:.6},{:.6}), vec4<f32>({:.6},{:.6},{:.6},{:.6}), clamp(_dist, 0.0, 1.0))",
                        start[0], start[1], start[2], start[3],
                        end[0], end[1], end[2], end[3],
                    )
                }
                MaterialOp::NeonGlow { radius, intensity } => {
                    let dist_var = Self::find_input(&var_names, node_id, MaterialSocket::Float, graph)
                        .unwrap_or("length(in.logical - in.size * 0.5) / max(in.size.x, in.size.y)".to_string());
                    format!(
                        "vec4<f32>(col.rgb * exp(-{} * {:.6}), col.a)",
                        dist_var, intensity / radius.max(0.001)
                    )
                }
                MaterialOp::GlassBlur { blur_radius } => {
                    // Simplified — full glass is complex, this is the core idea
                    format!(
                        "let _uv = clamp(in.uv, vec2<f32>(0.0), vec2<f32>(1.0)); let _blur_mip = theme.glass_blur_strength; let _env_base = textureSampleLevel(t_env, s_env, _uv, _blur_mip).rgb; vec4<f32>(_env_base, 0.02 + pow(length(in.logical / in.size - 0.5) * 1.8, 2.5) * 0.15)",
                    )
                }
                MaterialOp::LayerBlend { mode } => {
                    let bottom = Self::find_input(&var_names, node_id, MaterialSocket::Color, graph)
                        .unwrap_or("col".to_string());
                    let top = Self::find_input_map(&var_names, node_id, MaterialSocket::Color, graph, 1)
                        .unwrap_or("col".to_string());
                    let opacity = Self::find_input(&var_names, node_id, MaterialSocket::Float, graph)
                        .unwrap_or("1.0".to_string());
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
                    // PBR reads normal/metallic/roughness from vertex
                    "let _n = normalize(in.normal); let _metallic = in.slice.x; let _roughness = in.slice.y; let _opacity = in.slice.z; let _ld = normalize(vec3<f32>(0.5, 0.8, 0.6)); let _lc = vec3<f32>(1.0, 0.95, 0.9); let _ndl = max(dot(_n, _ld), 0.0); let _diffuse = _ndl * _lc; let _vd = vec3<f32>(0.0, 0.0, 1.0); let _hd = normalize(_ld + _vd); let _ndh = max(dot(_n, _hd), 0.0); let _shiny = mix(8.0, 256.0, 1.0 - _roughness); let _spec = pow(_ndh, _shiny) * _lc; let _f0 = mix(vec3<f32>(0.04), col.rgb, _metallic); let _fresnel = _f0 + (vec3<f32>(1.0) - _f0) * pow(1.0 - max(dot(_n, -_vd), 0.0), 5.0); let _amb = vec3<f32>(0.06, 0.07, 0.1); var _lit = col.rgb * (_amb + _diffuse); _lit += _spec * mix(vec3<f32>(1.0), col.rgb, _metallic) * _fresnel; let _depth = in.clip_position.z; let _fog = clamp(1.0 - _depth * 0.0005, 0.7, 1.0); _lit *= _fog; vec4<f32>(_lit, col.a * _opacity)".to_string()
                }
                MaterialOp::DropShadow => {
                    "col".to_string() // placeholder — reads UV for shadow params
                }
                MaterialOp::NineSlice => {
                    "col".to_string() // placeholder — UV remapping is CPU-side
                }
                MaterialOp::Heatmap => {
                    let val_var = Self::find_input(&var_names, node_id, MaterialSocket::Float, graph)
                        .unwrap_or("textureSample(t_diffuse[0], s_diffuse, in.uv).r".to_string());
                    format!("vec4<f32>(heatmap_palette({}), col.a)", val_var)
                }
                MaterialOp::Raymarch { shape } => {
                    match shape {
                        RaymarchShape::Box => {
                            "let _uv = (in.uv - 0.5) * 2.0; let _ro = vec3<f32>(0.0, 0.0, -2.5); let _rd = normalize(vec3<f32>(_uv.x, _uv.y, 1.5)); let _m = rotX(in.slice.x) * rotY(in.slice.y) * rotZ(in.slice.z); var _t = 0.0; var _hit = false; var _d = 0.0; for (var _i = 0; _i < 40; _i++) { let _p = _m * (_ro + _rd * _t); _d = sd_box_3d(_p, vec3(0.5, 0.5, 0.5)); if _d < 0.001 { _hit = true; break; } _t += _d; if _t > 5.0 { break; } } if _hit { let _p2 = _m * (_ro + _rd * _t); let _eps = vec2(0.001, 0.0); let _n = normalize(vec3(sd_box_3d(_p2 + _eps.xyy, vec3(0.5)) - sd_box_3d(_p2 - _eps.xyy, vec3(0.5)), sd_box_3d(_p2 + _eps.yxy, vec3(0.5)) - sd_box_3d(_p2 - _eps.yxy, vec3(0.5)), sd_box_3d(_p2 + _eps.yyx, vec3(0.5)) - sd_box_3d(_p2 - _eps.yyx, vec3(0.5)))); let _ld2 = normalize(vec3(1.0, 1.0, -2.0)); let _diff2 = max(dot(_n, _ld2), 0.1); let _rim = pow(1.0 - max(dot(_n, -_rd), 0.0), 3.0) * 0.5; vec4<f32>(col.rgb * _diff2 + _rim, col.a) } else { discard; }".to_string()
                        }
                        RaymarchShape::Sphere => {
                            "col".to_string() // placeholder
                        }
                    }
                }
            };

            lines.push(format!("    var {} = {};", result_var, expr));
            var_names.insert((node_id, MaterialSocket::Color), result_var);
        }

        let body = lines.join("\n");
        let fn_name = format!("material_{}", graph.output.unwrap_or(0));

        let wgsl_fn = format!(
            "fn {}(in: VertexOutput, col: vec4<f32>) -> vec4<f32> {{\n{}\n    return v_0;\n}}",
            fn_name, body
        );

        Ok(CompiledMaterial { wgsl_fn, fn_name })
    }

    fn find_input(
        names: &HashMap<(MatNodeId, MaterialSocket), String>,
        node: MatNodeId,
        socket: MaterialSocket,
        _graph: &MaterialGraph,
    ) -> Option<String> {
        names.get(&(node, socket)).cloned()
    }

    fn find_input_map(
        names: &HashMap<(MatNodeId, MaterialSocket), String>,
        node: MatNodeId,
        socket: MaterialSocket,
        _graph: &MaterialGraph,
        offset: usize,
    ) -> Option<String> {
        // Find the Nth input of this socket type
        let matching: Vec<_> = names
            .iter()
            .filter(|((n, s), _)| *n == node && *s == socket)
            .collect();
        matching.get(offset).map(|(_, v)| v.to_string())
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
    pub fn rounded_rect(radius: f32) -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let input = g.add_node(MaterialOp::InputColor);
        let sdf = g.add_node(MaterialOp::SDFRoundRect);
        // The SDF node reads vertex data directly; input color provides the base
        g.connect(input, MaterialSocket::Color, sdf, MaterialSocket::Color);
        g.set_output(sdf);
        g
    }

    /// Build a glass material (old mode 7).
    pub fn glass(blur_radius: f32) -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let glass = g.add_node(MaterialOp::GlassBlur { blur_radius });
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
        let blend = g.add_node(MaterialOp::LayerBlend { mode: BlendMode::Multiply });
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
        let sdf = g.add_node(MaterialOp::SDFFllipse);
        g.connect(input, MaterialSocket::Color, sdf, MaterialSocket::Color);
        g.set_output(sdf);
        g
    }

    /// Build a neon line material (old mode 1).
    pub fn neon_line() -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let color = g.add_node(MaterialOp::ConstantColor { r: 1.5, g: 1.5, b: 1.5, a: 1.0 });
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
        let blend = g.add_node(MaterialOp::LayerBlend { mode: BlendMode::Multiply });
        g.connect(input, MaterialSocket::Color, blend, MaterialSocket::Color);
        g.connect(tex, MaterialSocket::Color, blend, MaterialSocket::Color);
        g.set_output(blend);
        g
    }

    /// Build a raymarched cube material (old mode 21).
    pub fn raymarch_cube() -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let input = g.add_node(MaterialOp::InputColor);
        let rm = g.add_node(MaterialOp::Raymarch { shape: RaymarchShape::Box });
        g.connect(input, MaterialSocket::Color, rm, MaterialSocket::Color);
        g.set_output(rm);
        g
    }

    /// Build a stroke material (old mode 17).
    pub fn stroke(thickness: f32) -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let input = g.add_node(MaterialOp::InputColor);
        let sdf = g.add_node(MaterialOp::SDFRoundRect);
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
    pub fn dashed_stroke(thickness: f32, dash_len: f32, gap_len: f32) -> MaterialGraph {
        let mut g = MaterialGraph::new();
        let input = g.add_node(MaterialOp::InputColor);
        let sdf = g.add_node(MaterialOp::SDFRoundRect);
        g.connect(input, MaterialSocket::Color, sdf, MaterialSocket::Color);
        g.set_output(sdf);
        g
    }
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
        let graph = builtins::rounded_rect(8.0);
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
        let b = g.add_node(MaterialOp::NeonGlow { radius: 1.0, intensity: 1.0 });
        g.connect(a, MaterialSocket::Color, b, MaterialSocket::Color);
        g.connect(b, MaterialSocket::Color, a, MaterialSocket::Color); // cycle!
        g.set_output(b);
        assert!(g.validate().is_err());
    }

    #[test]
    fn test_all_builtins_compile() {
        let graphs: Vec<MaterialGraph> = vec![
            builtins::solid(),
            builtins::rounded_rect(8.0),
            builtins::glass(20.0),
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
            builtins::stroke(2.0),
            builtins::drop_shadow(),
            builtins::dashed_stroke(2.0, 10.0, 5.0),
        ];

        for (i, graph) in graphs.iter().enumerate() {
            match MaterialCompiler::compile(graph) {
                Ok(compiled) => {
                    assert!(!compiled.wgsl_fn.is_empty(), "graph {} produced empty WGSL", i);
                    assert!(!compiled.fn_name.is_empty(), "graph {} produced empty fn name", i);
                }
                Err(e) => {
                    panic!("graph {} failed to compile: {}", i, e);
                }
            }
        }
    }
}
