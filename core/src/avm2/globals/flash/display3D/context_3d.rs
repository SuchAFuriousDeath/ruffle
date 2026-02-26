use crate::avm2::Activation;
use crate::avm2::Error;
use crate::avm2::TObject as _;
use crate::avm2::Value;
use crate::avm2::error::make_error_2008;
use crate::avm2::error::{
    make_error_3669, make_error_3670, make_error_3671, make_error_3771, make_error_3772,
    make_error_3773, make_error_3780, make_error_3781,
};
use crate::avm2::globals::methods::flash_geom_matrix_3d as matrix3d_methods;
use crate::avm2::globals::slots::flash_geom_matrix_3d as matrix3d_slots;
use crate::avm2::globals::slots::flash_geom_rectangle as rectangle_slots;
use crate::avm2::object::{
    ClassObject, Context3DObject, IndexBuffer3DObject, Object, Program3DObject, TextureObject,
    VertexBuffer3DObject,
};
use crate::avm2_stub_method;
use crate::string::AvmString;
use ruffle_macros::{istr, native_methods};
use ruffle_render::backend::{
    BufferUsage, Context3DBlendFactor, Context3DCommand, Context3DCompareMode, Context3DMipFilter,
    Context3DProfile, Context3DStencilAction, Context3DTextureFilter, Context3DTextureFormat,
    Context3DTriangleFace, Context3DVertexBufferFormat, Context3DWrapMode, ProgramType, Texture,
};
use std::rc::Rc;
use swf::{Rectangle, Twips};

#[native_methods]
impl<'gc> Context3DObject<'gc> {
    fn create_index_buffer(
        self,
        activation: &mut Activation<'_, 'gc>,
        num_indices: u32,
    ) -> Result<IndexBuffer3DObject<'gc>, Error<'gc>> {
        // FIXME - get bufferUsage and pass it through
        if num_indices == 0 {
            return Err(make_error_3671(activation));
        }

        let index_buffer = self.create_index_buffer_internal(num_indices, activation);

        Ok(index_buffer)
    }

    fn create_vertex_buffer(
        self,
        activation: &mut Activation<'_, 'gc>,
        num_vertices: u32,
        data_32_per_vertex: u32,
    ) -> Result<VertexBuffer3DObject<'gc>, Error<'gc>> {
        // FIXME - get bufferUsage and pass it through
        if data_32_per_vertex > 64 {
            return Err(make_error_3670(activation));
        } else if data_32_per_vertex == 0 {
            return Err(make_error_3671(activation));
        }

        let vertex_buffer = self.create_vertex_buffer_internal(
            num_vertices,
            data_32_per_vertex as u8,
            BufferUsage::DynamicDraw,
            activation,
        );

        Ok(vertex_buffer)
    }

    #[expect(clippy::too_many_arguments)]
    fn configure_back_buffer(
        self,
        activation: &mut Activation<'_, 'gc>,
        width: u32,
        height: u32,
        anti_alias: u32,
        enable_depth_and_stencil: bool,
        wants_best_resolution: bool,
        wants_best_resolution_on_browser_zoom: bool,
    ) -> Result<(), Error<'gc>> {
        let old_swf = activation.context.root_swf.version() < 30;

        if old_swf && width == 0 && height == 0 && anti_alias == 0 && !enable_depth_and_stencil {
            return Ok(());
        }

        if width < 32 || width > 16384 {
            return Err(if old_swf {
                make_error_3669(activation)
            } else {
                make_error_3780(activation)
            });
        }

        if height < 32 || height > 16384 {
            return Err(if old_swf {
                make_error_3669(activation)
            } else {
                make_error_3781(activation)
            });
        }

        if wants_best_resolution {
            avm2_stub_method!(
                activation,
                "flash.display3D.Context3D",
                "configureBackBuffer",
                "wantsBestResolution"
            );
        }
        if wants_best_resolution_on_browser_zoom {
            avm2_stub_method!(
                activation,
                "flash.display3D.Context3D",
                "configureBackBuffer",
                "wantsBestResolutionOnBrowserZoom"
            );
        }

        self.configure_back_buffer_internal(
            width,
            height,
            anti_alias,
            enable_depth_and_stencil,
            wants_best_resolution,
            wants_best_resolution_on_browser_zoom,
        );

