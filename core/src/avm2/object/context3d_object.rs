//! Object representation for Context3D objects

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, TObject};
use crate::bitmap::bitmap_data::BitmapRawData;
use crate::context::RenderContext;
use gc_arena::{Collect, Gc, GcWeak};
use ruffle_common::utils::HasPrefixField;
use ruffle_render::backend::{Context3D, Context3DCommand, Texture};
use ruffle_render::commands::CommandHandler;
use std::cell::Cell;
use std::rc::Rc;

use super::program_3d_object::Program3DObject;
use super::{IndexBuffer3DObject, Stage3DObject, VertexBuffer3DObject};

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Context3DObject<'gc>(pub Gc<'gc, Context3DData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct Context3DObjectWeak<'gc>(pub GcWeak<'gc, Context3DData<'gc>>);

impl<'gc> Context3DObject<'gc> {
    pub fn from_context(
        activation: &mut Activation<'_, 'gc>,
        context: Box<dyn Context3D>,
        stage3d: Stage3DObject<'gc>,
    ) -> Object<'gc> {
        let class = activation.avm2().classes().context3d;

        Context3DObject(Gc::new(
            activation.gc(),
            Context3DData {
                base: ScriptObjectData::new(class),
                render_context: Cell::new(Some(context)),
                stage3d,
            },
        ))
        .into()
    }

    pub fn stage3d(self) -> Stage3DObject<'gc> {
        self.0.stage3d
    }

    pub fn with_context_3d<R>(self, f: impl FnOnce(&mut dyn Context3D) -> R) -> R {
        // Temporarily take ownership of the Context3D instance.
        let cell = &self.0.render_context;
        let mut guard = scopeguard::guard(cell.take(), |stolen| cell.set(stolen));
        f(guard
            .as_deref_mut()
            .expect("Context3D is missing or already in use"))
    }

    pub fn upload_vertex_buffer_data(
        self,
        buffer: VertexBuffer3DObject<'gc>,
        data: &[u8],
        start_vertex: usize,
        data32_per_vertex: u8,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::UploadToVertexBuffer {
                buffer: buffer.handle(),
                data,
                start_vertex,
                data32_per_vertex,
            })
        });
    }

    pub fn upload_index_buffer_data(
        self,
        buffer: IndexBuffer3DObject<'gc>,
        data: &[u8],
        start_offset: usize,
    ) {
        let mut handle = buffer.handle();
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::UploadToIndexBuffer {
                buffer: &mut *handle,
                data,
                start_offset,
            })
        });
    }

    pub fn upload_shaders(
        self,
        program: Program3DObject<'gc>,
        vertex_shader_agal: Vec<u8>,
        fragment_shader_agal: Vec<u8>,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::UploadShaders {
                module: program.shader_module_handle(),
                vertex_shader_agal,
                fragment_shader_agal,
            })
        });
    }

    // Renders our finalized frame to the screen, as part of the Ruffle rendering process.
    pub fn render(self, context: &mut RenderContext<'_, 'gc>) {
        self.with_context_3d(|context3d| {
            if context3d.should_render() {
                let handle = context3d.bitmap_handle();

                context.commands.render_stage3d(
                    handle,
                    // FIXME - apply x and y translation from Stage3D
                    context.transform_stack.transform(),
                );
            }
        });
    }

    pub(crate) fn copy_bitmapdata_to_texture(
        self,
        source: &BitmapRawData<'gc>,
        dest: Rc<dyn Texture>,
        layer: u32,
    ) {
        // Note - Flash appears to allow a source that's larger than the destination.
        // Let's leave in this assertion to see if there any real SWFS relying on this
        // behavior.
        assert!(
            source.width() <= dest.width(),
            "Source width {:?} larger than dest width {:?}",
            source.width(),
            dest.width()
        );
        assert!(
            source.height() <= dest.height(),
            "Source height {:?} larger than dest height {:?}",
            source.height(),
            dest.height()
        );

        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::CopyBitmapToTexture {
                source: source.pixels_rgba(),
                source_width: source.width(),
                source_height: source.height(),
                dest,
                layer,
            })
        });
    }

    #[cfg_attr(not(feature = "jpegxr"), allow(unused))]
    pub(crate) fn copy_pixels_to_texture(self, source: Vec<u8>, dest: Rc<dyn Texture>, layer: u32) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::CopyBitmapToTexture {
                source: &source,
                source_width: dest.width(),
                source_height: dest.height(),
                dest,
                layer,
            })
        });
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct Context3DData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    #[collect(require_static)]
    render_context: Cell<Option<Box<dyn Context3D>>>,

    stage3d: Stage3DObject<'gc>,
}

impl<'gc> TObject<'gc> for Context3DObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}

impl std::fmt::Debug for Context3DObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Context3D")
    }
}
