struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) color: vec3f,
};

@vertex
fn vertex_main(@builtin(vertex_index) vertexIndex: u32) -> VertexOutput {
    var pos = array<vec2f, 3>(
        vec2f(0.0, 0.5),
        vec2f(-0.5, -0.5),
        vec2f(0.5, -0.5)
    );
    var color = array<vec3f, 3>(
        vec3f(1.0, 0.0, 0.0),
        vec3f(0.0, 1.0, 0.0),
        vec3f(0.0, 0.0, 1.0)
    );
    return VertexOutput(
        vec4f(pos[vertexIndex], 0.0, 1.0),
        color[vertexIndex]
    );
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4f {
    return vec4f(in.color, 1.0);
}