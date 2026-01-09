struct Uniforms {
    view_port: vec2<u32>,
    cam_center: vec2<f32>,
    cam_half_size: vec2<f32>,
    num_bodies: u32,
    _pad: u32,
}

struct Body {
    position: vec2f,
    mass: f32,
    radius: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) world_position: vec2f,
    @location(1) body_center: vec2f,
    @location(2) body_radius: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read> bodies: array<Body>;

@vertex
fn vertex_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32
) -> VertexOutput {
    let body = bodies[instance_index];
    
    // create a quad from two triangles
    var quad_positions = array<vec2f, 6>(
        vec2f(-1.0, -1.0),
        vec2f( 1.0, -1.0),
        vec2f(-1.0,  1.0),
        vec2f(-1.0,  1.0),
        vec2f( 1.0, -1.0),
        vec2f( 1.0,  1.0),
    );
    
    let quad_pos = quad_positions[vertex_index];
    let world_pos = body.position + quad_pos * body.radius; // world pos of vertex
    
    // convert to ndc coords [-1, 1]
    let ndc = (world_pos - uniforms.cam_center) / uniforms.cam_half_size;
    
    var output: VertexOutput;
    output.clip_position = vec4f(ndc, 0.0, 1.0);
    output.world_position = world_pos;
    output.body_center = body.position;
    output.body_radius = body.radius;
    
    return output;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4f {
    // discard fragments outside the circle
    let r = in.world_position - in.body_center;
    let dist_squared = dot(r, r);
    if (dist_squared > in.body_radius * in.body_radius) {
        discard;
    }
    
    let color = vec3f(1.0, 1.0, 1.0);
    return vec4f(color, 1.0);
}