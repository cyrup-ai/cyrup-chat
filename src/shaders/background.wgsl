// Animated background shader for CYRUP Chat
// Based on the Dioxus wgpu-texture example

struct TimeUniform {
    time: f32,
    _padding: vec3<f32>, // Ensure 16-byte alignment
};

@group(0) @binding(0)
var<uniform> time_uniform: TimeUniform;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) frag_position: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32
) -> VertexOutput {
    var output: VertexOutput;

    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0,  3.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0)
    );

    let pos = positions[vertex_index];
    output.position = vec4<f32>(pos.x, -pos.y, 0.0, 1.0);
    output.frag_position = pos;
    return output;
}

// Removed push constants to avoid WGPU feature requirements
// Will use static time animation instead

// Smooth noise function
fn hash(p: vec2<f32>) -> f32 {
    var h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    
    let a = hash(i);
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));
    
    let u = f * f * (3.0 - 2.0 * f);
    
    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

// Fractal Brownian Motion
fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 0.0;
    
    for (var i = 0; i < 5; i++) {
        value += amplitude * noise(p);
        p *= 2.0;
        amplitude *= 0.5;
    }
    
    return value;
}

// Create animated background with flowing gradients
fn render_background(uv: vec2<f32>, time: f32) -> vec4<f32> {
    // Create flowing motion
    let flow = vec2<f32>(
        sin(time * 0.3 + uv.y * 2.0) * 0.1,
        cos(time * 0.2 + uv.x * 3.0) * 0.1
    );
    
    let animated_uv = uv + flow;
    
    // Generate noise patterns
    let noise1 = fbm(animated_uv * 3.0 + time * 0.1);
    let noise2 = fbm(animated_uv * 2.0 - time * 0.05);
    
    // Create gradient base
    let gradient = smoothstep(0.0, 1.0, uv.y + sin(uv.x * 3.14159 + time * 0.2) * 0.1);
    
    // Mix colors - dark blue to purple gradient with animated highlights
    let color1 = vec3<f32>(0.1, 0.1, 0.18); // Dark blue #1a1a2e
    let color2 = vec3<f32>(0.06, 0.06, 0.12); // Darker blue #0f0f1e
    let color3 = vec3<f32>(0.2, 0.1, 0.3); // Purple highlights
    
    var base_color = mix(color1, color2, gradient);
    
    // Add animated highlights
    let highlight = noise1 * noise2;
    base_color = mix(base_color, color3, highlight * 0.3);
    
    // Add subtle shimmer
    let shimmer = sin(time * 2.0 + uv.x * 10.0 + uv.y * 8.0) * 0.02;
    base_color += shimmer;
    
    return vec4<f32>(base_color, 1.0);
}

@fragment
fn fs_main(@location(0) frag_position: vec2<f32>) -> @location(0) vec4<f32> {
    // Use uniform buffer for animated time
    let time = time_uniform.time;
    
    // Convert fragment position to UV coordinates
    let uv = (frag_position + 1.0) * 0.5;
    
    return render_background(uv, time);
}