        Ok(())
    }

    fn create_program(self, activation: &mut Activation<'_, 'gc>) -> Program3DObject<'gc> {
        self.create_program_internal(activation)
    }

    fn set_program(
        self,
        _activation: &mut Activation<'_, 'gc>,
        program: Option<Program3DObject<'gc>>,
    ) {
        self.set_program_internal(program);
    }

    fn draw_triangles(
        self,
        _activation: &mut Activation<'_, 'gc>,
        #[name = "indexBuffer"] index_buffer: IndexBuffer3DObject<'gc>,
        first_index: u32,
        num_triangles: u32,
    ) {
        self.draw_triangles_internal(index_buffer, first_index, num_triangles as i32);
    }

    fn present(self, _activation: &mut Activation<'_, 'gc>) {
        self.present_internal();
    }

    fn get_profile(self, activation: &mut Activation<'_, 'gc>) -> AvmString<'gc> {
        match self.with_context_3d(|context| context.profile()) {
            Context3DProfile::Baseline => istr!("baseline"),
            Context3DProfile::BaselineConstrained => istr!("baselineConstrained"),
            Context3DProfile::BaselineExtended => istr!("baselineExtended"),
            Context3DProfile::Standard => istr!("standard"),
            Context3DProfile::StandardConstrained => istr!("standardConstrained"),
            Context3DProfile::StandardExtended => istr!("standardExtended"),
        }
    }

    fn set_culling(
        self,
        _activation: &mut Activation<'_, 'gc>,
        #[name = "triangleFaceToCull"] culling: Context3DTriangleFace,
    ) {
        self.set_culling_internal(culling);
    }

    fn set_program_constants_from_matrix(
        self,
        activation: &mut Activation<'_, 'gc>,
        #[name = "programType"] program_type: ProgramType,
        first_register: u32,
        #[name = "matrix"] matrix: Object<'gc>,
        user_transposed_matrix: bool,
    ) -> Result<(), Error<'gc>> {
        let mut matrix = matrix;

        // Hack - we store in column-major form, but we need it in row-major form
        // So, do the *opposite* of what the user pasess in`
        // or that's what I thought, but doing this seems to work???
        //
        // It seems like the documentation is wrong - we really copy to the registers
        // in column-major order.
        // See https://github.com/openfl/openfl/blob/971a4c9e43b5472fd84d73920a2b7c1b3d8d9257/src/openfl/display3D/Context3D.hx#L1532-L1550
        if user_transposed_matrix {
            matrix = Value::from(matrix)
                .call_method(matrix3d_methods::CLONE, &[], activation)?
                .as_object()
                .expect("Matrix3D.clone returns Object");

            Value::from(matrix).call_method(matrix3d_methods::TRANSPOSE, &[], activation)?;
        }

        let matrix_raw_data = matrix
            .get_slot(matrix3d_slots::_RAW_DATA)
            .as_object()
            .expect("rawData cannot be null");

        let matrix_raw_data = matrix_raw_data
            .as_vector_storage()
            .unwrap()
            .iter()
            .map(|val| val.as_f64() as f32)
            .collect::<Vec<f32>>();

        self.set_program_constants_internal(program_type, first_register, matrix_raw_data);

        Ok(())
    }

    fn set_program_constants_from_vector(
        self,
        activation: &mut Activation<'_, 'gc>,
        #[name = "programType"] program_type: ProgramType,
        first_register: u32,
        #[name = "vector"] vector: Object<'gc>,
        num_registers: i32,
    ) -> Result<(), Error<'gc>> {
        let vector = vector.as_vector_storage().unwrap();

        let to_take = if num_registers != -1 {
            let required = num_registers as usize * 4;

            if vector.length() < required {
                return Err(make_error_3669(activation));
            }

            required
        } else {
            vector.length()
        };

        let raw_data = vector
            .iter()
            .map(|val| val.as_f64() as f32)
            .take(to_take)
            .collect::<Vec<f32>>();

        self.set_program_constants_internal(program_type, first_register, raw_data);

        Ok(())
    }

    fn set_program_constants_from_byte_array(
        self,
        activation: &mut Activation<'_, 'gc>,
        #[name = "programType"] program_type: ProgramType,
        first_register: u32,
        num_registers: i32,
        #[name = "data"] data: Object<'gc>,
        byte_offset: u32,
    ) -> Result<(), Error<'gc>> {
        let data = data.as_bytearray().expect("Parameter must be a ByteArray");
        let byte_offset = byte_offset as usize;

        // Negative numRegisters is invalid for ByteArray (unlike Vector which treats -1 as "use all")
        let num_registers =
            usize::try_from(num_registers).map_err(|_| make_error_3669(activation))?;

        let required_bytes = num_registers * 16;
        let data_len = data.len();

        if byte_offset >= data_len || data_len - byte_offset < required_bytes {
            return Err(make_error_3669(activation));
        }

        let num_floats = num_registers * 4;
        let mut raw_data = Vec::with_capacity(num_floats);

        for i in 0..num_floats {
            let float_offset = byte_offset + i * 4;
            let value = data
                .read_float_at(float_offset)
                .expect("Already validated bounds");
            raw_data.push(value);
        }

        self.set_program_constants_internal(program_type, first_register, raw_data);

        Ok(())
    }

    #[expect(clippy::too_many_arguments)]
    fn clear(
        self,
        _activation: &mut Activation<'_, 'gc>,
        red: f64,
        green: f64,
        blue: f64,
        alpha: f64,
        depth: f64,
        stencil: u32,
        mask: u32,
    ) {
        self.clear_internal(red, green, blue, alpha, depth, stencil, mask);
    }

    fn create_texture(
        self,
        activation: &mut Activation<'_, 'gc>,
        width: i32,
        height: i32,
        #[name = "textureFormat"] format: Context3DTextureFormat,
        optimize_for_render_to_texture: bool,
        streaming_levels: i32,
    ) -> Result<TextureObject<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().texture;

        self.create_texture_internal(
            width as u32,
            height as u32,
            format,
            optimize_for_render_to_texture,
            streaming_levels as u32,
            class,
            activation,
        )
    }

    fn create_rectangle_texture(
        self,
        activation: &mut Activation<'_, 'gc>,
        width: i32,
        height: i32,
        #[name = "textureFormat"] format: Context3DTextureFormat,
        optimize_for_render_to_texture: bool,
    ) -> Result<TextureObject<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().rectangletexture;

        self.create_texture_internal(
            width as u32,
            height as u32,
            format,
            optimize_for_render_to_texture,
            0,
            class,
            activation,
        )
    }

    fn create_cube_texture(
        self,
        activation: &mut Activation<'_, 'gc>,
        size: i32,
        #[name = "textureFormat"] format: Context3DTextureFormat,
        optimize_for_render_to_texture: bool,
        streaming_levels: i32,
    ) -> Result<TextureObject<'gc>, Error<'gc>> {
        self.create_cube_texture_internal(
            size as u32,
            format,
            optimize_for_render_to_texture,
            streaming_levels as u32,
            activation,
        )
    }

    fn set_texture_at(
        self,
        activation: &mut Activation<'_, 'gc>,
        sampler: i32,
        texture_object: Option<TextureObject<'gc>>,
    ) {
        let mut cube = false;

        let texture = if let Some(texture_object) = texture_object {
            cube = texture_object.is_of_type(
                activation
                    .avm2()
                    .classes()
                    .cubetexture
                    .inner_class_definition(),
            );

            Some(texture_object.handle())
        } else {
            None
        };

        self.set_texture_at_internal(sampler as u32, texture, cube);
    }

    fn set_color_mask(
        self,
        _activation: &mut Activation<'_, 'gc>,
        red: bool,
        green: bool,
        blue: bool,
        alpha: bool,
    ) {
        self.set_color_mask_internal(red, green, blue, alpha);
    }

    fn set_depth_test(
        self,
        _activation: &mut Activation<'_, 'gc>,
        depth_mask: bool,
        #[name = "passCompareMode"] pass_compare_mode: Context3DCompareMode,
    ) {
        self.set_depth_test_internal(depth_mask, pass_compare_mode);
    }

    fn set_blend_factors(
        self,
        _activation: &mut Activation<'_, 'gc>,
        #[name = "sourceFactor"] source_factor: Context3DBlendFactor,
        #[name = "destinationFactor"] destination_factor: Context3DBlendFactor,
    ) {
        self.set_blend_factors_internal(source_factor, destination_factor);
    }

    fn set_render_to_texture(
        self,
        activation: &mut Activation<'_, 'gc>,
        #[name = "texture"] texture: TextureObject<'gc>,
        enable_depth_and_stencil: bool,
        anti_alias: u32,
        surface_selector: u32,
        color_output_index: u32,
    ) -> Result<(), Error<'gc>> {
        if texture.instance_class() == activation.avm2().class_defs().cubetexture {
            if surface_selector > 5 {
                return Err(make_error_3772(activation));
            }
        } else if texture.instance_class() == activation.avm2().class_defs().rectangletexture {
            if surface_selector != 0 {
                return Err(make_error_3773(activation));
            }
        } else {
            // normal Texture or video texture (but the latter should probably not be supported here anyway)
            if surface_selector != 0 {
                return Err(make_error_3771(activation));
            }
        }

        if anti_alias != 0 {
            avm2_stub_method!(
                activation,
                "flash.display3D.Context3D",
                "setRenderToTexture",
                "antiAlias != 0"
            );
        }

        if color_output_index != 0 {
            avm2_stub_method!(
                activation,
                "flash.display3D.Context3D",
                "setRenderToTexture",
                "colorOutputIndex != 0"
            );
        }

        self.set_render_to_texture_internal(
            texture.handle(),
            enable_depth_and_stencil,
            anti_alias,
            surface_selector,
        );

        Ok(())
    }

    fn set_stencil_actions(
        self,
        _activation: &mut Activation<'_, 'gc>,
        #[name = "triangleFace"] triangle_face: Context3DTriangleFace,
        #[name = "compareMode"] compare_mode: Context3DCompareMode,
        #[name = "actionOnBothPass"] on_both_pass: Context3DStencilAction,
        #[name = "actionOnDepthFail"] on_depth_fail: Context3DStencilAction,
        #[name = "actionOnDepthPassStencilFail"] on_depth_pass_stencil_fail: Context3DStencilAction,
    ) {
        self.set_stencil_actions_internal(
            triangle_face,
            compare_mode,
            on_both_pass,
            on_depth_fail,
            on_depth_pass_stencil_fail,
        );
    }

    fn set_render_to_back_buffer(self, _activation: &mut Activation<'_, 'gc>) {
        self.set_render_to_back_buffer_internal();
    }

    // TODO: Add visual tests for setStencilReferenceValue edge cases
    // (e.g. values > 255 for reference/masks on an 8-bit stencil buffer).
    fn set_stencil_reference_value(
        self,
        _activation: &mut Activation<'_, 'gc>,
        reference_value: u32,
        read_mask: u32,
        write_mask: u32,
    ) {
        self.set_stencil_reference_value_internal(reference_value, read_mask, write_mask);
    }

    fn set_sampler_state_at(
        self,
        activation: &mut Activation<'_, 'gc>,
        sampler: i32,
        #[name = "wrap"] wrap: Context3DWrapMode,
        #[name = "filter"] filter: Context3DTextureFilter,
        #[name = "mipfilter"] mip_filter: Context3DMipFilter,
    ) {
        if matches!(
            filter,
            Context3DTextureFilter::Anisotropic2X
                | Context3DTextureFilter::Anisotropic4X
                | Context3DTextureFilter::Anisotropic8X
                | Context3DTextureFilter::Anisotropic16X
        ) {
            avm2_stub_method!(
                activation,
                "flash.display3D.Context3D",
                "setSamplerStateAt",
                "filter == 'anisotropic'"
            );
        }

        if !matches!(mip_filter, Context3DMipFilter::MipNone) {
            avm2_stub_method!(
                activation,
                "flash.display3D.Context3D",
                "setSamplerStateAt",
                "mipFilter != 'none'"
            );
        }

        self.set_sampler_state_at_internal(sampler as u32, wrap, filter);
    }

    fn set_scissor_rectangle(
        self,
        activation: &mut Activation<'_, 'gc>,
        rectangle: Option<Object<'gc>>,
    ) -> Result<(), Error<'gc>> {
        let rectangle = if let Some(rectangle) = rectangle {
            let x = rectangle
                .get_slot(rectangle_slots::X)
                .coerce_to_number(activation)?;
            let y = rectangle
                .get_slot(rectangle_slots::Y)
                .coerce_to_number(activation)?;
            let width = rectangle
                .get_slot(rectangle_slots::WIDTH)
                .coerce_to_number(activation)?;
            let height = rectangle
                .get_slot(rectangle_slots::HEIGHT)
                .coerce_to_number(activation)?;
            Some(Rectangle {
                x_min: Twips::from_pixels(x),
                y_min: Twips::from_pixels(y),
                x_max: Twips::from_pixels(x + width),
                y_max: Twips::from_pixels(y + height),
            })
        } else {
            None
        };

        self.set_scissor_rectangle_internal(rectangle);

        Ok(())
    }

    fn set_vertex_buffer_at(
        self,
        activation: &mut Activation<'_, 'gc>,
        index: u32,
        buffer: Option<VertexBuffer3DObject<'gc>>,
        buffer_offset: u32,
        #[name = "vertexStreamFormat"] format: Value<'gc>,
    ) -> Result<(), Error<'gc>> {
        let buffer = if let Some(buffer) = buffer {
            // Note - we only check the format string if the buffer is non-null
            let format = format
                .coerce_to_string(activation)?
                .parse()
                .map_err(|_| make_error_2008(activation, "vertexStreamFormat"))?;

            Some((buffer, format))
        } else {
            None
        };

        self.set_vertex_buffer_at_internal(index, buffer, buffer_offset);

        Ok(())
    }

    fn dispose(self, activation: &mut Activation<'_, 'gc>) {
        avm2_stub_method!(activation, "flash.display3D.Context3D", "dispose");

        self.stage3d().set_context3d(None, activation.gc());
    }
}

