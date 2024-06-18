struct VertexOutput {
    @builtin(position) position_px: vec4<f32>,
    @location(0) position_org: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct BBox {
    min: vec2<f32>,
    max: vec2<f32>,
};

struct TextureUniforms {
    _transform: mat4x4<f32>,
    bbox: BBox,
    mode: u32, // 0: exact, 1: exact center, 2: stretch
};

@group(0) @binding(0)
var<uniform> uniforms: TextureUniforms;

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_dims = textureDimensions(texture);

    // if exact mode, we don't need to do anything
    if (uniforms.mode == 0u) {
        let tex_coords = (in.position_org.xy - uniforms.bbox.min) / vec2<f32>(f32(tex_dims.x), f32(tex_dims.y));
        return textureSample(texture, texture_sampler, tex_coords);
    } else if (uniforms.mode == 1u) {
        let bbox_center = (uniforms.bbox.min + uniforms.bbox.max) / 2.0;
        let offset = in.position_org.xy - bbox_center - vec2<f32>(f32(tex_dims.x) / 2.0, f32(tex_dims.y) / 2.0);
        let tex_coords = (in.position_org.xy - uniforms.bbox.min + offset) / vec2<f32>(f32(tex_dims.x), f32(tex_dims.y));
        return textureSample(texture, texture_sampler, tex_coords);
    } else if (uniforms.mode == 2u) {

        // if stretch mode, we need to normalize the coordinates to [0, 1] based on the bounding box
        let tex_coords = (in.position_org - uniforms.bbox.min) / (uniforms.bbox.max - uniforms.bbox.min);

        return textureSample(texture, texture_sampler, tex_coords);
    }

    // return red if mode is invalid
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
