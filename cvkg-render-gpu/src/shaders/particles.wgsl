struct Particle {
    pos_vel: vec4<f32>, // xy = pos, zw = vel
    color_life: vec4<f32>, // xyz = color, w = lifetime
};

struct ParticleBuffer {
    particles: array<Particle>,
};

struct ParticleUniforms {
    dt: f32,
    _pad: vec3<f32>,
};

@group(0) @binding(0) var<storage, read_write> particle_buf: ParticleBuffer;
@group(0) @binding(1) var<uniform> uniforms: ParticleUniforms;

@compute
@workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    let num_particles = arrayLength(&particle_buf.particles);
    if (idx >= num_particles) {
        return;
    }

    var p = particle_buf.particles[idx];

    // Simple Euler integration with proper delta time
    let dt = uniforms.dt;
    p.pos_vel.x = p.pos_vel.x + p.pos_vel.z * dt;
    p.pos_vel.y = p.pos_vel.y + p.pos_vel.w * dt;

    // Apply some drag
    p.pos_vel.z = p.pos_vel.z * 0.98;
    p.pos_vel.w = p.pos_vel.w * 0.98;

    // Decrease lifetime
    p.color_life.w = p.color_life.w - dt;
    if (p.color_life.w < 0.0) {
        p.color_life.w = 0.0;
    }

    particle_buf.particles[idx] = p;
}
