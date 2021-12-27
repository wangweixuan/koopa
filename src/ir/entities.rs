use crate::ir::dfg::DataFlowGraph;
use crate::ir::idman::{next_func_id, next_value_id};
use crate::ir::idman::{BasicBlockId, FunctionId, ValueId};
use crate::ir::layout::Layout;
use crate::ir::types::{Type, TypeKind};
use crate::ir::values;
use std::cell::{Ref, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};

/// A Koopa IR program.
///
/// Programs can hold global values and functions.
#[derive(Default)]
pub struct Program {
  values: Rc<RefCell<HashMap<Value, ValueData>>>,
  funcs: HashMap<Function, FunctionData>,
}

/// Returns a mutable reference of the global value data by the given
/// value handle.
macro_rules! data_mut {
  ($self:ident, $value:expr) => {
    $self
      .values
      .borrow_mut()
      .get_mut(&$value)
      .expect("value does not exist")
  };
}

impl Program {
  /// Creates a new program.
  pub fn new() -> Self {
    Self::default()
  }

  /// Creates a new global value in the current program.
  ///
  /// # Panics
  ///
  /// Panics if the given value data uses unexisted values.
  pub fn new_value(&mut self, data: ValueData) -> Value {
    let value = Value(next_value_id());
    for v in data.kind().value_uses() {
      data_mut!(self, v).used_by.insert(value);
    }
    self.values.borrow_mut().insert(value, data);
    value
  }

  /// Removes the specific global value by its handle. Returns the
  /// corresponding value data.
  ///
  /// # Panics
  ///
  /// Panics if the given value does not exist, or the removed value is
  /// currently used by other values.
  pub fn remove_value(&mut self, value: Value) -> ValueData {
    let data = self
      .values
      .borrow_mut()
      .remove(&value)
      .expect("`value` does not exist");
    assert!(data.used_by.is_empty(), "`value` is used by other values");
    for v in data.kind().value_uses() {
      data_mut!(self, v).used_by.remove(&value);
    }
    data
  }

  /// Immutably borrows the global value map.
  pub fn borrow_values(&self) -> Ref<HashMap<Value, ValueData>> {
    self.values.as_ref().borrow()
  }

  /// Creates a new function in the current program.
  pub fn new_func(&mut self, mut data: FunctionData) -> Function {
    let func = Function(next_func_id());
    data.dfg.globals = Rc::downgrade(&self.values);
    self.funcs.insert(func, data);
    func
  }

  /// Removes the specific function by its handle.
  ///
  /// Returns the function data if the function was previously in the program.
  pub fn remove_func(&mut self, func: Function) -> Option<FunctionData> {
    self.funcs.remove(&func)
  }

  /// Returns a reference to the function map.
  pub fn funcs(&self) -> &HashMap<Function, FunctionData> {
    &self.funcs
  }

  /// Returns a mutable reference to the function map.
  pub fn funcs_mut(&mut self) -> &mut HashMap<Function, FunctionData> {
    &mut self.funcs
  }
}

/// Weak pointer for the `RefCell` of global value map.
///
/// For [`DataFlowGraph`]s in function.
pub(crate) type GlobalValueMapCell = Weak<RefCell<HashMap<Value, ValueData>>>;

/// A handle of Koopa IR function.
///
/// You can fetch `FunctionData` from [`Program`] by using this handle.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Function(FunctionId);

/// Data of Koopa IR function.
///
/// Functions can hold basic blocks.
pub struct FunctionData {
  ty: Type,
  name: String,
  params: Vec<Value>,
  dfg: DataFlowGraph,
  layout: Layout,
}

impl FunctionData {
  /// Creates a new function definition.
  ///
  /// # Panics
  ///
  /// Panics if the given name not starts with `%` or `@`, or the given
  /// type is not a valid function type.
  pub fn new(name: String, params: Vec<Value>, ty: Type) -> Self {
    Self::check_sanity(&name, &ty);
    Self {
      ty,
      name,
      params,
      dfg: DataFlowGraph::new(),
      layout: Layout::new(),
    }
  }

  /// Creates a new function declaration.
  ///
  /// # Panics
  ///
  /// Panics if the given name not starts with `%` or `@`, or the given
  /// type is not a valid function type.
  pub fn new_decl(name: String, ty: Type) -> Self {
    Self::check_sanity(&name, &ty);
    Self {
      ty,
      name,
      params: Vec::new(),
      dfg: DataFlowGraph::new(),
      layout: Layout::new(),
    }
  }

