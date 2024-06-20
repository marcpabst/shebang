#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextureUniforms {
    pub size_mode_x: u32,
    pub size_mode_y: u32,
    pub size_value_x: f32,
    pub size_value_y: f32,
}
