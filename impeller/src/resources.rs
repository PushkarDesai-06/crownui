use std::collections::HashMap;

use graphics::{ImageId, ShaderProgram, TextLayoutId};
use impellers::{Context, FragmentProgram, Paragraph, Texture};

use crate::{
    segment_textures::SegmentTextures,
    util::{
        fragment_program::load_fragment_program,
        path_shader::load_path_program,
        shape_shader::{ShapeShader, load_shape_program},
    },
};

/// A GPU texture and its size in texels.
#[derive(Debug)]
pub(crate) struct UploadedTexture {
    pub(crate) texture: Texture,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

/// Paragraphs and textures that draw lists refer to by id, the rounded-shape and flattened-path
/// programs, and the segment textures of paths drawn with the latter.
#[derive(Debug, Default)]
pub(crate) struct DrawResources {
    pub(crate) paragraphs: HashMap<TextLayoutId, Paragraph>,
    pub(crate) textures: HashMap<ImageId, UploadedTexture>,
    pub(crate) shape_program: Option<FragmentProgram>,
    pub(crate) path_program: Option<FragmentProgram>,
    pub(crate) segment_textures: SegmentTextures,
    /// Apps' fragment programs by [`ShaderProgram::id`]; `None` when Impeller rejected one.
    pub(crate) app_programs: HashMap<u64, Option<FragmentProgram>>,
}

impl DrawResources {
    pub(crate) fn with_shader_programs() -> Self {
        Self {
            shape_program: load_shape_program(),
            path_program: load_path_program(),
            ..Self::default()
        }
    }

    /// The rounded-shape shader on `context`, when its program loaded.
    pub(crate) fn shape_shader<'gpu>(
        &'gpu self,
        context: &'gpu Context,
    ) -> Option<ShapeShader<'gpu>> {
        self.shape_program
            .as_ref()
            .map(|program| ShapeShader::new(context, program))
    }

    /// Loads an app's program the first time it is drawn.
    pub(crate) fn load_app_program(&mut self, program: &ShaderProgram) {
        self.app_programs
            .entry(program.id)
            .or_insert_with(|| load_fragment_program(program.bytes, "app shader"));
    }

    pub(crate) fn clear(&mut self) {
        self.paragraphs.clear();
        self.textures.clear();
        self.segment_textures.clear();
        self.app_programs.clear();
        self.shape_program = None;
        self.path_program = None;
    }
}
