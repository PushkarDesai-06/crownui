use std::sync::Arc;

use geometry::{Color, Point, Rect};

use crate::ImageId;
use smallvec::SmallVec;

/// An app's runtime-stage fragment shader: the bytes Impeller's `impellerc` compiled from a
/// `.frag` (an `.iplr` file) for the engine version the renderer links. Renderers load each
/// `id` once, so every distinct program needs its own id.
#[derive(Debug, Clone, Copy)]
pub struct ShaderProgram {
    /// Names the program for the renderer's cache.
    pub id: u64,
    /// The compiled runtime stage.
    pub bytes: &'static [u8],
}

impl PartialEq for ShaderProgram {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// A color position along a gradient, `offset` in `0..=1`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradientStop {
    /// Position along the gradient.
    pub offset: f32,
    /// Color at that position.
    pub color: Color,
}

/// What fills a shape.
#[derive(Debug, Clone, PartialEq)]
pub enum Brush {
    /// A single color.
    Solid(Color),
    /// A linear gradient between two points in the shape's coordinates.
    LinearGradient {
        /// Gradient start.
        start: Point,
        /// Gradient end.
        end: Point,
        /// Color stops.
        stops: SmallVec<[GradientStop; 4]>,
    },
    /// An uploaded image scaled into `rect`, in the shape's coordinates; the shape crops it,
    /// so a rounded rectangle filled this way is a rounded image without any clip.
    Image {
        /// The image, as uploaded with `Renderer::upload_image`.
        image: ImageId,
        /// Where the whole image lands.
        rect: Rect,
    },
    /// Pixels shaded by an app's fragment program. The program sees `_fragCoord` in the
    /// shape's coordinates, samples `samplers` (uploaded images) in declaration order, and reads
    /// `uniforms` as 32-bit floats packed in declaration order. Renderers without programs, or
    /// missing a sampler, paint nothing.
    Shader {
        /// The compiled program.
        program: ShaderProgram,
        /// Images bound to the program's samplers, in order.
        samplers: SmallVec<[ImageId; 2]>,
        /// The program's uniforms, packed.
        uniforms: Arc<[u8]>,
    },
}

/// How a shape is painted.
#[derive(Debug, Clone, PartialEq)]
pub struct Paint {
    /// Fill.
    pub brush: Brush,
    /// Gaussian blur of the shape's edges (CSS `filter: blur` on the shape), in logical pixels.
    pub blur: Option<f32>,
}

/// How an open outline ends.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StrokeCap {
    /// Flat, flush with the endpoint.
    #[default]
    Butt,
    /// A half circle past the endpoint.
    Round,
    /// A half square past the endpoint.
    Square,
}

/// How two outline segments meet.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StrokeJoin {
    /// A sharp corner.
    #[default]
    Miter,
    /// A circular corner.
    Round,
    /// A flattened corner.
    Bevel,
}

/// Outline parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stroke {
    /// Line width in logical pixels.
    pub width: f32,
    /// How open ends finish.
    pub cap: StrokeCap,
    /// How segments meet.
    pub join: StrokeJoin,
}

impl Stroke {
    /// A `width` outline with butt caps and miter joins.
    pub const fn new(width: f32) -> Self {
        Self {
            width,
            cap: StrokeCap::Butt,
            join: StrokeJoin::Miter,
        }
    }

    /// A `width` outline with round caps and round joins, as Lucide icons are drawn.
    pub const fn rounded(width: f32) -> Self {
        Self {
            width,
            cap: StrokeCap::Round,
            join: StrokeJoin::Round,
        }
    }
}

impl Paint {
    /// A solid fill.
    pub fn solid(color: Color) -> Self {
        Self {
            brush: Brush::Solid(color),
            blur: None,
        }
    }

    /// A fill showing `image` scaled into `rect`.
    pub fn image(image: ImageId, rect: Rect) -> Self {
        Self {
            brush: Brush::Image { image, rect },
            blur: None,
        }
    }
}

impl From<Color> for Paint {
    fn from(color: Color) -> Self {
        Self::solid(color)
    }
}
