#![allow(clippy::new_ret_no_self)]

use crate::front::span::Span;
use crate::ir::instructions::{BinaryOp, UnaryOp};
use std::cmp::PartialEq;

/// AST of Koopa IR.
#[derive(Debug)]
pub struct Ast {
  pub span: Span,
  pub kind: AstKind,
}

/// Box of AST.
pub type AstBox = Box<Ast>;

impl Ast {
  /// Creates a new AST box.
  pub(crate) fn new(span: Span, kind: AstKind) -> AstBox {
    Box::new(Self { span, kind })
  }
}

impl PartialEq for Ast {
  fn eq(&self, other: &Self) -> bool {
    // ignore field `span`
    self.kind == other.kind
  }
}

/// Kind of AST.
#[derive(Debug, PartialEq)]
pub enum AstKind {
  IntType(IntType),
  ArrayType(ArrayType),
  PointerType(PointerType),
  FunType(FunType),
  SymbolRef(SymbolRef),
  IntVal(IntVal),
  UndefVal(UndefVal),
  Aggregate(Aggregate),
  ZeroInit(ZeroInit),
  SymbolDef(SymbolDef),
  GlobalDef(GlobalDef),
  MemDecl(MemDecl),
  GlobalDecl(GlobalDecl),
  Load(Load),
  Store(Store),
  GetPointer(GetPointer),
  GetElementPointer(GetElementPointer),
  BinaryExpr(BinaryExpr),
  UnaryExpr(UnaryExpr),
  Branch(Branch),
  Jump(Jump),
  FunCall(FunCall),
  Return(Return),
  FunDef(FunDef),
  Block(Block),
  FunDecl(FunDecl),
  Phi(Phi),
  End(End),
  Error(Error),
}

/// 32-bit integer type.
#[derive(Debug, PartialEq)]
pub struct IntType;

impl IntType {
  /// Creates a new box of `IntType` AST.
  pub fn new(span: Span) -> AstBox {
    Ast::new(span, AstKind::IntType(Self))
  }
}

/// Array type.
#[derive(Debug, PartialEq)]
pub struct ArrayType {
  pub base: AstBox,
  pub len: usize,
}

impl ArrayType {
  /// Creates a new box of `ArrayType` AST.
  pub fn new(span: Span, base: AstBox, len: usize) -> AstBox {
    Ast::new(span, AstKind::ArrayType(Self { base, len }))
  }
}

/// Pointer type.
#[derive(Debug, PartialEq)]
pub struct PointerType {
  pub base: AstBox,
}

impl PointerType {
  /// Creates a new box of `PointerType` AST.
  pub fn new(span: Span, base: AstBox) -> AstBox {
    Ast::new(span, AstKind::PointerType(Self { base }))
  }
}

/// Function type.
#[derive(Debug, PartialEq)]
pub struct FunType {
  pub params: Vec<AstBox>,
  pub ret: Option<AstBox>,
}

impl FunType {
  /// Creates a new box of `FunType` AST.
  pub fn new(span: Span, params: Vec<AstBox>, ret: Option<AstBox>) -> AstBox {
    Ast::new(span, AstKind::FunType(Self { params, ret }))
  }
}

/// Symbol refernce.
#[derive(Debug, PartialEq)]
pub struct SymbolRef {
  pub symbol: String,
}

impl SymbolRef {
  /// Creates a new box of `SymbolRef` AST.
  pub fn new(span: Span, symbol: String) -> AstBox {
    Ast::new(span, AstKind::SymbolRef(Self { symbol }))
  }
}

/// Integer literal.
#[derive(Debug, PartialEq)]
pub struct IntVal {
  pub value: i32,
}

impl IntVal {
  /// Creates a new box of `IntVal` AST.
  pub fn new(span: Span, value: i32) -> AstBox {
    Ast::new(span, AstKind::IntVal(Self { value }))
  }
}

/// Undefined value.
#[derive(Debug, PartialEq)]
pub struct UndefVal;

impl UndefVal {
  /// Creates a new box of `UndefVal` AST.
  pub fn new(span: Span) -> AstBox {
    Ast::new(span, AstKind::UndefVal(Self))
  }
}

/// Aggregate value.
#[derive(Debug, PartialEq)]
pub struct Aggregate {
  pub elems: Vec<AstBox>,
}

