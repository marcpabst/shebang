use super::{geometry::Vector2, texture::Texture};
use std::hash::{Hash, Hasher};

/// An RGBA colour in the current colour space.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Colour {
    /// The red component of the colour.
    pub r: f32,
    /// The green component of the colour.
    pub g: f32,
    /// The blue component of the colour.
    pub b: f32,
    /// The alpha component of the colour.
    pub a: f32,
}

impl Colour {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl Hash for Colour {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.r.to_bits().hash(state);
        self.g.to_bits().hash(state);
        self.b.to_bits().hash(state);
        self.a.to_bits().hash(state);
    }
}

/// The type of gradient.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GradientType {
    /// A linear gradient.
    Linear,
    /// A radial gradient.
    Radial,
    /// A conic gradient.
    Conic,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GradientRepeatMode {
    /// Do not repeat the gradient.
    None,
    /// Repeat the gradient after the given length in pixels.
    Repeat(f32),
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextureStretchMode {
    /// Use the exact texture extents in pixels, starting from the top-left corner.
    Exact,
    /// Use the exact texture extents in pixels, starting from the center.
    ExactCenter,
    /// Stretch the texture to fit the shape.
    Stretch,
}

impl TextureStretchMode {
    pub fn u32_repr(&self) -> u32 {
        match self {
            Self::Exact => 0,
            Self::ExactCenter => 1,
            Self::Stretch => 2,
        }
    }
}

/// A material that defines how a shape should be rendered.
#[derive(Clone)]
pub enum Material {
    /// All pixels are the same colour.
    Color { color: Colour },
    /// Apply the given texture to the shape.
    Texture {
        texture: Texture,
        stretch: TextureStretchMode,
    },
    /// Apply the given gradient to the shape.
    Gradient {
        /// The type of gradient.
        gradient_type: GradientType,
        /// The colours of the gradient.
        colours: Vec<Colour>,
        /// The stops of the gradient (0.0 to 1.0)
        stops: Vec<f32>,
        /// The repeat mode of the gradient.
        repeat: GradientRepeatMode,
        /// The rotation of the gradient in degrees.
        rotation: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MaterialType {
    Color,
    Texture,
    Gradient,
}

impl Material {
    /// Returns the vertex shader module for this material.
    pub fn vertex_shader_module(&self, device: &wgpu::Device) -> wgpu::ShaderModule {
        device.create_shader_module(wgpu::include_wgsl!("shaders/vertex.wgsl"))
    }

    /// Returns the fragment shader module for this material.
    pub fn fragment_shader_module(&self, device: &wgpu::Device) -> wgpu::ShaderModule {
        match self {
            Self::Color { .. } => {
                device.create_shader_module(wgpu::include_wgsl!("shaders/colour.wgsl"))
            }
            Self::Texture { .. } => {
                device.create_shader_module(wgpu::include_wgsl!("shaders/texture.wgsl"))
            }
            Self::Gradient { .. } => {
                device.create_shader_module(wgpu::include_wgsl!("shaders/gradient.wgsl"))
            }
        }
    }

    /// Returns the material type.
    pub fn material_type(&self) -> MaterialType {
        match self {
            Self::Color { .. } => MaterialType::Color,
            Self::Texture { .. } => MaterialType::Texture,
            Self::Gradient { .. } => MaterialType::Gradient,
        }
    }

    /// Returns the texture for this material, if it has one.
    pub fn texture(&self) -> Option<&Texture> {
        match self {
            Self::Texture { texture, .. } => Some(texture),
            _ => None,
        }
    }

    /// Returns true if the material has a texture.
    pub fn has_texture(&self) -> bool {
        self.texture().is_some()
    }

    /// Returns the uniform buffer for this material.
    pub fn uniform_bytes(&self) -> Vec<u8> {
        match self {
            Self::Color { color } => bytemuck::bytes_of(color).to_vec(),
            Self::Texture { stretch, .. } => match stretch {
                TextureStretchMode::Exact => vec![stretch.u32_repr() as u8],
                TextureStretchMode::ExactCenter => vec![stretch.u32_repr() as u8],
                TextureStretchMode::Stretch => vec![stretch.u32_repr() as u8],
            },
            Self::Gradient { .. } => todo!(),
        }
    }

    /// Returns the size of the uniform buffer for this material.
    pub fn uniform_buffer_size(&self) -> usize {
        self.material_type().uniform_buffer_size()
    }
}

impl MaterialType {
    /// Returns the name of the material type.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Color => "Color",
            Self::Texture => "Texture",
            Self::Gradient => "Gradient",
        }
    }

    /// Returns the size of the uniform buffer for this material.
    pub fn uniform_buffer_size(&self) -> usize {
        match self {
            Self::Color { .. } => std::mem::size_of::<[f32; 4]>(),
            Self::Texture { .. } => std::mem::size_of::<[f32; 4]>(),
            Self::Gradient { .. } => std::mem::size_of::<[f32; 4]>(),
        }
    }

    /// Returns true if the material has a texture.
    pub fn has_texture(&self) -> bool {
        match self {
            Self::Texture => true,
            _ => false,
        }
    }
}
