use crate::avm2::error::{make_error_2007, make_error_2008};
use crate::avm2::object::{FunctionObject, Object};
use crate::avm2::{Activation, Error, Value};
use crate::string::AvmString;

use ruffle_macros::istr;
use ruffle_wstr::FromWStr;

/// Extensions over parameters that are passed into AS-defined, Rust-implemented methods.
///
/// It is expected that the AS signature is correct and you only operate on values defined from it.
/// These values will be `expect()`ed to exist, and any method here will panic if they're missing.
///
/// The rules for ActionScript type coercion may be surprising. Here is a table mapping
/// ParametersExt functions to the corresponding ActionScript types:
///
/// `ParametersExt::get_value`: All parameter types work
/// `ParametersExt::get_f64`: `Number`, `int`, or `uint` type
/// `ParametersExt::get_i32`: `Number`, `int`, or `uint` type
/// `ParametersExt::get_u32`: `Number`, `int`, or `uint` type
/// `ParametersExt::get_bool`: `Boolean` type only
/// `ParametersExt::get_string` and family: `String` type only
/// `ParametersExt::get_function` and family: `Function` type only
/// `ParametersExt::get_object` and family: Any non-primitive type; i.e. any type *except* the following:
///   - `*` (aka "any") type
///   - `Object` type (as `Object` can represent any primitive value except `undefined`)
///   - `Boolean` type
///   - `int` type
///   - `uint` type
///   - `Number` type
///   - `String` type
pub trait ParametersExt<'gc> {
    /// Gets the value at the given index.
    fn get_value(&self, index: usize) -> Value<'gc>;

    /// Gets the value at the given index, if it exists.
    fn get_optional(&self, index: usize) -> Option<Value<'gc>>;

    /// Gets the value at the given index as an Object. It is expected that the
    /// value is either Object or Null.
    ///
    /// If the value is null, a TypeError 2007 is raised.
    fn get_object(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        name: &'static str,
    ) -> Result<Object<'gc>, Error<'gc>> {
        self.try_get_object(index)
            .ok_or_else(|| make_error_2007(activation, name))
    }

    /// Gets the value at the given index as an Object. It is expected that the
    /// value is either Object or Null.
    ///
    /// If the value is null, None is returned.
    fn try_get_object(&self, index: usize) -> Option<Object<'gc>> {
        match self.get_value(index) {
            Value::Null => None,
            Value::Object(o) => Some(o),
            _ => panic!("Expected Object or null as parameter"),
        }
    }

    /// Gets the value at the given index as a FunctionObject. It is expected
    /// that the value is either FunctionObject or Null.
    ///
    /// If the value is null, a TypeError 2007 is raised.
    fn get_function(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        name: &'static str,
    ) -> Result<FunctionObject<'gc>, Error<'gc>> {
        self.try_get_function(index)
            .ok_or_else(|| make_error_2007(activation, name))
    }

    /// Gets the value at the given index as an FunctionObject. It is expected
    /// that the value is either FunctionObject or Null.
    ///
    /// If the value is null, None is returned.
    fn try_get_function(&self, index: usize) -> Option<FunctionObject<'gc>> {
        match self.get_value(index) {
            Value::Null => None,
            Value::Object(Object::FunctionObject(f)) => Some(f),
            _ => panic!("Expected FunctionObject or null as parameter"),
        }
    }

    /// Gets the Number-typed value at the given index. It is expected that the
    /// value is numerical.
    fn get_f64(&self, index: usize) -> f64 {
        self.get_value(index).as_f64()
    }

    /// Gets the uint-typed value at the given index. It is expected that the
    /// value is numerical.
    fn get_u32(&self, index: usize) -> u32 {
        self.get_value(index).as_u32()
    }

    /// Gets the int-typed value at the given index. It is expected that the
    /// value is numerical.
    fn get_i32(&self, index: usize) -> i32 {
        self.get_value(index).as_i32()
    }

    /// Gets the Boolean-typed value at the given index. It is expected that the
    /// value is of the Boolean type.
    fn get_bool(&self, index: usize) -> bool {
        match self.get_value(index) {
            Value::Bool(b) => b,
            _ => unreachable!("Expected Boolean-typed parameter"),
        }
    }

    /// Gets the String-typed value at the given index. It is expected that the
    /// value is either String or Null.
    ///
    /// If the value is null, None is returned.
    fn try_get_string(&self, index: usize) -> Option<AvmString<'gc>> {
        match self.get_value(index) {
            Value::Null => None,
            Value::String(s) => Some(s),
            _ => unreachable!("Expected String-typed parameter"),
        }
    }

    /// Like `try_get_string`, but returns "null" for null values instead
    /// of returning `None`.
    fn get_string(&self, activation: &mut Activation<'_, 'gc>, index: usize) -> AvmString<'gc> {
        self.try_get_string(index).unwrap_or_else(|| istr!("null"))
    }

    /// Like `try_get_string`, but throws TypeError 2007 for null values instead
    /// of returning `None`.
    fn get_string_non_null(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        name: &'static str,
    ) -> Result<AvmString<'gc>, Error<'gc>> {
        self.try_get_string(index)
            .ok_or_else(|| make_error_2007(activation, name))
    }
}

