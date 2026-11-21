use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use crate::context::UpdateContext;
use crate::display_object::DisplayObjectBase;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use ruffle_common::utils::HasPrefixField;
use ruffle_render::matrix3d::Matrix3D;
use std::cell::{Ref, RefCell};

/// A class instance allocator that allocates Matrix3D objects.
pub fn matrix_3d_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(Matrix3DObject(Gc::new(
        activation.gc(),
        Matrix3DObjectData {
            base,
            matrix: RefCell::new(Matrix3D::IDENTITY),
            owner: Lock::new(None),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Matrix3DObject<'gc>(pub Gc<'gc, Matrix3DObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct Matrix3DObjectWeak<'gc>(pub GcWeak<'gc, Matrix3DObjectData<'gc>>);

impl fmt::Debug for Matrix3DObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Matrix3DObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct Matrix3DObjectData<'gc> {
    /// Base script object.
    base: ScriptObjectData<'gc>,

    matrix: RefCell<Matrix3D>,

    /// The display object this is the `transform.matrix3D` of, if any.
    owner: Lock<Option<GcWeak<'gc, DisplayObjectBase<'gc>>>>,
}

impl<'gc> Matrix3DObject<'gc> {
    pub fn new(context: &mut UpdateContext<'gc>, matrix: Matrix3D) -> Matrix3DObject<'gc> {
        let class = context.avm2.classes().matrix3d;
        let base = ScriptObjectData::new(class);
        Matrix3DObject(Gc::new(
            context.gc(),
            Matrix3DObjectData {
                base,
                matrix: RefCell::new(matrix),
                owner: Lock::new(None),
            },
        ))
    }

    pub fn matrix_ref(&self) -> Ref<'_, Matrix3D> {
        self.0.matrix.borrow()
    }

    pub fn matrix(self) -> Matrix3D {
        *self.matrix_ref()
    }

    /// Sets the matrix, applying it to the display object this is bound to.
    pub fn set_matrix(self, mc: &Mutation<'gc>, matrix: Matrix3D) {
        self.store_matrix(matrix);

        if let Some(owner) = self.owner(mc) {
            owner.set_matrix_3d(matrix);
        }
    }

    /// Changes the matrix, applying it to the display object this is bound to.
    pub fn update_matrix(self, mc: &Mutation<'gc>, update: impl FnOnce(&mut Matrix3D)) {
        let mut matrix = self.matrix();
        update(&mut matrix);

        self.set_matrix(mc, matrix);
    }

    /// Sets the matrix without applying it to the display object this is
    /// bound to, which is how that display object updates it.
    pub fn store_matrix(self, matrix: Matrix3D) {
        *self.0.matrix.borrow_mut() = matrix;
    }

    /// The display object this is the `transform.matrix3D` of, if it's still
    /// bound to it.
    pub fn owner(self, mc: &Mutation<'gc>) -> Option<Gc<'gc, DisplayObjectBase<'gc>>> {
        let owner = self.0.owner.get()?.upgrade(mc)?;
        let bound = owner
            .matrix3d_object()
            .is_some_and(|object| Gc::ptr_eq(object.0, self.0));

        bound.then_some(owner)
    }

    pub fn set_owner(self, mc: &Mutation<'gc>, owner: Gc<'gc, DisplayObjectBase<'gc>>) {
        unlock!(Gc::write(mc, self.0), Matrix3DObjectData, owner).set(Some(Gc::downgrade(owner)));
    }
}

impl<'gc> TObject<'gc> for Matrix3DObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}