impl Aggregate {
  /// Creates a new box of `Aggregate` AST.
  pub fn new(span: Span, elems: Vec<AstBox>) -> AstBox {
    Ast::new(span, AstKind::Aggregate(Self { elems }))
  }
}

/// Zero initializer.
#[derive(Debug, PartialEq)]
pub struct ZeroInit;

impl ZeroInit {
  /// Creates a new box of `ZeroInit` AST.
  pub fn new(span: Span) -> AstBox {
    Ast::new(span, AstKind::ZeroInit(Self))
  }
}

/// Symbol definition.
#[derive(Debug, PartialEq)]
pub struct SymbolDef {
  pub name: String,
  pub value: AstBox,
}

impl SymbolDef {
  /// Creates a new box of `SymbolDef` AST.
  pub fn new(span: Span, name: String, value: AstBox) -> AstBox {
    Ast::new(span, AstKind::SymbolDef(Self { name, value }))
  }
}

/// Global symbol definition.
#[derive(Debug, PartialEq)]
pub struct GlobalDef {
  pub name: String,
  pub value: AstBox,
}

impl GlobalDef {
  /// Creates a new box of `GlobalDef` AST.
  pub fn new(span: Span, name: String, value: AstBox) -> AstBox {
    Ast::new(span, AstKind::GlobalDef(Self { name, value }))
  }
}

/// Memory declaration.
#[derive(Debug, PartialEq)]
pub struct MemDecl {
  pub ty: AstBox,
}

impl MemDecl {
  /// Creates a new box of `MemDecl` AST.
  pub fn new(span: Span, ty: AstBox) -> AstBox {
    Ast::new(span, AstKind::MemDecl(Self { ty }))
  }
}

/// Global memory declaration.
#[derive(Debug, PartialEq)]
pub struct GlobalDecl {
  pub ty: AstBox,
  pub init: AstBox,
}

impl GlobalDecl {
  /// Creates a new box of `GlobalDecl` AST.
  pub fn new(span: Span, ty: AstBox, init: AstBox) -> AstBox {
    Ast::new(span, AstKind::GlobalDecl(Self { ty, init }))
  }
}

/// Load.
#[derive(Debug, PartialEq)]
pub struct Load {
  pub symbol: String,
}

impl Load {
  /// Creates a new box of `Load` AST.
  pub fn new(span: Span, symbol: String) -> AstBox {
    Ast::new(span, AstKind::Load(Self { symbol }))
  }
}

/// Store.
#[derive(Debug, PartialEq)]
pub struct Store {
  pub value: AstBox,
  pub symbol: String,
}

impl Store {
  /// Creates a new box of `Store` AST.
  pub fn new(span: Span, value: AstBox, symbol: String) -> AstBox {
    Ast::new(span, AstKind::Store(Self { value, symbol }))
  }
}

/// Pointer calculation.
#[derive(Debug, PartialEq)]
pub struct GetPointer {
  pub symbol: String,
  pub value: AstBox,
}

impl GetPointer {
  /// Creates a new box of `GetPointer` AST.
  pub fn new(span: Span, symbol: String, value: AstBox) -> AstBox {
    Ast::new(span, AstKind::GetPointer(Self { symbol, value }))
  }
}

/// Element pointer calculation.
#[derive(Debug, PartialEq)]
pub struct GetElementPointer {
  pub symbol: String,
  pub value: AstBox,
}

impl GetElementPointer {
  /// Creates a new box of `GetElementPointer` AST.
  pub fn new(span: Span, symbol: String, value: AstBox) -> AstBox {
    Ast::new(span, AstKind::GetElementPointer(Self { symbol, value }))
  }
}

/// Binary expression.
#[derive(Debug, PartialEq)]
pub struct BinaryExpr {
  pub op: BinaryOp,
  pub lhs: AstBox,
  pub rhs: AstBox,
}

impl BinaryExpr {
  /// Creates a new box of `BinaryExpr` AST.
  pub fn new(span: Span, op: BinaryOp, lhs: AstBox, rhs: AstBox) -> AstBox {
    Ast::new(span, AstKind::BinaryExpr(Self { op, lhs, rhs }))
  }
}

/// Unary expression.
#[derive(Debug, PartialEq)]
pub struct UnaryExpr {
  pub op: UnaryOp,
  pub opr: AstBox,
}