impl<'gc> ParametersExt<'gc> for &[Value<'gc>] {
    #[inline]
    fn get_value(&self, index: usize) -> Value<'gc> {
        self[index]
    }

    #[inline]
    fn get_optional(&self, index: usize) -> Option<Value<'gc>> {
        self.get(index).copied()
    }
}

/// Extract a typed Rust value from an AVM2 `Value`, performing AS3 coercion.
pub trait NativeArg<'gc>: Sized {
    fn from_native_arg(
        activation: &mut Activation<'_, 'gc>,
        value: Value<'gc>,
        name: &'static str,
    ) -> Result<Self, Error<'gc>>;
}

impl<'gc> NativeArg<'gc> for Value<'gc> {
    fn from_native_arg(
        _activation: &mut Activation<'_, 'gc>,
        value: Value<'gc>,
        _name: &'static str,
    ) -> Result<Self, Error<'gc>> {
        Ok(value)
    }
}

/// Convert a Rust return value back into an AVM2 `Value`.
pub trait NativeReturn<'gc> {
    fn into_return_value(
        self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>>;
}

impl<'gc> NativeArg<'gc> for f64 {
    fn from_native_arg(
        activation: &mut Activation<'_, 'gc>,
        value: Value<'gc>,
        _name: &'static str,
    ) -> Result<Self, Error<'gc>> {
        value.coerce_to_number(activation)
    }
}

impl<'gc> NativeArg<'gc> for i32 {
    fn from_native_arg(
        activation: &mut Activation<'_, 'gc>,
        value: Value<'gc>,
        _name: &'static str,
    ) -> Result<Self, Error<'gc>> {
        value.coerce_to_i32(activation)
    }
}

impl<'gc> NativeArg<'gc> for u32 {
    fn from_native_arg(
        activation: &mut Activation<'_, 'gc>,
        value: Value<'gc>,
        _name: &'static str,
    ) -> Result<Self, Error<'gc>> {
        value.coerce_to_u32(activation)
    }
}

impl<'gc> NativeArg<'gc> for bool {
    fn from_native_arg(
        _activation: &mut Activation<'_, 'gc>,
        value: Value<'gc>,
        _name: &'static str,
    ) -> Result<Self, Error<'gc>> {
        Ok(value.coerce_to_boolean())
    }
}

impl<'gc> NativeArg<'gc> for AvmString<'gc> {
    fn from_native_arg(
        activation: &mut Activation<'_, 'gc>,
        value: Value<'gc>,
        name: &'static str,
    ) -> Result<Self, Error<'gc>> {
        match value {
            Value::Null => Err(make_error_2007(activation, name)),
            Value::String(s) => Ok(s),
            other => other.coerce_to_string(activation),
        }
    }
}

impl<'gc> NativeArg<'gc> for Object<'gc> {
    fn from_native_arg(
        activation: &mut Activation<'_, 'gc>,
        value: Value<'gc>,
        name: &'static str,
    ) -> Result<Self, Error<'gc>> {
        match value {
            Value::Object(o) => Ok(o),
            _ => Err(make_error_2007(activation, name)),
        }
    }
}

macro_rules! impl_native_arg_for_object_downcast {
    ($($obj_ty:ident :: $downcast:ident),* $(,)?) => {
        $(
            impl<'gc> NativeArg<'gc> for crate::avm2::object::$obj_ty<'gc> {
                fn from_native_arg(
                    _activation: &mut Activation<'_, 'gc>,
                    value: Value<'gc>,
                    _name: &'static str,
                ) -> Result<Self, Error<'gc>> {
                    Ok(value.as_object().unwrap().$downcast().unwrap())
                }
            }
        )*
    };
}

