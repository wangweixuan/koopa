use crate::ir::instructions::*;
use crate::ir::structs::BasicBlockRef;
use crate::ir::types::Type;
use crate::ir::values::*;
use crate::utils::NewWithRef;
use intrusive_collections::{intrusive_adapter, LinkedList, LinkedListLink, UnsafeRef};
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};

/// Value in Koopa IR.
///
/// A value can be used by other users.
pub struct Value {
  link: LinkedListLink,
  ty: Type,
  kind: ValueKind,
  inner: RefCell<ValueInner>,
}

intrusive_adapter!(pub ValueAdapter = ValueRc: Value { link: LinkedListLink });

/// Rc of `Value`.
///
/// Used when a type has ownership of `Value`.
pub type ValueRc = Rc<Value>;

/// Reference of `Value`.
///
/// Used when a type only needs to refer to `Value`.
pub type ValueRef = Weak<Value>;

impl Value {
  pub(crate) fn new(ty: Type, kind: ValueKind) -> ValueRc {
    Rc::new(Self {
      link: LinkedListLink::new(),
      ty,
      kind,
      inner: RefCell::new(ValueInner {
        name: None,
        uses: LinkedList::default(),
        bb: None,
      }),
    })
  }

  pub(crate) fn new_with_init<F>(ty: Type, init: F) -> ValueRc
  where
    F: FnOnce(ValueRef) -> ValueKind,
  {
    Rc::new_with_ref(|value| Self {
      link: LinkedListLink::new(),
      ty,
      kind: init(value),
      inner: RefCell::new(ValueInner {
        name: None,
        uses: LinkedList::default(),
        bb: None,
      }),
    })
  }

  /// Gets the type of the current `Value`.
  pub fn ty(&self) -> &Type {
    &self.ty
  }

  /// Gets the kind of the current `Value`.
  pub fn kind(&self) -> &ValueKind {
    &self.kind
  }

  /// Checks if the current `Value` is a constant.
  pub fn is_const(&self) -> bool {
    matches!(
      self.kind,
      ValueKind::Integer(..)
        | ValueKind::ZeroInit(..)
        | ValueKind::Undef(..)
        | ValueKind::Aggregate(..)
    )
  }

  /// Checks if the current `Value` is an instruction.
  pub fn is_inst(&self) -> bool {
    !matches!(
      self.kind,
      ValueKind::Integer(..)
        | ValueKind::ZeroInit(..)
        | ValueKind::Undef(..)
        | ValueKind::Aggregate(..)
        | ValueKind::ArgRef(..)
    )
  }

  /// Immutably borrows the inner of the current value.
  ///
  /// # Panics
  ///
  /// Panics if the inner value is currently mutably borrowed.
  pub fn inner(&self) -> Ref<ValueInner> {
    self.inner.borrow()
  }

  /// Mutably borrows the inner of the current value.
  ///
  /// # Panics
  ///
  /// Panics if the inner value is currently borrowed.
  pub fn inner_mut(&self) -> RefMut<ValueInner> {
    self.inner.borrow_mut()
  }
}

pub struct ValueInner {
  name: Option<String>,
  uses: LinkedList<UseAdapter>,
  bb: Option<BasicBlockRef>,
}

impl ValueInner {
  /// Gets the name of the current `Value`
  pub fn name(&self) -> &Option<String> {
    &self.name
  }

  /// Sets the name of the current `Value`
  pub fn set_name(&mut self, name: Option<String>) {
    assert!(
      name.as_ref().map_or(true, |n| n.len() > 1
        && (n.starts_with('%') || n.starts_with('@'))),
      "invalid value name"
    );
    self.name = name;
  }

  /// Gets use list of the current `Value`.
  pub fn uses(&self) -> &LinkedList<UseAdapter> {
    &self.uses
  }

  /// Adds use to the current `Value`.
  fn add_use(&mut self, u: UseRef) {
    self.uses.push_back(u);
  }

  /// Removes the specific use `u` from the current `Value`.
  ///
  /// Undefined if `u` is not in the use list.
  fn remove_use(&mut self, u: &Use) {
    unsafe {
      self.uses.cursor_mut_from_ptr(u).remove();
    }
  }

  /// Replaces all uses of the current `Value` to another `Value`.
  ///
  /// This method will not handle the values in basic blocks.
  /// To replace those values, using `BasicBlockInnwe::replace_inst`.
  pub fn replace_all_uses_with(&mut self, value: Option<ValueRc>) {
    assert!(
      value
        .as_ref()
        .map_or(true, |v| !std::ptr::eq(&v.inner().uses, &self.uses)),
      "`value` can not be the same as `self`!"
    );
    while let Some(u) = self.uses.front_mut().remove() {
      u.as_ref().value.set(value.clone());
      if let Some(v) = value.clone() {
        v.inner_mut().add_use(u);
      }
    }
  }

  /// Gets the parent basic block of the current `Value`.
  pub fn bb(&self) -> &Option<BasicBlockRef> {
    &self.bb
  }

  /// Sets the parent basic block of the current `Value`.
  pub(crate) fn set_bb(&mut self, bb: Option<BasicBlockRef>) {
    self.bb = bb;
  }
}

/// All supported values.
pub enum ValueKind {
  Integer(Integer),
  ZeroInit(ZeroInit),
  Undef(Undef),
  Aggregate(Aggregate),
  ArgRef(ArgRef),
  Alloc(Alloc),
  GlobalAlloc(GlobalAlloc),
  Load(Load),
  Store(Store),
  GetPtr(GetPtr),
  GetElemPtr(GetElemPtr),
  Binary(Binary),
  Unary(Unary),
  Branch(Branch),
  Jump(Jump),
  Call(Call),
  Return(Return),
  Phi(Phi),
}

/// Bidirectional reference between `Value`s and `Instruction`s.
pub struct Use {
  link: LinkedListLink,
  value: Cell<Option<ValueRc>>,
  user: ValueRef,
}

intrusive_adapter!(pub UseAdapter = UseRef: Use { link: LinkedListLink });

/// Box of `Use`.
///
/// Used when a type has ownership of `Use`.
pub type UseBox = Box<Use>;

/// Reference of `Use`.
///
/// Used when a type only needs to refer to `Use`.
pub type UseRef = UnsafeRef<Use>;

impl Use {
  /// Creates a new `Rc` of `Use`.
  pub fn new(value: Option<ValueRc>, user: ValueRef) -> UseBox {
    let use_ptr = Box::into_raw(Box::new(Self {
      link: LinkedListLink::new(),
      value: Cell::new(value.clone()),
      user,
    }));
    unsafe {
      if let Some(val) = value {
        val.inner_mut().add_use(UnsafeRef::from_raw(use_ptr));
      }
      Box::from_raw(use_ptr)
    }
  }

  /// Gets the clone of value that the current use holds.
  pub fn value(&self) -> Option<ValueRc> {
    let v = self.value.take();
    self.value.set(v.clone());
    v
  }

  /// Gets the user that the current use holds.
  pub fn user(&self) -> &ValueRef {
    &self.user
  }

  /// Sets the value that the current use holds.
  pub fn set_value(&self, value: Option<ValueRc>) {
    let old_val = self.value.replace(value.clone());
    if let Some(v) = old_val {
      v.inner_mut().remove_use(self);
    }
    if let Some(v) = value {
      v.inner_mut().add_use(unsafe { UnsafeRef::from_raw(self) });
    }
  }
}

impl Drop for Use {
  fn drop(&mut self) {
    let s = &*self;
    if let Some(v) = s.value.take() {
      v.inner_mut().remove_use(s)
    }
  }
}
