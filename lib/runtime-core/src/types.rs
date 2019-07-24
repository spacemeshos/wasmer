use crate::{memory::MemoryType, module::ModuleInfo, structures::TypedIndex, units::Pages};
use std::borrow::Cow;

/// Represents a WebAssembly type.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    /// The `i32` type.
    I32,
    /// The `i64` type.
    I64,
    /// The `f32` type.
    F32,
    /// The `f64` type.
    F64,
    /// The `v128` type.
    V128,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Represents a WebAssembly value.
///
/// As the number of types in WebAssembly expand,
/// this structure will expand as well.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Value {
    /// The `i32` type.
    I32(i32),
    /// The `i64` type.
    I64(i64),
    /// The `f32` type.
    F32(f32),
    /// The `f64` type.
    F64(f64),
    /// The `v128` type.
    V128(u128),
}

impl Value {
    pub fn ty(&self) -> Type {
        match self {
            Value::I32(_) => Type::I32,
            Value::I64(_) => Type::I64,
            Value::F32(_) => Type::F32,
            Value::F64(_) => Type::F64,
            Value::V128(_) => Type::V128,
        }
    }

    pub fn to_u128(&self) -> u128 {
        match *self {
            Value::I32(x) => x as u128,
            Value::I64(x) => x as u128,
            Value::F32(x) => f32::to_bits(x) as u128,
            Value::F64(x) => f64::to_bits(x) as u128,
            Value::V128(x) => x,
        }
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::I32(i)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::I64(i)
    }
}

impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Value::F32(f)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::F64(f)
    }
}

impl From<u128> for Value {
    fn from(v: u128) -> Self {
        Value::V128(v)
    }
}

pub unsafe trait NativeWasmType: Copy + Into<Value>
where
    Self: Sized,
{
    const TYPE: Type;
    fn from_binary(bits: u64) -> Self;
    fn to_binary(self) -> u64;
}

unsafe impl NativeWasmType for i32 {
    const TYPE: Type = Type::I32;
    fn from_binary(bits: u64) -> Self {
        bits as _
    }
    fn to_binary(self) -> u64 {
        self as _
    }
}
unsafe impl NativeWasmType for i64 {
    const TYPE: Type = Type::I64;
    fn from_binary(bits: u64) -> Self {
        bits as _
    }
    fn to_binary(self) -> u64 {
        self as _
    }
}
unsafe impl NativeWasmType for f32 {
    const TYPE: Type = Type::F32;
    fn from_binary(bits: u64) -> Self {
        f32::from_bits(bits as u32)
    }
    fn to_binary(self) -> u64 {
        self.to_bits() as _
    }
}
unsafe impl NativeWasmType for f64 {
    const TYPE: Type = Type::F64;
    fn from_binary(bits: u64) -> Self {
        f64::from_bits(bits)
    }
    fn to_binary(self) -> u64 {
        self.to_bits()
    }
}

pub unsafe trait WasmExternType: Copy
where
    Self: Sized,
{
    type Native: NativeWasmType;
    fn from_native(native: Self::Native) -> Self;
    fn to_native(self) -> Self::Native;
}

unsafe impl WasmExternType for i8 {
    type Native = i32;
    fn from_native(native: Self::Native) -> Self {
        native as _
    }
    fn to_native(self) -> Self::Native {
        self as _
    }
}
unsafe impl WasmExternType for u8 {
    type Native = i32;
    fn from_native(native: Self::Native) -> Self {
        native as _
    }
    fn to_native(self) -> Self::Native {
        self as _
    }
}
unsafe impl WasmExternType for i16 {
    type Native = i32;
    fn from_native(native: Self::Native) -> Self {
        native as _
    }
    fn to_native(self) -> Self::Native {
        self as _
    }
}
unsafe impl WasmExternType for u16 {
    type Native = i32;
    fn from_native(native: Self::Native) -> Self {
        native as _
    }
    fn to_native(self) -> Self::Native {
        self as _
    }
}
unsafe impl WasmExternType for i32 {
    type Native = i32;
    fn from_native(native: Self::Native) -> Self {
        native
    }
    fn to_native(self) -> Self::Native {
        self
    }
}
unsafe impl WasmExternType for u32 {
    type Native = i32;
    fn from_native(native: Self::Native) -> Self {
        native as _
    }
    fn to_native(self) -> Self::Native {
        self as _
    }
}
unsafe impl WasmExternType for i64 {
    type Native = i64;
    fn from_native(native: Self::Native) -> Self {
        native
    }
    fn to_native(self) -> Self::Native {
        self
    }
}
unsafe impl WasmExternType for u64 {
    type Native = i64;
    fn from_native(native: Self::Native) -> Self {
        native as _
    }
    fn to_native(self) -> Self::Native {
        self as _
    }
}
unsafe impl WasmExternType for f32 {
    type Native = f32;
    fn from_native(native: Self::Native) -> Self {
        native
    }
    fn to_native(self) -> Self::Native {
        self
    }
}
unsafe impl WasmExternType for f64 {
    type Native = f64;
    fn from_native(native: Self::Native) -> Self {
        native
    }
    fn to_native(self) -> Self::Native {
        self
    }
}