impl_native_arg_for_object_downcast! {
    Context3DObject::as_context_3d,
    Program3DObject::as_program_3d,
    IndexBuffer3DObject::as_index_buffer,
    VertexBuffer3DObject::as_vertex_buffer,
    TextureObject::as_texture,
}

impl<'gc, T: NativeArg<'gc>> NativeArg<'gc> for Option<T> {
    fn from_native_arg(
        activation: &mut Activation<'_, 'gc>,
        value: Value<'gc>,
        name: &'static str,
    ) -> Result<Self, Error<'gc>> {
        match value {
            Value::Null | Value::Undefined => Ok(None),
            other => Ok(Some(T::from_native_arg(activation, other, name)?)),
        }
    }
}

fn parse_string_enum<'gc, T: FromWStr<Err = ()>>(
    activation: &mut Activation<'_, 'gc>,
    value: Value<'gc>,
    name: &'static str,
) -> Result<T, Error<'gc>> {
    let s = AvmString::from_native_arg(activation, value, name)?;
    s.parse().map_err(|_| make_error_2008(activation, name))
}

macro_rules! impl_native_arg_for_string_enum {
    ($($ty:ty),* $(,)?) => {
        $(
            impl<'gc> NativeArg<'gc> for $ty {
                fn from_native_arg(
                    activation: &mut Activation<'_, 'gc>,
                    value: Value<'gc>,
                    name: &'static str,
                ) -> Result<Self, Error<'gc>> {
                    parse_string_enum(activation, value, name)
                }
            }
        )*
    };
}

impl_native_arg_for_string_enum! {
    ruffle_render::backend::Context3DBlendFactor,
    ruffle_render::backend::Context3DCompareMode,
    ruffle_render::backend::Context3DMipFilter,
    ruffle_render::backend::Context3DStencilAction,
    ruffle_render::backend::Context3DTextureFilter,
    ruffle_render::backend::Context3DTextureFormat,
    ruffle_render::backend::Context3DTriangleFace,
    ruffle_render::backend::Context3DVertexBufferFormat,
    ruffle_render::backend::Context3DWrapMode,
    ruffle_render::backend::ProgramType,
}

impl<'gc> NativeReturn<'gc> for Value<'gc> {
    fn into_return_value(
        self,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        Ok(self)
    }
}

impl<'gc> NativeReturn<'gc> for () {
    fn into_return_value(
        self,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Undefined)
    }
}

impl<'gc> NativeReturn<'gc> for f64 {
    fn into_return_value(
        self,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Number(self))
    }
}

impl<'gc> NativeReturn<'gc> for i32 {
    fn into_return_value(
        self,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Integer(self))
    }
}

impl<'gc> NativeReturn<'gc> for u32 {
    fn into_return_value(
        self,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Integer(self as i32))
    }
}

impl<'gc> NativeReturn<'gc> for bool {
    fn into_return_value(
        self,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Bool(self))
    }
}

impl<'gc> NativeReturn<'gc> for AvmString<'gc> {
    fn into_return_value(
        self,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::String(self))
    }
}

impl<'gc> NativeReturn<'gc> for Object<'gc> {
    fn into_return_value(
        self,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(self))
    }
}

macro_rules! impl_native_return_for_object_downcast {
    ($($obj_ty:ident),* $(,)?) => {
        $(
            impl<'gc> NativeReturn<'gc> for crate::avm2::object::$obj_ty<'gc> {
                fn into_return_value(self, _activation: &mut Activation<'_, 'gc>) -> Result<Value<'gc>, Error<'gc>> {
                    Ok(Value::Object(self.into()))
                }
            }
        )*
    };
}

impl_native_return_for_object_downcast! {
    IndexBuffer3DObject,
    Program3DObject,
    TextureObject,
    VertexBuffer3DObject,
}

impl<'gc, T: NativeReturn<'gc>> NativeReturn<'gc> for Option<T> {
    fn into_return_value(
        self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        match self {
            Some(v) => v.into_return_value(activation),
            None => Ok(Value::Null),
        }
    }
}

impl<'gc, T: NativeReturn<'gc>> NativeReturn<'gc> for Result<T, Error<'gc>> {
    fn into_return_value(
        self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        self?.into_return_value(activation)
    }
}
