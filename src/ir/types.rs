use std::collections::HashMap;
use std::rc::Rc;
use std::{cmp, convert, fmt};

/// Type data.
#[derive(Hash, Clone, PartialEq, Eq)]
pub enum TypeData {
  /// 32-bit integer.
  Int32,
  /// Array (with base type and shape).
  Array(Type, usize),
  /// Pointer (with base type).
  Pointer(Type),
  /// Function (with optional return type and parameter types).
  Function(Option<Type>, Vec<Type>),
}

impl fmt::Display for TypeData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      TypeData::Int32 => write!(f, "i32"),
      TypeData::Array(t, len) => write!(f, "{}[{}]", t, len),
      TypeData::Pointer(t) => write!(f, "{}*", t),
      TypeData::Function(ret, args) => {
        if let Some(ret) = ret {
          write!(f, "{}", ret);
        }
        write!(f, "(");
        args.iter().fold(true, |first, t| {
          if !first {
            write!(f, ", ");
          }
          write!(f, "{}", t);
          false
        });
        write!(f, ")")
      }
    }
  }
}

/// Types in Koopa.
#[derive(Hash, Clone, Eq)]
pub struct Type(Rc<TypeData>);

impl cmp::PartialEq for Type {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.0, &other.0)
  }
}

impl convert::AsRef<TypeData> for Type {
  fn as_ref(&self) -> &TypeData {
    self.0.as_ref()
  }
}

impl fmt::Display for Type {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

/// Type factory for creating types.
pub struct TypeFactory {
  types: HashMap<TypeData, Type>,
}

impl TypeFactory {
  pub fn new() -> Self {
    TypeFactory {
      types: HashMap::new(),
    }
  }

  /// Gets a specific type.
  pub fn get_type(&mut self, type_data: TypeData) -> Type {
    self.types.get(&type_data).cloned().unwrap_or_else(|| {
      let k = Type(Rc::new(type_data.clone()));
      self.types.insert(type_data, k.clone());
      k
    })
  }

  /// Gets an `i32` type.
  pub fn get_i32(&mut self) -> Type {
    self.get_type(TypeData::Int32)
  }

  /// Gets an array type.
  pub fn get_array(&mut self, base: Type, len: usize) -> Type {
    self.get_type(TypeData::Array(base, len))
  }

  /// Gets an pointer type.
  pub fn get_pointer(&mut self, base: Type) -> Type {
    self.get_type(TypeData::Pointer(base))
  }

  /// Gets an function type.
  pub fn get_function(&mut self, ret: Option<Type>, args: Vec<Type>) -> Type {
    self.get_type(TypeData::Function(ret, args))
  }
}