// pub trait IntegerAtomic
// where
//     Self: Sized
// {
//     type Primitive;

//     fn add(&self, other: Self::Primitive) -> Self::Primitive;
//     fn sub(&self, other: Self::Primitive) -> Self::Primitive;
//     fn and(&self, other: Self::Primitive) -> Self::Primitive;
//     fn or(&self, other: Self::Primitive) -> Self::Primitive;
//     fn xor(&self, other: Self::Primitive) -> Self::Primitive;
//     fn load(&self) -> Self::Primitive;
//     fn store(&self, other: Self::Primitive) -> Self::Primitive;
//     fn compare_exchange(&self, expected: Self::Primitive, new: Self::Primitive) -> Self::Primitive;
//     fn swap(&self, other: Self::Primitive) -> Self::Primitive;
// }

pub unsafe trait ValueType: Copy
where
    Self: Sized,
{
}

macro_rules! convert_value_impl {
    ($t:ty) => {
        unsafe impl ValueType for $t {}
    };
    ( $($t:ty),* ) => {
        $(
            convert_value_impl!($t);
        )*
    };
}

convert_value_impl!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementType {
    /// Any wasm function.
    Anyfunc,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct TableDescriptor {
    /// Type of data stored in this table.
    pub element: ElementType,
    /// The minimum number of elements that must be stored in this table.
    pub minimum: u32,
    /// The maximum number of elements in this table.
    pub maximum: Option<u32>,
}

impl TableDescriptor {
    pub(crate) fn fits_in_imported(&self, imported: TableDescriptor) -> bool {
        // TODO: We should define implementation limits.
        let imported_max = imported.maximum.unwrap_or(u32::max_value());
        let self_max = self.maximum.unwrap_or(u32::max_value());
        self.element == imported.element
            && imported_max <= self_max
            && self.minimum <= imported.minimum
    }
}

/// A const value initializer.
/// Over time, this will be able to represent more and more
/// complex expressions.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Initializer {
    /// Corresponds to a `const.*` instruction.
    Const(Value),
    /// Corresponds to a `get_global` instruction.
    GetGlobal(ImportedGlobalIndex),
}

/// Describes the mutability and type of a Global
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalDescriptor {
    pub mutable: bool,
    pub ty: Type,
}

/// A wasm global.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlobalInit {
    pub desc: GlobalDescriptor,
    pub init: Initializer,
}

/// A wasm memory.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryDescriptor {
    /// The minimum number of allowed pages.
    pub minimum: Pages,
    /// The maximum number of allowed pages.
    pub maximum: Option<Pages>,
    /// This memory can be shared between wasm threads.
    pub shared: bool,
}

impl MemoryDescriptor {
    pub fn memory_type(self) -> MemoryType {
        match (self.maximum.is_some(), self.shared) {
            (true, true) => MemoryType::SharedStatic,
            (true, false) => MemoryType::Static,
            (false, false) => MemoryType::Dynamic,
            (false, true) => panic!("shared memory without a max is not allowed"),
        }
    }

    pub(crate) fn fits_in_imported(&self, imported: MemoryDescriptor) -> bool {
        let imported_max = imported.maximum.unwrap_or(Pages(65_536));
        let self_max = self.maximum.unwrap_or(Pages(65_536));

        self.shared == imported.shared
            && imported_max <= self_max
            && self.minimum <= imported.minimum
    }
}

/// The signature of a function that is either implemented
/// in a wasm module or exposed to wasm by the host.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FuncSig {
    params: Cow<'static, [Type]>,
    returns: Cow<'static, [Type]>,
}

impl FuncSig {
    pub fn new<Params, Returns>(params: Params, returns: Returns) -> Self
    where
        Params: Into<Cow<'static, [Type]>>,
        Returns: Into<Cow<'static, [Type]>>,
    {
        Self {
            params: params.into(),
            returns: returns.into(),
        }
    }

    pub fn params(&self) -> &[Type] {
        &self.params
    }

    pub fn returns(&self) -> &[Type] {
        &self.returns
    }

    pub fn check_param_value_types(&self, params: &[Value]) -> bool {
        self.params.len() == params.len()
            && self
                .params
                .iter()
                .zip(params.iter().map(|val| val.ty()))
                .all(|(t0, ref t1)| t0 == t1)
    }
}

impl std::fmt::Display for FuncSig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let params = self
            .params
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let returns = self
            .returns
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "[{}] -> [{}]", params, returns)
    }
}

pub trait LocalImport {
    type Local: TypedIndex;
    type Import: TypedIndex;
}

