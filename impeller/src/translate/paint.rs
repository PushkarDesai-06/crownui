use graphics::{Brush, Paint, Stroke, StrokeCap, StrokeJoin};
use impellers::{
    BlurStyle, ColorSource, Context, DrawStyle, MaskFilter, Matrix, Texture, TextureSampling,
    TileMode,
};

use crate::{
    resources::DrawResources,
    util::{
        blur::filter_sigma,
        geometry::{to_color, to_point},
    },
};

pub(crate) fn fill_paint(
    paint: &Paint,
    resources: &DrawResources,
    context: Option<&Context>,
) -> impellers::Paint {
    let mut converted = impellers::Paint::default();
    match &paint.brush {
        Brush::Solid(color) => {
            converted.set_color(to_color(*color));
        }
        Brush::LinearGradient { stops, .. } if stops.is_empty() => {
            converted.set_color(impellers::Color::TRANSPARENT);
        }
        Brush::LinearGradient { start, end, stops } => {
            let colors: Vec<_> = stops.iter().map(|stop| to_color(stop.color)).collect();
            let offsets: Vec<_> = stops.iter().map(|stop| stop.offset).collect();
            let gradient = ColorSource::new_linear_gradient(
                to_point(*start),
                to_point(*end),
                &colors,
                &offsets,
                TileMode::Clamp,
                None,
            );
            converted.set_color_source(&gradient);
        }
        Brush::Image { image, rect } => match resources.textures.get(image) {
            Some(uploaded) if uploaded.width > 0 && uploaded.height > 0 => {
                let placement = Matrix::new_2d(
                    rect.size.width / uploaded.width as f32,
                    0.0,
                    0.0,
                    rect.size.height / uploaded.height as f32,
                    rect.origin.x,
                    rect.origin.y,
                );
                let source = ColorSource::new_image(
                    &uploaded.texture,
                    TileMode::Clamp,
                    TileMode::Clamp,
                    TextureSampling::Linear,
                    Some(&placement),
                );
                converted.set_color_source(&source);
            }
            _ => {
                converted.set_color(impellers::Color::TRANSPARENT);
            }
        },
        Brush::Shader {
            program,
            samplers,
            uniforms,
        } => {
            let program = resources
                .app_programs
                .get(&program.id)
                .and_then(Option::as_ref);
            let textures: Option<Vec<Texture>> = samplers
                .iter()
                .map(|id| {
                    resources
                        .textures
                        .get(id)
                        .map(|uploaded| uploaded.texture.clone())
                })
                .collect();
            match (context, program, textures) {
                (Some(context), Some(program), Some(textures)) => {
                    // SAFETY: the app compiled `program` with this engine's impellerc and packs
                    // `uniforms` as the program declares them (see `Brush::Shader`).
                    let source = unsafe {
                        context.new_color_source_from_fragment_program(program, &textures, uniforms)
                    };
                    converted.set_color_source(&source);
                }
                _ => {
                    converted.set_color(impellers::Color::TRANSPARENT);
                }
            }
        }
    }
    if let Some(sigma) = paint.blur.and_then(filter_sigma) {
        converted.set_mask_filter(&MaskFilter::new_blur(BlurStyle::Normal, sigma));
    }
    converted
}

pub(crate) fn stroke_paint(
    paint: &Paint,
    stroke: Stroke,
    resources: &DrawResources,
    context: Option<&Context>,
) -> impellers::Paint {
    let mut converted = fill_paint(paint, resources, context);
    converted
        .set_draw_style(DrawStyle::Stroke)
        .set_stroke_width(stroke.width)
        .set_stroke_cap(match stroke.cap {
            StrokeCap::Butt => impellers::StrokeCap::Butt,
            StrokeCap::Round => impellers::StrokeCap::Round,
            StrokeCap::Square => impellers::StrokeCap::Square,
        })
        .set_stroke_join(match stroke.join {
            StrokeJoin::Miter => impellers::StrokeJoin::Miter,
            StrokeJoin::Round => impellers::StrokeJoin::Round,
            StrokeJoin::Bevel => impellers::StrokeJoin::Bevel,
        });
    converted
}

pub(crate) fn opacity_paint(opacity: f32) -> impellers::Paint {
    let mut converted = impellers::Paint::default();
    converted.set_color(impellers::Color::new_srgba(
        0.0,
        0.0,
        0.0,
        opacity.clamp(0.0, 1.0),
    ));
    converted
}