  /// Checks if the given name and type is valid.
  ///
  /// # Panics
  ///
  /// Panics if the given name and type is invalid.
  fn check_sanity(name: &str, ty: &Type) {
    assert!(
      name.len() > 1 && (name.starts_with('%') || name.starts_with('@')),
      "invalid function name"
    );
    match ty.kind() {
      TypeKind::Function(params, _) => {
        assert!(
          params.iter().all(|p| !p.is_unit()),
          "parameter type must not be `unit`!"
        )
      }
      _ => panic!("expected a function type!"),
    };
  }

  /// Returns a reference to the function's type.
  pub fn ty(&self) -> &Type {
    &self.ty
  }

  /// Returns a reference to the function's name.
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Returns a reference to the function parameters.
  pub fn params(&self) -> &[Value] {
    &self.params
  }

  /// Returns a reference to the data flow graph.
  pub fn dfg(&self) -> &DataFlowGraph {
    &self.dfg
  }

  /// Returns a mutable reference to the data flow graph.
  pub fn dfg_mut(&mut self) -> &mut DataFlowGraph {
    &mut self.dfg
  }

  /// Returns a reference to the layout.
  pub fn layout(&self) -> &Layout {
    &self.layout
  }

  /// Returns a mutable reference to the layout.
  pub fn layout_mut(&mut self) -> &mut Layout {
    &mut self.layout
  }
}

/// A handle of Koopa IR basic block.
///
/// You can fetch `BasicBlockData` from [`DataFlowGraph`] in [`FunctionData`]
/// by using this handle.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BasicBlock(pub(crate) BasicBlockId);

/// Data of Koopa IR basic block.
///
/// `BasicBlockData` only holds parameters about this basic block, and
/// which values (branch/jump instructions) the current basic block is
/// used by. Other information, such as the data and order of instructions
/// in this basic block, can be found in [`FunctionData`] (in the data flow
/// graph or the layout).
pub struct BasicBlockData {
  ty: Type,
  name: Option<String>,
  params: Vec<Value>,
  pub(crate) used_by: HashSet<Value>,
}

impl BasicBlockData {
  /// Creates a new `BasicBlockData` with the given name.
  ///
  /// # Panics
  ///
  /// Panics if the given name (if exists) not starts with `%` or `@`.
  pub fn new(name: Option<String>) -> Self {
    assert!(
      name.as_ref().map_or(true, |n| n.len() > 1
        && (n.starts_with('%') || n.starts_with('@'))),
      "invalid basic block name"
    );
    Self {
      ty: Type::get_basic_block(Vec::new()),
      name,
      params: Vec::new(),
      used_by: HashSet::new(),
    }
  }

  /// Creates a new `BasicBlockData` with the given name, parameters and type.
  ///
  /// # Panics
  ///
  /// Panics if `ty` is not a valid basic block type.
  pub fn with_params(name: Option<String>, params: Vec<Value>, ty: Type) -> Self {
    assert!(
      matches!(ty.kind(), TypeKind::BasicBlock(p) if p.len() == params.len()),
      "`ty` is not a valid basic block type"
    );
    Self {
      ty,
      name,
      params,
      used_by: HashSet::new(),
    }
  }

  /// Returns a reference to the basic block's type.
  pub fn ty(&self) -> &Type {
    &self.ty
  }

  /// Returns a reference to the basic block's name.
  pub fn name(&self) -> &Option<String> {
    &self.name
  }

  /// Returns a reference to the basic block parameters.
  pub fn params(&self) -> &[Value] {
    &self.params
  }

  /// Returns a reference to the values that the current basic block
  /// is used by.
  pub fn used_by(&self) -> &HashSet<Value> {
    &self.used_by
  }
}

impl Default for BasicBlockData {
  /// Creates a `BasicBlockData` without name and parameters.
  fn default() -> Self {
    Self {
      ty: Type::get_basic_block(Vec::default()),
      name: None,
      params: Vec::default(),
      used_by: HashSet::default(),
    }
  }
}

/// A handle of Koopa IR value.
///
/// You can fetch `ValueData` from [`DataFlowGraph`] in [`FunctionData`]
/// by using this handle.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Value(pub(crate) ValueId);

/// Data of Koopa IR value.
///
/// `ValueData` can hold the type and the kind of the value, and which
/// values the current value is used by.
pub struct ValueData {
  ty: Type,
  name: Option<String>,
  kind: ValueKind,
  pub(crate) used_by: HashSet<Value>,
}

impl ValueData {
  /// Creates a new `ValueData` with the given type and kind.
  pub(crate) fn new(ty: Type, kind: ValueKind) -> Self {
    Self {
      ty,
      name: None,
      kind,
      used_by: HashSet::new(),
    }
  }