impl UnaryExpr {
  /// Creates a new box of `UnaryExpr` AST.
  pub fn new(span: Span, op: UnaryOp, opr: AstBox) -> AstBox {
    Ast::new(span, AstKind::UnaryExpr(Self { op, opr }))
  }
}

/// Branch.
#[derive(Debug, PartialEq)]
pub struct Branch {
  pub cond: AstBox,
  pub tbb: String,
  pub fbb: String,
}

impl Branch {
  /// Creates a new box of `Branch` AST.
  pub fn new(span: Span, cond: AstBox, tbb: String, fbb: String) -> AstBox {
    Ast::new(span, AstKind::Branch(Self { cond, tbb, fbb }))
  }
}

/// Jump.
#[derive(Debug, PartialEq)]
pub struct Jump {
  pub target: String,
}

impl Jump {
  /// Creates a new box of `Jump` AST.
  pub fn new(span: Span, target: String) -> AstBox {
    Ast::new(span, AstKind::Jump(Self { target }))
  }
}

/// Function call.
#[derive(Debug, PartialEq)]
pub struct FunCall {
  pub fun: String,
  pub args: Vec<AstBox>,
}

impl FunCall {
  /// Creates a new box of `FunCall` AST.
  pub fn new(span: Span, fun: String, args: Vec<AstBox>) -> AstBox {
    Ast::new(span, AstKind::FunCall(Self { fun, args }))
  }
}

/// Return.
#[derive(Debug, PartialEq)]
pub struct Return {
  pub value: Option<AstBox>,
}

impl Return {
  /// Creates a new box of `Return` AST.
  pub fn new(span: Span, value: Option<AstBox>) -> AstBox {
    Ast::new(span, AstKind::Return(Self { value }))
  }
}

/// Function definition.
#[derive(Debug, PartialEq)]
pub struct FunDef {
  pub name: String,
  pub params: Vec<(String, AstBox)>,
  pub ret: Option<AstBox>,
  pub bbs: Vec<AstBox>,
}

impl FunDef {
  /// Creates a new box of `FunDef` AST.
  pub fn new(
    span: Span,
    name: String,
    params: Vec<(String, AstBox)>,
    ret: Option<AstBox>,
    bbs: Vec<AstBox>,
  ) -> AstBox {
    Ast::new(
      span,
      AstKind::FunDef(Self {
        name,
        params,
        ret,
        bbs,
      }),
    )
  }
}

/// Basic block.
#[derive(Debug, PartialEq)]
pub struct Block {
  pub name: String,
  pub stmts: Vec<AstBox>,
}

impl Block {
  /// Creates a new box of `Block` AST.
  pub fn new(span: Span, name: String, stmts: Vec<AstBox>) -> AstBox {
    Ast::new(span, AstKind::Block(Self { name, stmts }))
  }
}

/// Function declaration.
#[derive(Debug, PartialEq)]
pub struct FunDecl {
  pub name: String,
  pub params: Vec<AstBox>,
  pub ret: Option<AstBox>,
}

impl FunDecl {
  /// Creates a new box of `FunDecl` AST.
  pub fn new(span: Span, name: String, params: Vec<AstBox>, ret: Option<AstBox>) -> AstBox {
    Ast::new(span, AstKind::FunDecl(Self { name, params, ret }))
  }
}

/// Phi function.
#[derive(Debug, PartialEq)]
pub struct Phi {
  pub ty: AstBox,
  pub oprs: Vec<(AstBox, String)>,
}

impl Phi {
  /// Creates a new box of `Phi` AST.
  pub fn new(span: Span, ty: AstBox, oprs: Vec<(AstBox, String)>) -> AstBox {
    Ast::new(span, AstKind::Phi(Self { ty, oprs }))
  }
}

/// End of file.
#[derive(Debug, PartialEq)]
pub struct End;

impl End {
  /// Creates a new box of `End` AST.
  pub fn new(span: Span) -> AstBox {
    Ast::new(span, AstKind::End(Self))
  }
}

/// Error
#[derive(Debug, PartialEq)]
pub struct Error;

impl Error {
  /// Creates a new box of `Error` AST.
  pub fn new(span: Span) -> AstBox {
    Ast::new(span, AstKind::Error(Self))
  }
}
