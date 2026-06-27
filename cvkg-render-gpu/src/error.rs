use std::fmt;

#[derive(Debug)]
pub enum RenderError {
    DeviceLost(String),
    Surface(String),
    MaterialCompile { name: String, reason: String },
    ShaderValidation(String),
    UnsupportedFormat(wgpu::TextureFormat),
    VertexOverflow { needed: usize, max: usize },
    FrameAcquire(String),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::DeviceLost(msg) => {
                write!(f, "GPU device lost: {msg}. Try recreating the renderer.")
            }
            RenderError::Surface(msg) => {
                write!(f, "Surface error: {msg}. Check window state and GPU availability.")
            }
            RenderError::MaterialCompile { name, reason } => {
                write!(
                    f,
                    "Material compile failed for graph '{name}': {reason}. Validate node connections and types."
                )
            }
            RenderError::ShaderValidation(msg) => {
                write!(f, "Shader validation failed: {msg}. See inner WGSL error for line/column info.")
            }
            RenderError::UnsupportedFormat(fmt) => {
                write!(
                    f,
                    "Surface format {fmt:?} not supported by this adapter. Try a different backend."
                )
            }
            RenderError::VertexOverflow { needed, max } => {
                write!(
                    f,
                    "Vertex buffer overflow: needed {needed} vertices, max is {max}. Batch geometry or increase pool size."
                )
            }
            RenderError::FrameAcquire(msg) => {
                write!(f, "Failed to acquire next frame from surface: {msg}")
            }
        }
    }
}

impl std::error::Error for RenderError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_lost_includes_message_and_suggestion() {
        let err = RenderError::DeviceLost("GPU removed".into());
        let msg = err.to_string();
        assert!(msg.contains("GPU removed"), "should contain message");
        assert!(msg.contains("recreating"), "should suggest recreating");
    }

    #[test]
    fn material_compile_includes_name_and_reason() {
        let err = RenderError::MaterialCompile {
            name: "graph_0".into(),
            reason: "cycle detected".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("graph_0"), "should contain graph name");
        assert!(msg.contains("cycle detected"), "should contain reason");
        assert!(msg.contains("node connections"), "should suggest fix");
    }

    #[test]
    fn vertex_overflow_includes_counts() {
        let err = RenderError::VertexOverflow { needed: 100000, max: 65536 };
        let msg = err.to_string();
        assert!(msg.contains("100000"), "should contain needed count");
        assert!(msg.contains("65536"), "should contain max count");
    }

    #[test]
    fn shader_validation_includes_detail() {
        let err = RenderError::ShaderValidation("type mismatch at line 42".into());
        let msg = err.to_string();
        assert!(msg.contains("type mismatch"), "should contain detail");
        assert!(msg.contains("WGSL"), "should mention WGSL");
    }

    #[test]
    fn error_trait_satisfied() {
        let _boxed: Box<dyn std::error::Error> = Box::new(RenderError::FrameAcquire("test".into()));
    }
}