  /// Returns a reference to the value's type.
  pub fn ty(&self) -> &Type {
    &self.ty
  }

  /// Returns a reference to the value's name.
  pub fn name(&self) -> &Option<String> {
    &self.name
  }

  /// Sets the name of this value.
  pub fn set_name(&mut self, name: Option<String>) {
    self.name = name;
  }

  /// Returns a reference to the value's kind.
  pub fn kind(&self) -> &ValueKind {
    &self.kind
  }

  /// Returns a reference to the values that the current value is used by.
  pub fn used_by(&self) -> &HashSet<Value> {
    &self.used_by
  }
}

/// Kind of Koopa IR value.
pub enum ValueKind {
  Integer(values::Integer),
  ZeroInit(values::ZeroInit),
  Undef(values::Undef),
  Aggregate(values::Aggregate),
  FuncArgRef(values::FuncArgRef),
  BlockArgRef(values::BlockArgRef),
  Alloc(values::Alloc),
  GlobalAlloc(values::GlobalAlloc),
  Load(values::Load),
  Store(values::Store),
  GetPtr(values::GetPtr),
  GetElemPtr(values::GetElemPtr),
  Binary(values::Binary),
  Branch(values::Branch),
  Jump(values::Jump),
  Call(values::Call),
  Return(values::Return),
}

impl ValueKind {
  /// Returns an iterator of all values that used by the `ValueKind`.
  pub fn value_uses(&self) -> ValueUses {
    ValueUses {
      kind: self,
      index: 0,
    }
  }

  /// Returns an iterator of all basic blocks that used by the `ValueKind`.
  pub fn bb_uses(&self) -> BasicBlockUses {
    BasicBlockUses {
      kind: self,
      index: 0,
    }
  }
}

/// An iterator over all values that used by a [`ValueKind`].
pub struct ValueUses<'a> {
  kind: &'a ValueKind,
  index: usize,
}

impl<'a> Iterator for ValueUses<'a> {
  type Item = Value;

  fn next(&mut self) -> Option<Self::Item> {
    let cur = self.index;
    self.index += 1;
    macro_rules! vec_use {
      ($vec:expr) => {
        if cur < $vec.len() {
          Some($vec[cur])
        } else {
          None
        }
      };
    }
    macro_rules! field_use {
      ($($field:expr),+) => {
        field_use!(@expand 0 $(,$field)+)
      };
      (@expand $index:expr) => {
        None
      };
      (@expand $index:expr, $head:expr $(,$tail:expr)*) => {
        if cur == $index {
          Some($head)
        } else {
          field_use!(@expand $index + 1 $(,$tail)*)
        }
      };
    }
    match self.kind {
      ValueKind::Aggregate(v) => vec_use!(v.elems()),
      ValueKind::GlobalAlloc(v) => field_use!(v.init()),
      ValueKind::Load(v) => field_use!(v.src()),
      ValueKind::Store(v) => field_use!(v.value(), v.dest()),
      ValueKind::GetPtr(v) => field_use!(v.src(), v.index()),
      ValueKind::GetElemPtr(v) => field_use!(v.src(), v.index()),
      ValueKind::Binary(v) => field_use!(v.lhs(), v.rhs()),
      ValueKind::Branch(v) => {
        let tlen = v.true_args().len();
        if cur == 0 {
          Some(v.cond())
        } else if cur >= 1 && cur <= tlen {
          Some(v.true_args()[cur - 1])
        } else if cur > tlen && cur <= tlen + v.false_args().len() {
          Some(v.false_args()[cur - tlen - 1])
        } else {
          None
        }
      }
      ValueKind::Jump(v) => vec_use!(v.args()),
      ValueKind::Call(v) => vec_use!(v.args()),
      ValueKind::Return(v) => match cur {
        0 => v.value(),
        _ => None,
      },
      _ => None,
    }
  }
}

/// An iterator over all basic blocks that used by a [`ValueKind`].
pub struct BasicBlockUses<'a> {
  kind: &'a ValueKind,
  index: usize,
}

impl<'a> Iterator for BasicBlockUses<'a> {
  type Item = BasicBlock;

  fn next(&mut self) -> Option<Self::Item> {
    let cur = self.index;
    self.index += 1;
    match self.kind {
      ValueKind::Branch(br) => match cur {
        0 => Some(br.true_bb()),
        1 => Some(br.false_bb()),
        _ => None,
      },
      ValueKind::Jump(jump) => match cur {
        0 => Some(jump.target()),
        _ => None,
      },
      _ => None,
    }
  }
}
