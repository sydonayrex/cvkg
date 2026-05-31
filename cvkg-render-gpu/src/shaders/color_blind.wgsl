// Color blindness simulation post-process shader
// Applies Brettel/Viénot Daltonization matrix to the screen texture

// This file is included in the main WGSL_SRC concat in lib.rs

// Note: The shader functions are defined in the color_blindness module's
// shader_source() function and compiled separately as a dedicated pipeline.
// This file provides the shared uniforms and types for the WGSL concat.

// Color blind simulation uniforms (must match Rust ColorBlindUniforms layout)
// These are appended to SceneUniforms group(2) @binding(2)

// The actual fragment shader for color blindness simulation
// uses its own render pipeline to avoid interfering with the main pipeline.
