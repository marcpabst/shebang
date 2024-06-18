struct VertexOutput {
    @builtin(position) position_px: vec4<f32>,
    @location(0) position_org: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct ColourUniforms {
    _transform: mat4x4<f32>,
    _origin: vec2<f32>,
    _dimensions: vec2<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: ColourUniforms;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return uniforms.color;
}