/// Helper methods dispatching to the rendering backend.
/// These are only used by the native methods above.
impl<'gc> Context3DObject<'gc> {
    fn configure_back_buffer_internal(
        self,
        width: u32,
        height: u32,
        anti_alias: u32,
        depth_and_stencil: bool,
        wants_best_resolution: bool,
        wants_best_resolution_on_browser_zoom: bool,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::ConfigureBackBuffer {
                width,
                height,
                anti_alias,
                depth_and_stencil,
                wants_best_resolution,
                wants_best_resolution_on_browser_zoom,
            })
        });
    }

    fn create_index_buffer_internal(
        self,
        num_indices: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> IndexBuffer3DObject<'gc> {
        let index_buffer = self
            .with_context_3d(|ctx| ctx.create_index_buffer(BufferUsage::StaticDraw, num_indices));

        IndexBuffer3DObject::from_handle(activation, self, index_buffer)
    }

    #[expect(clippy::too_many_arguments)]
    fn create_texture_internal(
        self,
        width: u32,
        height: u32,
        format: Context3DTextureFormat,
        optimize_for_render_to_texture: bool,
        streaming_levels: u32,
        class: ClassObject<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<TextureObject<'gc>, Error<'gc>> {
        check_texture_stub(activation, format);

        let texture = self.with_context_3d(|ctx| {
            ctx.create_texture(
                width,
                height,
                format,
                optimize_for_render_to_texture,
                streaming_levels,
            )
        })?;

        Ok(TextureObject::from_handle(
            activation, self, texture, format, class,
        ))
    }

    fn create_vertex_buffer_internal(
        self,
        num_vertices: u32,
        data_32_per_vertex: u8,
        usage: BufferUsage,
        activation: &mut Activation<'_, 'gc>,
    ) -> VertexBuffer3DObject<'gc> {
        let handle = self.with_context_3d(|ctx| {
            ctx.create_vertex_buffer(usage, num_vertices, data_32_per_vertex)
        });

        VertexBuffer3DObject::from_handle(activation, self, handle, data_32_per_vertex)
    }

    fn set_vertex_buffer_at_internal(
        self,
        index: u32,
        buffer: Option<(VertexBuffer3DObject<'gc>, Context3DVertexBufferFormat)>,
        buffer_offset: u32,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::SetVertexBufferAt {
                index,
                buffer: buffer.map(|(b, format)| (b.handle(), format)),
                buffer_offset,
            })
        });
    }

    fn create_program_internal(self, activation: &mut Activation<'_, 'gc>) -> Program3DObject<'gc> {
        Program3DObject::from_context(activation, self)
    }

    fn set_program_internal(self, program: Option<Program3DObject<'gc>>) {
        let module = program.and_then(|p| p.shader_module_handle().borrow().clone());

        self.with_context_3d(|ctx| ctx.process_command(Context3DCommand::SetShaders { module }));
    }

    fn draw_triangles_internal(
        self,
        index_buffer: IndexBuffer3DObject<'gc>,
        first_index: u32,
        mut num_triangles: i32,
    ) {
        if num_triangles == -1 {
            // FIXME - should we error if the number of indices isn't a multiple of 3?
            num_triangles = (index_buffer.count() / 3) as i32;
        }
        let handle = index_buffer.handle();

        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::DrawTriangles {
                index_buffer: &*handle,
                first_index: first_index as usize,
                num_triangles: num_triangles as isize,
            })
        });
    }

    fn set_program_constants_internal(
        self,
        program_type: ProgramType,
        first_register: u32,
        matrix_raw_data_column_major: Vec<f32>,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::SetProgramConstantsFromVector {
                program_type,
                first_register,
                matrix_raw_data_column_major,
            })
        });
    }

    fn set_culling_internal(self, face: Context3DTriangleFace) {
        self.with_context_3d(|ctx| ctx.process_command(Context3DCommand::SetCulling { face }));
    }

    fn set_blend_factors_internal(
        self,
        source_factor: Context3DBlendFactor,
        destination_factor: Context3DBlendFactor,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::SetBlendFactors {
                source_factor,
                destination_factor,
            })
        });
    }

    fn set_render_to_texture_internal(
        self,
        texture: Rc<dyn Texture>,
        enable_depth_and_stencil: bool,
        anti_alias: u32,
        surface_selector: u32,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::SetRenderToTexture {
                texture,
                enable_depth_and_stencil,
                anti_alias,
                surface_selector,
            })
        });
    }

    fn set_render_to_back_buffer_internal(self) {
        self.with_context_3d(|ctx| ctx.process_command(Context3DCommand::SetRenderToBackBuffer));
    }

    fn present_internal(self) {
        self.with_context_3d(|ctx| ctx.present())
    }

    #[expect(clippy::too_many_arguments)]
    fn clear_internal(
        self,
        red: f64,
        green: f64,
        blue: f64,
        alpha: f64,
        depth: f64,
        stencil: u32,
        mask: u32,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::Clear {
                red,
                green,
                blue,
                alpha,
                depth,
                stencil,
                mask,
            })
        });
    }

    fn create_cube_texture_internal(
        self,
        size: u32,
        format: Context3DTextureFormat,
        optimize_for_render_to_texture: bool,
        streaming_levels: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<TextureObject<'gc>, Error<'gc>> {
        check_texture_stub(activation, format);

        let texture = self.with_context_3d(|ctx| {
            ctx.create_cube_texture(
                size,
                format,
                optimize_for_render_to_texture,
                streaming_levels,
            )
        })?;

        let class = activation.avm2().classes().cubetexture;

        Ok(TextureObject::from_handle(
            activation, self, texture, format, class,
        ))
    }

    fn set_texture_at_internal(self, sampler: u32, texture: Option<Rc<dyn Texture>>, cube: bool) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::SetTextureAt {
                sampler,
                texture,
                cube,
            })
        });
    }

    fn set_color_mask_internal(self, red: bool, green: bool, blue: bool, alpha: bool) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::SetColorMask {
                red,
                green,
                blue,
                alpha,
            })
        });
    }

    fn set_depth_test_internal(self, depth_mask: bool, pass_compare_mode: Context3DCompareMode) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::SetDepthTest {
                depth_mask,
                pass_compare_mode,
            })
        });
    }

    fn set_sampler_state_at_internal(
        self,
        sampler: u32,
        wrap: Context3DWrapMode,
        filter: Context3DTextureFilter,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::SetSamplerStateAt {
                sampler,
                wrap,
                filter,
            })
        });
    }

    fn set_scissor_rectangle_internal(self, rect: Option<Rectangle<Twips>>) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::SetScissorRectangle { rect })
        });
    }

    fn set_stencil_actions_internal(
        self,
        triangle_face: Context3DTriangleFace,
        compare_mode: Context3DCompareMode,
        on_both_pass: Context3DStencilAction,
        on_depth_fail: Context3DStencilAction,
        on_depth_pass_stencil_fail: Context3DStencilAction,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::SetStencilActions {
                triangle_face,
                compare_mode,
                on_both_pass,
                on_depth_fail,
                on_depth_pass_stencil_fail,
            })
        });
    }

    fn set_stencil_reference_value_internal(
        self,
        reference_value: u32,
        read_mask: u32,
        write_mask: u32,
    ) {
        self.with_context_3d(|ctx| {
            ctx.process_command(Context3DCommand::SetStencilReferenceValue {
                reference_value,
                read_mask,
                write_mask,
            })
        });
    }
}

fn check_texture_stub(activation: &mut Activation<'_, '_>, format: Context3DTextureFormat) {
    match format {
        Context3DTextureFormat::BgrPacked => {
            avm2_stub_method!(
                activation,
                "flash.display3D.Context3D",
                "createTexture",
                "with BgrPacked"
            );
        }
        Context3DTextureFormat::Compressed => {
            avm2_stub_method!(
                activation,
                "flash.display3D.Context3D",
                "createTexture",
                "with Compressed"
            );
        }
        _ => {}
    }
}