#[rustfmt::skip]
macro_rules! define_map_index {
    ($ty:ident) => {
        #[derive(Serialize, Deserialize)]
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $ty (u32);
        impl TypedIndex for $ty {
            #[doc(hidden)]
            fn new(index: usize) -> Self {
                $ty (index as _)
            }

            #[doc(hidden)]
            fn index(&self) -> usize {
                self.0 as usize
            }
        }
    };
    ($($normal_ty:ident,)* | local: $($local_ty:ident,)* | imported: $($imported_ty:ident,)*) => {
        $(
            define_map_index!($normal_ty);
            define_map_index!($local_ty);
            define_map_index!($imported_ty);

            impl LocalImport for $normal_ty {
                type Local = $local_ty;
                type Import = $imported_ty;
            }
        )*
    };
}

#[rustfmt::skip]
define_map_index![
    FuncIndex, MemoryIndex, TableIndex, GlobalIndex,
    | local: LocalFuncIndex, LocalMemoryIndex, LocalTableIndex, LocalGlobalIndex,
    | imported: ImportedFuncIndex, ImportedMemoryIndex, ImportedTableIndex, ImportedGlobalIndex,
];

#[rustfmt::skip]
macro_rules! define_local_or_import {
    ($ty:ident, $local_ty:ident, $imported_ty:ident, $imports:ident) => {
        impl $ty {
            pub fn local_or_import(self, info: &ModuleInfo) -> LocalOrImport<$ty> {
                if self.index() < info.$imports.len() {
                    LocalOrImport::Import(<Self as LocalImport>::Import::new(self.index()))
                } else {
                    LocalOrImport::Local(<Self as LocalImport>::Local::new(self.index() - info.$imports.len()))
                }
            }
        }

        impl $local_ty {
            pub fn convert_up(self, info: &ModuleInfo) -> $ty {
                $ty ((self.index() + info.$imports.len()) as u32)
            }
        }

        impl $imported_ty {
            pub fn convert_up(self, _info: &ModuleInfo) -> $ty {
                $ty (self.index() as u32)
            }
        }
    };
    ($(($ty:ident | ($local_ty:ident, $imported_ty:ident): $imports:ident),)*) => {
        $(
            define_local_or_import!($ty, $local_ty, $imported_ty, $imports);
        )*
    };
}

#[rustfmt::skip]
define_local_or_import![
    (FuncIndex | (LocalFuncIndex, ImportedFuncIndex): imported_functions),
    (MemoryIndex | (LocalMemoryIndex, ImportedMemoryIndex): imported_memories),
    (TableIndex | (LocalTableIndex, ImportedTableIndex): imported_tables),
    (GlobalIndex | (LocalGlobalIndex, ImportedGlobalIndex): imported_globals),
];

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SigIndex(u32);
impl TypedIndex for SigIndex {
    #[doc(hidden)]
    fn new(index: usize) -> Self {
        SigIndex(index as _)
    }

    #[doc(hidden)]
    fn index(&self) -> usize {
        self.0 as usize
    }
}

pub enum LocalOrImport<T>
where
    T: LocalImport,
{
    Local(T::Local),
    Import(T::Import),
}

impl<T> LocalOrImport<T>
where
    T: LocalImport,
{
    pub fn local(self) -> Option<T::Local> {
        match self {
            LocalOrImport::Local(local) => Some(local),
            LocalOrImport::Import(_) => None,
        }
    }

    pub fn import(self) -> Option<T::Import> {
        match self {
            LocalOrImport::Import(import) => Some(import),
            LocalOrImport::Local(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::NativeWasmType;
    use crate::types::WasmExternType;

    #[test]
    fn test_native_types_round_trip() {
        assert_eq!(
            42i32,
            i32::from_native(i32::from_binary((42i32).to_native().to_binary()))
        );

        assert_eq!(
            -42i32,
            i32::from_native(i32::from_binary((-42i32).to_native().to_binary()))
        );

        use std::i64;
        let xi64 = i64::MAX;
        assert_eq!(
            xi64,
            i64::from_native(i64::from_binary((xi64).to_native().to_binary()))
        );
        let yi64 = i64::MIN;
        assert_eq!(
            yi64,
            i64::from_native(i64::from_binary((yi64).to_native().to_binary()))
        );

        assert_eq!(
            16.5f32,
            f32::from_native(f32::from_binary((16.5f32).to_native().to_binary()))
        );

        assert_eq!(
            -16.5f32,
            f32::from_native(f32::from_binary((-16.5f32).to_native().to_binary()))
        );

        use std::f64;
        let xf64: f64 = f64::MAX;
        assert_eq!(
            xf64,
            f64::from_native(f64::from_binary((xf64).to_native().to_binary()))
        );

        let yf64: f64 = f64::MIN;
        assert_eq!(
            yf64,
            f64::from_native(f64::from_binary((yf64).to_native().to_binary()))
        );
    }

}
