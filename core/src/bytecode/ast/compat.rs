//! Compatibility with the current stable Nickel version.
//!
//! This module defines a trait for converting to and from the representation used in stable Nickel
//! to the new AST representation of the bytecode compiler, and implements it for the types defined
//! in [crate::bytecode::ast].

use super::{primop::PrimOp, *};
use crate::{label, term, typ as mline_type};
use smallvec::SmallVec;

/// Convert from the mainline Nickel representation to the new AST representation. This trait is
/// mostly `From` with an additional argument for the allocator.
///
/// # Parameters
///
/// - `'ast`: the lifetime of the AST nodes, tied to the allocator
/// - `'a`: the lifetime of the reference to the mainline Nickel object, which doesn't need to be
///   related to `'ast` (we will copy any required data into the allocator)
/// - `T`: the type of the mainline Nickel object ([term::Term], [term::pattern::Pattern], etc.)
pub trait FromMainline<'ast, T> {
    fn from_mainline(alloc: &'ast AstAlloc, mainline: &T) -> Self;
}

impl<'ast> FromMainline<'ast, term::pattern::Pattern> for &'ast Pattern<'ast> {
    fn from_mainline(
        alloc: &'ast AstAlloc,
        pattern: &term::pattern::Pattern,
    ) -> &'ast Pattern<'ast> {
        alloc.pattern(pattern.to_ast(alloc))
    }
}

impl<'ast> FromMainline<'ast, term::pattern::Pattern> for Pattern<'ast> {
    fn from_mainline(alloc: &'ast AstAlloc, pattern: &term::pattern::Pattern) -> Self {
        Pattern {
            data: pattern.data.to_ast(alloc),
            alias: pattern.alias,
            pos: pattern.pos,
        }
    }
}

impl<'ast> FromMainline<'ast, term::pattern::PatternData> for PatternData<'ast> {
    fn from_mainline(alloc: &'ast AstAlloc, data: &term::pattern::PatternData) -> Self {
        match data {
            term::pattern::PatternData::Wildcard => PatternData::Wildcard,
            term::pattern::PatternData::Any(id) => PatternData::Any(*id),
            term::pattern::PatternData::Record(record_pattern) => record_pattern.to_ast(alloc),
            term::pattern::PatternData::Array(array_pattern) => array_pattern.to_ast(alloc),
            term::pattern::PatternData::Enum(enum_pattern) => enum_pattern.to_ast(alloc),
            term::pattern::PatternData::Constant(constant_pattern) => {
                constant_pattern.to_ast(alloc)
            }
            term::pattern::PatternData::Or(or_pattern) => or_pattern.to_ast(alloc),
        }
    }
}

impl<'ast> FromMainline<'ast, term::pattern::RecordPattern> for PatternData<'ast> {
    fn from_mainline(alloc: &'ast AstAlloc, record_pat: &term::pattern::RecordPattern) -> Self {
        let patterns = record_pat
            .patterns
            .iter()
            .map(|field_pattern| field_pattern.to_ast(alloc));

        let tail = match record_pat.tail {
            term::pattern::TailPattern::Empty => TailPattern::Empty,
            term::pattern::TailPattern::Open => TailPattern::Open,
            term::pattern::TailPattern::Capture(id) => TailPattern::Capture(id),
        };

        PatternData::Record(alloc.record_pattern(patterns, tail, record_pat.pos))
    }
}

impl<'ast> FromMainline<'ast, term::pattern::FieldPattern> for FieldPattern<'ast> {
    fn from_mainline(alloc: &'ast AstAlloc, field_pat: &term::pattern::FieldPattern) -> Self {
        let pattern = field_pat.pattern.to_ast(alloc);

        let default = field_pat.default.as_ref().map(|term| term.to_ast(alloc));

        let annotation = field_pat.annotation.to_ast(alloc);

        FieldPattern {
            matched_id: field_pat.matched_id,
            annotation,
            default,
            pattern,
            pos: field_pat.pos,
        }
    }
}

impl<'ast> FromMainline<'ast, term::pattern::ArrayPattern> for PatternData<'ast> {
    fn from_mainline(alloc: &'ast AstAlloc, array_pat: &term::pattern::ArrayPattern) -> Self {
        let patterns = array_pat.patterns.iter().map(|pat| pat.to_ast(alloc));

        let tail = match array_pat.tail {
            term::pattern::TailPattern::Empty => TailPattern::Empty,
            term::pattern::TailPattern::Open => TailPattern::Open,
            term::pattern::TailPattern::Capture(id) => TailPattern::Capture(id),
        };

        PatternData::Array(alloc.array_pattern(patterns, tail, array_pat.pos))
    }
}

impl<'ast> FromMainline<'ast, term::pattern::EnumPattern> for PatternData<'ast> {
    fn from_mainline(alloc: &'ast AstAlloc, enum_pat: &term::pattern::EnumPattern) -> Self {
        let pattern = enum_pat.pattern.as_ref().map(|pat| (**pat).to_ast(alloc));
        PatternData::Enum(alloc.enum_pattern(EnumPattern {
            tag: enum_pat.tag,
            pattern,
            pos: enum_pat.pos,
        }))
    }
}

impl<'ast> FromMainline<'ast, term::pattern::ConstantPattern> for PatternData<'ast> {
    fn from_mainline(alloc: &'ast AstAlloc, pattern: &term::pattern::ConstantPattern) -> Self {
        let data = match &pattern.data {
            term::pattern::ConstantPatternData::Bool(b) => ConstantPatternData::Bool(*b),
            term::pattern::ConstantPatternData::Number(n) => {
                ConstantPatternData::Number(alloc.generic_arena.alloc(n.clone()))
            }
            term::pattern::ConstantPatternData::String(s) => {
                ConstantPatternData::String(alloc.generic_arena.alloc_str(s))
            }
            term::pattern::ConstantPatternData::Null => ConstantPatternData::Null,
        };

        PatternData::Constant(alloc.constant_pattern(ConstantPattern {
            data,
            pos: pattern.pos,
        }))
    }
}

impl<'ast> FromMainline<'ast, term::pattern::OrPattern> for PatternData<'ast> {
    fn from_mainline(alloc: &'ast AstAlloc, pattern: &term::pattern::OrPattern) -> Self {
        let patterns = pattern
            .patterns
            .iter()
            .map(|pat| pat.to_ast(alloc))
            .collect::<Vec<_>>();

        PatternData::Or(alloc.or_pattern(patterns, pattern.pos))
    }
}

impl<'ast> FromMainline<'ast, term::TypeAnnotation> for Annotation<'ast> {
    fn from_mainline(alloc: &'ast AstAlloc, annot: &term::TypeAnnotation) -> Self {
        let typ = annot.typ.as_ref().map(|typ| typ.typ.to_ast(alloc));

        let contracts = alloc.types(
            annot
                .contracts
                .iter()
                .map(|contract| contract.typ.to_ast(alloc)),
        );

        Annotation { typ, contracts }
    }
}

impl<'ast> FromMainline<'ast, term::record::Field> for record::Field<'ast> {
    fn from_mainline(alloc: &'ast AstAlloc, field: &term::record::Field) -> Self {
        record::Field {
            value: field.value.as_ref().map(|term| term.to_ast(alloc)),
            metadata: field.metadata.to_ast(alloc),
        }
    }
}

impl<'ast> FromMainline<'ast, term::record::FieldMetadata> for record::FieldMetadata<'ast> {
    fn from_mainline(alloc: &'ast AstAlloc, metadata: &term::record::FieldMetadata) -> Self {
        let doc = metadata.doc.as_ref().map(|doc| rc::Rc::from(doc.as_str()));

        record::FieldMetadata {
            doc,
            annotation: metadata.annotation.to_ast(alloc),
            opt: metadata.opt,
            not_exported: metadata.not_exported,
            priority: metadata.priority.clone(),
        }
    }
}

impl<'ast> FromMainline<'ast, mline_type::Type> for Type<'ast> {
    fn from_mainline(alloc: &'ast AstAlloc, mainline: &mline_type::Type) -> Self {
        Type {
            typ: mainline.typ.to_ast(alloc),
            pos: mainline.pos,
        }
    }
}

type MainlineTypeUnr = mline_type::TypeF<
    Box<mline_type::Type>,
    mline_type::RecordRows,
    mline_type::EnumRows,
    term::RichTerm,
>;

impl<'ast> FromMainline<'ast, MainlineTypeUnr> for TypeUnr<'ast> {
    fn from_mainline(alloc: &'ast AstAlloc, typ: &MainlineTypeUnr) -> Self {
        typ.clone().map(
            |typ| &*alloc.generic_arena.alloc((*typ).to_ast(alloc)),
            |rrows| rrows.to_ast(alloc),
            |erows| erows.to_ast(alloc),
            |ctr| ctr.to_ast(alloc),
        )
    }
}

impl<'ast> FromMainline<'ast, mline_type::RecordRows> for RecordRows<'ast> {
    fn from_mainline(alloc: &'ast AstAlloc, rrows: &mline_type::RecordRows) -> Self {
        RecordRows(rrows.0.to_ast(alloc))
    }
}

impl<'ast> FromMainline<'ast, mline_type::EnumRows> for EnumRows<'ast> {
    fn from_mainline(alloc: &'ast AstAlloc, erows: &mline_type::EnumRows) -> Self {
        EnumRows(erows.0.to_ast(alloc))
    }
}

type MainlineEnumRowsUnr = mline_type::EnumRowsF<Box<mline_type::Type>, Box<mline_type::EnumRows>>;

impl<'ast> FromMainline<'ast, MainlineEnumRowsUnr> for EnumRowsUnr<'ast> {
    fn from_mainline(alloc: &'ast AstAlloc, erows: &MainlineEnumRowsUnr) -> Self {
        erows.clone().map(
            |typ| &*alloc.generic_arena.alloc((*typ).to_ast(alloc)),
            |erows| &*alloc.generic_arena.alloc((*erows).to_ast(alloc)),
        )
    }
}

type MainlineRecordRowsUnr =
    mline_type::RecordRowsF<Box<mline_type::Type>, Box<mline_type::RecordRows>>;

impl<'ast> FromMainline<'ast, MainlineRecordRowsUnr> for RecordRowsUnr<'ast> {
    fn from_mainline(alloc: &'ast AstAlloc, rrows: &MainlineRecordRowsUnr) -> Self {
        rrows.clone().map(
            |typ| &*alloc.generic_arena.alloc((*typ).to_ast(alloc)),
            |rrows| &*alloc.generic_arena.alloc((*rrows).to_ast(alloc)),
        )
    }
}

impl<'ast> FromMainline<'ast, term::Term> for Node<'ast> {
    fn from_mainline(alloc: &'ast AstAlloc, term: &term::Term) -> Self {
        use term::Term;

        match term {
            Term::Null => Node::Null,
            Term::Bool(b) => Node::Bool(*b),
            Term::Num(n) => alloc.number(n.clone()),
            Term::Str(s) => alloc.string(s),
            Term::StrChunks(chunks) => alloc.str_chunks(
                chunks
                    .iter()
                    .map(|chunk| match chunk {
                        term::StrChunk::Literal(s) => StrChunk::Literal(s.clone()),
                        term::StrChunk::Expr(expr, indent) => {
                            StrChunk::Expr(expr.to_ast(alloc), *indent)
                        }
                    })
                    .rev(),
            ),
            Term::Fun(id, body) => alloc.fun(Pattern::any(*id), body.to_ast(alloc)),
            Term::FunPattern(pat, body) => alloc.fun(pat.to_ast(alloc), body.to_ast(alloc)),
            Term::Let(bindings, body, attrs) => alloc.let_binding(
                bindings
                    .iter()
                    .map(|(id, term)| (Pattern::any(*id), term.to_ast(alloc))),
                body.to_ast(alloc),
                attrs.rec,
            ),
            Term::LetPattern(bindings, body, attrs) => alloc.let_binding(
                bindings
                    .iter()
                    .map(|(pat, term)| (pat.to_ast(alloc), term.to_ast(alloc))),
                body.to_ast(alloc),
                attrs.rec,
            ),
            Term::App(fun, arg) => {
                match fun.as_ref() {
                    // We have to special-case if-then-else, which is encoded as a primop application
                    // of the unary operator if-then-else to the condition, followed by two normal
                    // applications for the if and else branches, which is a bit painful to match.
                    Term::App(fun_inner, arg_inner)
                        if matches!(
                            fun_inner.as_ref(),
                            Term::Op1(term::UnaryOp::IfThenElse, _)
                        ) =>
                    {
                        if let Term::Op1(term::UnaryOp::IfThenElse, cond) = fun_inner.as_ref() {
                            return alloc.if_then_else(
                                cond.to_ast(alloc),
                                arg_inner.to_ast(alloc),
                                arg.to_ast(alloc),
                            );
                        }
                    }
                    _ => (),
                };

                let mut args = vec![arg.to_ast(alloc)];
                let mut maybe_next_app = fun.as_ref();

                while let Term::App(next_fun, next_arg) = maybe_next_app {
                    args.push(next_arg.to_ast(alloc));
                    maybe_next_app = next_fun.as_ref();
                }

                alloc.app(fun.to_ast(alloc), args.into_iter().rev())
            }
            Term::Var(id) => Node::Var(*id),
            Term::Enum(id) => alloc.enum_variant(*id, None),
            Term::EnumVariant { tag, arg, attrs: _ } => {
                alloc.enum_variant(*tag, Some(arg.to_ast(alloc)))
            }
            Term::RecRecord(data, dyn_fields, _deps) => {
                let stat_fields = alloc.generic_arena.alloc_slice_fill_iter(
                    data.fields
                        .iter()
                        .map(|(id, field)| (*id, field.to_ast(alloc))),
                );

                let dyn_fields = alloc.generic_arena.alloc_slice_fill_iter(
                    dyn_fields
                        .iter()
                        .map(|(expr, field)| (expr.to_ast(alloc), field.to_ast(alloc))),
                );

                let open = data.attrs.open;

                alloc.record(Record {
                    stat_fields,
                    dyn_fields,
                    open,
                })
            }
            Term::Record(data) => {
                let stat_fields = alloc.generic_arena.alloc_slice_fill_iter(
                    data.fields
                        .iter()
                        .map(|(id, field)| (*id, field.to_ast(alloc))),
                );

                let open = data.attrs.open;

                alloc.record(Record {
                    stat_fields,
                    dyn_fields: alloc
                        .generic_arena
                        .alloc_slice_fill_iter(std::iter::empty()),
                    open,
                })
            }
            Term::Match(data) => {
                let branches = data.branches.iter().map(|branch| MatchBranch {
                    pattern: branch.pattern.to_ast(alloc),
                    guard: branch.guard.as_ref().map(|term| term.to_ast(alloc)),
                    body: branch.body.to_ast(alloc),
                });

                alloc.match_expr(branches)
            }
            Term::Array(data, _attrs) => {
                // We should probably make array's iterator an ExactSizeIterator. But for now, we
                // don't care about the translation's performance so it's simpler to just collect
                // them in a vec locally.
                let elts = data
                    .iter()
                    .map(|term| term.to_ast(alloc))
                    .collect::<Vec<_>>();
                alloc.array(elts)
            }
            Term::Op1(op, arg) => {
                alloc.prim_op(PrimOp::from(op), std::iter::once(arg.to_ast(alloc)))
            }
            Term::Op2(op, arg1, arg2) => {
                // [^primop-argument-order]: Some primops have had exotic arguments order for
                // historical reasons. The new AST tries to follow the stdlib argument order
                // whenever possible, which means we have to swap the arguments for a few primops.

                let op = PrimOp::from(op);
                let mut args = [arg1.to_ast(alloc), arg2.to_ast(alloc)];

                if matches!(op, PrimOp::ArrayAt | PrimOp::StringContains) {
                    args.swap(0, 1);
                }

                alloc.prim_op(op, args)
            }
            Term::OpN(op, args) => {
                // See [^primop-argument-order].
                let op = PrimOp::from(op);
                let mut args: Vec<_> = args.iter().map(|arg| arg.to_ast(alloc)).collect();
                if let PrimOp::StringSubstr = op {
                    debug_assert_eq!(args.len(), 3);
                    // The original order is: the string, then start and end.
                    // The target order is: start, end and then the string.
                    args.swap(0, 1);
                    args.swap(1, 2);
                }

                alloc.prim_op(op, args)
            }
            Term::SealingKey(_) => panic!("didn't expect a sealing key at the first stage"),
            Term::Sealed(..) => panic!("didn't expect a sealed term at the first stage"),
            Term::Annotated(annot, term) => {
                alloc.annotated(annot.to_ast(alloc), term.to_ast(alloc))
            }
            Term::Import { path, format } => alloc.import(path.clone(), *format),
            Term::ResolvedImport(_) => panic!("didn't expect a resolved import at parsing stage"),
            Term::Type { typ, .. } => alloc.typ(typ.to_ast(alloc)),
            Term::CustomContract(_) => panic!("didn't expect a custom contract at parsing stage"),
            Term::ParseError(error) => alloc.parse_error(error.clone()),
            Term::RuntimeError(_) => panic!("didn't expect a runtime error at parsing stage"),
            Term::Closure(_) => panic!("didn't expect a closure at parsing stage"),
            Term::ForeignId(_) => panic!("didn't expect a foreign id at parsing stage"),
            _ => unimplemented!(),
        }
    }
}

impl<'ast> FromMainline<'ast, term::RichTerm> for Ast<'ast> {
    fn from_mainline(alloc: &'ast AstAlloc, rterm: &term::RichTerm) -> Self {
        Ast {
            node: rterm.as_ref().to_ast(alloc),
            pos: rterm.pos,
        }
    }
}

impl<'ast> FromMainline<'ast, term::RichTerm> for &'ast Ast<'ast> {
    fn from_mainline(alloc: &'ast AstAlloc, rterm: &term::RichTerm) -> Self {
        alloc.ast(rterm.to_ast(alloc))
    }
}

/// Symmetric to `FromMainline`, as `Into` is to `From`.
pub trait ToAst<'ast, T> {
    fn to_ast(&self, alloc: &'ast AstAlloc) -> T;
}

impl<'ast, S, T> ToAst<'ast, T> for S
where
    T: FromMainline<'ast, S>,
{
    fn to_ast(&self, alloc: &'ast AstAlloc) -> T {
        T::from_mainline(alloc, self)
    }
}

// Primops don't need any heap allocation, so we can implement `From` directly.
impl From<&term::UnaryOp> for PrimOp {
    fn from(op: &term::UnaryOp) -> Self {
        match op {
            term::UnaryOp::IfThenElse => {
                panic!("if-then-else should have been handed separately by special casing")
            }
            term::UnaryOp::Typeof => PrimOp::Typeof,
            term::UnaryOp::BoolAnd => PrimOp::BoolAnd,
            term::UnaryOp::BoolOr => PrimOp::BoolOr,
            term::UnaryOp::BoolNot => PrimOp::BoolNot,
            term::UnaryOp::Blame => PrimOp::Blame,
            term::UnaryOp::EnumEmbed(loc_ident) => PrimOp::EnumEmbed(*loc_ident),
            term::UnaryOp::RecordAccess(loc_ident) => PrimOp::RecordStatAccess(*loc_ident),
            term::UnaryOp::ArrayMap => PrimOp::ArrayMap,
            term::UnaryOp::RecordMap => PrimOp::RecordMap,
            term::UnaryOp::LabelFlipPol => PrimOp::LabelFlipPol,
            term::UnaryOp::LabelPol => PrimOp::LabelPol,
            term::UnaryOp::LabelGoDom => PrimOp::LabelGoDom,
            term::UnaryOp::LabelGoCodom => PrimOp::LabelGoCodom,
            term::UnaryOp::LabelGoArray => PrimOp::LabelGoArray,
            term::UnaryOp::LabelGoDict => PrimOp::LabelGoDict,
            term::UnaryOp::Seq => PrimOp::Seq,
            term::UnaryOp::DeepSeq => PrimOp::DeepSeq,
            term::UnaryOp::ArrayLength => PrimOp::ArrayLength,
            term::UnaryOp::ArrayGen => PrimOp::ArrayGen,
            term::UnaryOp::RecordFields(record_op_kind) => PrimOp::RecordFields(*record_op_kind),
            term::UnaryOp::RecordValues => PrimOp::RecordValues,
            term::UnaryOp::StringTrim => PrimOp::StringTrim,
            term::UnaryOp::StringChars => PrimOp::StringChars,
            term::UnaryOp::StringUppercase => PrimOp::StringUppercase,
            term::UnaryOp::StringLowercase => PrimOp::StringLowercase,
            term::UnaryOp::StringLength => PrimOp::StringLength,
            term::UnaryOp::ToString => PrimOp::ToString,
            term::UnaryOp::NumberFromString => PrimOp::NumberFromString,
            term::UnaryOp::EnumFromString => PrimOp::EnumFromString,
            term::UnaryOp::StringIsMatch => PrimOp::StringIsMatch,
            term::UnaryOp::StringFind => PrimOp::StringFind,
            term::UnaryOp::StringFindAll => PrimOp::StringFindAll,
            term::UnaryOp::Force {
                ignore_not_exported,
            } => PrimOp::Force {
                ignore_not_exported: *ignore_not_exported,
            },
            term::UnaryOp::RecordEmptyWithTail => PrimOp::RecordEmptyWithTail,
            term::UnaryOp::Trace => PrimOp::Trace,
            term::UnaryOp::LabelPushDiag => PrimOp::LabelPushDiag,
            #[cfg(feature = "nix-experimental")]
            term::UnaryOp::EvalNix => PrimOp::EvalNix,
            term::UnaryOp::EnumGetArg => PrimOp::EnumGetArg,
            term::UnaryOp::EnumMakeVariant => PrimOp::EnumMakeVariant,
            term::UnaryOp::EnumIsVariant => PrimOp::EnumIsVariant,
            term::UnaryOp::EnumGetTag => PrimOp::EnumGetTag,
            term::UnaryOp::ContractCustom => PrimOp::ContractCustom,
            term::UnaryOp::NumberArcCos => PrimOp::NumberArcCos,
            term::UnaryOp::NumberArcSin => PrimOp::NumberArcSin,
            term::UnaryOp::NumberArcTan => PrimOp::NumberArcTan,
            term::UnaryOp::NumberCos => PrimOp::NumberCos,
            term::UnaryOp::NumberSin => PrimOp::NumberSin,
            term::UnaryOp::NumberTan => PrimOp::NumberTan,

            op @ (term::UnaryOp::TagsOnlyMatch { .. }
            | term::UnaryOp::ChunksConcat
            | term::UnaryOp::StringIsMatchCompiled(_)
            | term::UnaryOp::StringFindCompiled(_)
            | term::UnaryOp::StringFindAllCompiled(_)
            | term::UnaryOp::RecDefault
            | term::UnaryOp::RecForce
            | term::UnaryOp::PatternBranch
            | term::UnaryOp::ContractPostprocessResult) => {
                panic!("didn't expect {op} at the parsing stage")
            }
        }
    }
}

impl From<&term::BinaryOp> for PrimOp {
    fn from(op: &term::BinaryOp) -> Self {
        match op {
            term::BinaryOp::Plus => PrimOp::Plus,
            term::BinaryOp::Sub => PrimOp::Sub,
            term::BinaryOp::Mult => PrimOp::Mult,
            term::BinaryOp::Div => PrimOp::Div,
            term::BinaryOp::Modulo => PrimOp::Modulo,
            term::BinaryOp::NumberArcTan2 => PrimOp::NumberArcTan2,
            term::BinaryOp::NumberLog => PrimOp::NumberLog,
            term::BinaryOp::Pow => PrimOp::Pow,
            term::BinaryOp::StringConcat => PrimOp::StringConcat,
            term::BinaryOp::Eq => PrimOp::Eq,
            term::BinaryOp::LessThan => PrimOp::LessThan,
            term::BinaryOp::LessOrEq => PrimOp::LessOrEq,
            term::BinaryOp::GreaterThan => PrimOp::GreaterThan,
            term::BinaryOp::GreaterOrEq => PrimOp::GreaterOrEq,
            term::BinaryOp::ContractApply => PrimOp::ContractApply,
            term::BinaryOp::ContractCheck => PrimOp::ContractCheck,
            term::BinaryOp::LabelWithErrorData => PrimOp::LabelWithErrorData,
            term::BinaryOp::LabelGoField => PrimOp::LabelGoField,
            // This corresponds to a call to `%record/insert%` from the source language. Other
            // forms are introduced by the evaluator, e.g. when evaluating a recursive record to a
            // record.
            term::BinaryOp::RecordInsert {
                metadata,
                pending_contracts,
                ext_kind,
                op_kind,
            } if metadata.is_empty()
                && pending_contracts.is_empty()
                && *ext_kind == term::RecordExtKind::WithValue =>
            {
                PrimOp::RecordInsert(*op_kind)
            }
            term::BinaryOp::RecordRemove(record_op_kind) => PrimOp::RecordRemove(*record_op_kind),
            term::BinaryOp::RecordGet => PrimOp::RecordGet,
            term::BinaryOp::RecordHasField(record_op_kind) => {
                PrimOp::RecordHasField(*record_op_kind)
            }
            term::BinaryOp::RecordFieldIsDefined(record_op_kind) => {
                PrimOp::RecordFieldIsDefined(*record_op_kind)
            }
            term::BinaryOp::RecordSplitPair => PrimOp::RecordSplitPair,
            term::BinaryOp::RecordDisjointMerge => PrimOp::RecordDisjointMerge,
            term::BinaryOp::ArrayConcat => PrimOp::ArrayConcat,
            term::BinaryOp::ArrayAt => PrimOp::ArrayAt,
            term::BinaryOp::Merge(merge_label) => PrimOp::Merge(merge_label.kind),
            term::BinaryOp::Hash => PrimOp::Hash,
            term::BinaryOp::Serialize => PrimOp::Serialize,
            term::BinaryOp::Deserialize => PrimOp::Deserialize,
            term::BinaryOp::StringSplit => PrimOp::StringSplit,
            term::BinaryOp::StringContains => PrimOp::StringContains,
            term::BinaryOp::StringCompare => PrimOp::StringCompare,
            term::BinaryOp::ContractArrayLazyApp => PrimOp::ContractArrayLazyApp,
            term::BinaryOp::ContractRecordLazyApp => PrimOp::ContractRecordLazyApp,
            term::BinaryOp::LabelWithMessage => PrimOp::LabelWithMessage,
            term::BinaryOp::LabelWithNotes => PrimOp::LabelWithNotes,
            term::BinaryOp::LabelAppendNote => PrimOp::LabelAppendNote,
            term::BinaryOp::LabelLookupTypeVar => PrimOp::LabelLookupTypeVar,

            op @ (term::BinaryOp::RecordInsert { .. }
            | term::BinaryOp::Unseal
            | term::BinaryOp::Seal) => panic!("didn't expect {op} at the parsing stage"),
        }
    }
}

impl From<&term::NAryOp> for PrimOp {
    fn from(op: &term::NAryOp) -> Self {
        match op {
            term::NAryOp::StringReplace => PrimOp::StringReplace,
            term::NAryOp::StringReplaceRegex => PrimOp::StringReplaceRegex,
            term::NAryOp::StringSubstr => PrimOp::StringSubstr,
            term::NAryOp::MergeContract => PrimOp::MergeContract,
            term::NAryOp::RecordSealTail => PrimOp::RecordSealTail,
            term::NAryOp::RecordUnsealTail => PrimOp::RecordUnsealTail,
            term::NAryOp::LabelInsertTypeVar => PrimOp::LabelInsertTypeVar,
            term::NAryOp::ArraySlice => PrimOp::ArraySlice,
        }
    }
}

/// Trait from converting from the new AST representation to the mainline Nickel representation.
///
/// Note that in that direction, we don't need the allocator: those traits are thus isomorphic to
/// to `From<_>` and `Into<_>` respectively. However, we convert from a reference to an owned
/// value. We initially used `From` directly, but this causes annoying inference issue around auto
/// deref and blanket implementations of `From`/`Into`. It's just simpler and more explicit to have
/// a separate trait for this conversion as well.
pub trait FromAst<T> {
    fn from_ast(ast: &T) -> Self;
}

pub trait ToMainline<T> {
    fn to_mainline(&self) -> T;
}

impl<S, T> ToMainline<T> for S
where
    T: FromAst<S>,
{
    fn to_mainline(&self) -> T {
        T::from_ast(self)
    }
}

impl<'ast> FromAst<Pattern<'ast>> for term::pattern::Pattern {
    fn from_ast(pattern: &Pattern<'ast>) -> Self {
        term::pattern::Pattern {
            data: pattern.data.to_mainline(),
            alias: pattern.alias,
            pos: pattern.pos,
        }
    }
}

impl<'ast> FromAst<PatternData<'ast>> for term::pattern::PatternData {
    fn from_ast(ast: &PatternData<'ast>) -> Self {
        match ast {
            PatternData::Wildcard => term::pattern::PatternData::Wildcard,
            PatternData::Any(id) => term::pattern::PatternData::Any(*id),
            PatternData::Record(record_pattern) => (*record_pattern).to_mainline(),
            PatternData::Array(array_pattern) => (*array_pattern).to_mainline(),
            PatternData::Enum(enum_pattern) => (*enum_pattern).to_mainline(),
            PatternData::Constant(constant_pattern) => (*constant_pattern).to_mainline(),
            PatternData::Or(or_pattern) => (*or_pattern).to_mainline(),
        }
    }
}

impl<'ast> FromAst<RecordPattern<'ast>> for term::pattern::PatternData {
    fn from_ast(record_pat: &RecordPattern<'ast>) -> Self {
        let patterns = record_pat
            .patterns
            .iter()
            .map(|field_pattern| field_pattern.to_mainline())
            .collect();

        let tail = match record_pat.tail {
            TailPattern::Empty => term::pattern::TailPattern::Empty,
            TailPattern::Open => term::pattern::TailPattern::Open,
            TailPattern::Capture(id) => term::pattern::TailPattern::Capture(id),
        };

        term::pattern::PatternData::Record(term::pattern::RecordPattern {
            patterns,
            tail,
            pos: record_pat.pos,
        })
    }
}

impl<'ast> FromAst<FieldPattern<'ast>> for term::pattern::FieldPattern {
    fn from_ast(field_pat: &FieldPattern<'ast>) -> Self {
        let pattern = field_pat.pattern.to_mainline();

        let default = field_pat.default.as_ref().map(|term| term.to_mainline());

        let annotation = field_pat.annotation.to_mainline();

        term::pattern::FieldPattern {
            matched_id: field_pat.matched_id,
            annotation,
            default,
            pattern,
            pos: field_pat.pos,
        }
    }
}

impl<'ast> FromAst<ArrayPattern<'ast>> for term::pattern::PatternData {
    fn from_ast(array_pat: &ArrayPattern<'ast>) -> Self {
        let patterns = array_pat
            .patterns
            .iter()
            .map(|pat| pat.to_mainline())
            .collect();

        let tail = match array_pat.tail {
            TailPattern::Empty => term::pattern::TailPattern::Empty,
            TailPattern::Open => term::pattern::TailPattern::Open,
            TailPattern::Capture(id) => term::pattern::TailPattern::Capture(id),
        };

        term::pattern::PatternData::Array(term::pattern::ArrayPattern {
            patterns,
            tail,
            pos: array_pat.pos,
        })
    }
}

impl<'ast> FromAst<EnumPattern<'ast>> for term::pattern::PatternData {
    fn from_ast(enum_pat: &EnumPattern<'ast>) -> Self {
        let pattern = enum_pat
            .pattern
            .as_ref()
            .map(|pat| Box::new(pat.to_mainline()));

        term::pattern::PatternData::Enum(term::pattern::EnumPattern {
            tag: enum_pat.tag,
            pattern,
            pos: enum_pat.pos,
        })
    }
}

impl<'ast> FromAst<ConstantPattern<'ast>> for term::pattern::PatternData {
    fn from_ast(pattern: &ConstantPattern<'ast>) -> Self {
        let data = match pattern.data {
            ConstantPatternData::Bool(b) => term::pattern::ConstantPatternData::Bool(b),
            ConstantPatternData::Number(n) => term::pattern::ConstantPatternData::Number(n.clone()),
            ConstantPatternData::String(s) => term::pattern::ConstantPatternData::String(s.into()),
            ConstantPatternData::Null => term::pattern::ConstantPatternData::Null,
        };

        term::pattern::PatternData::Constant(term::pattern::ConstantPattern {
            data,
            pos: pattern.pos,
        })
    }
}

impl<'ast> FromAst<OrPattern<'ast>> for term::pattern::PatternData {
    fn from_ast(pattern: &OrPattern<'ast>) -> Self {
        let patterns = pattern
            .patterns
            .iter()
            .map(|pat| pat.to_mainline())
            .collect::<Vec<_>>();

        term::pattern::PatternData::Or(term::pattern::OrPattern {
            patterns,
            pos: pattern.pos,
        })
    }
}

impl<'ast> FromAst<Annotation<'ast>> for term::TypeAnnotation {
    fn from_ast(annot: &Annotation<'ast>) -> Self {
        let typ = annot.typ.as_ref().map(ToMainline::to_mainline);

        let contracts = annot
            .contracts
            .iter()
            .map(ToMainline::to_mainline)
            .collect();

        term::TypeAnnotation { typ, contracts }
    }
}

impl<'ast> FromAst<record::Field<'ast>> for term::record::Field {
    fn from_ast(field: &record::Field<'ast>) -> Self {
        term::record::Field {
            value: field.value.as_ref().map(|term| term.to_mainline()),
            metadata: field.metadata.to_mainline(),
            pending_contracts: Vec::new(),
        }
    }
}

impl<'ast> FromAst<record::FieldMetadata<'ast>> for term::record::FieldMetadata {
    fn from_ast(metadata: &record::FieldMetadata<'ast>) -> Self {
        let doc = metadata.doc.as_ref().map(|doc| String::from(&**doc));

        term::record::FieldMetadata {
            doc,
            annotation: metadata.annotation.to_mainline(),
            opt: metadata.opt,
            not_exported: metadata.not_exported,
            priority: metadata.priority.clone(),
        }
    }
}

impl<'ast> FromAst<Type<'ast>> for mline_type::Type {
    fn from_ast(typ: &Type<'ast>) -> Self {
        mline_type::Type {
            typ: typ.typ.to_mainline(),
            pos: typ.pos,
        }
    }
}

impl<'ast> FromAst<TypeUnr<'ast>> for MainlineTypeUnr {
    fn from_ast(typ: &TypeUnr<'ast>) -> Self {
        typ.clone().map(
            |typ| Box::new(typ.to_mainline()),
            |rrows| rrows.to_mainline(),
            |erows| erows.to_mainline(),
            |ctr| ctr.to_mainline(),
        )
    }
}

impl<'ast> FromAst<RecordRows<'ast>> for mline_type::RecordRows {
    fn from_ast(rrows: &RecordRows<'ast>) -> Self {
        mline_type::RecordRows(rrows.0.to_mainline())
    }
}

impl<'ast> FromAst<EnumRows<'ast>> for mline_type::EnumRows {
    fn from_ast(erows: &EnumRows<'ast>) -> Self {
        mline_type::EnumRows(erows.0.to_mainline())
    }
}

impl<'ast> FromAst<EnumRowsUnr<'ast>> for MainlineEnumRowsUnr {
    fn from_ast(erows: &EnumRowsUnr<'ast>) -> Self {
        erows.clone().map(
            |typ| Box::new(typ.to_mainline()),
            |erows| Box::new(erows.to_mainline()),
        )
    }
}

impl<'ast> FromAst<RecordRowsUnr<'ast>> for MainlineRecordRowsUnr {
    fn from_ast(rrows: &RecordRowsUnr<'ast>) -> Self {
        rrows.clone().map(
            |typ| Box::new(typ.to_mainline()),
            |rrows| Box::new(rrows.to_mainline()),
        )
    }
}

impl<'ast> FromAst<Type<'ast>> for term::LabeledType {
    fn from_ast(typ: &Type<'ast>) -> Self {
        let typ: mline_type::Type = typ.to_mainline();
        // We expect the new AST node to always have a position set. In fact we should
        // probably switch to `RawSpan` instead of `TermPos` everywhere; but let's do that
        // later
        let span = typ.pos.unwrap();

        term::LabeledType {
            typ: typ.clone(),
            label: label::Label {
                typ: std::rc::Rc::new(typ),
                span,
                ..Default::default()
            },
        }
    }
}

impl<'ast> FromAst<MatchBranch<'ast>> for term::MatchBranch {
    fn from_ast(branch: &MatchBranch<'ast>) -> Self {
        term::MatchBranch {
            pattern: branch.pattern.to_mainline(),
            guard: branch.guard.as_ref().map(|ast| ast.to_mainline()),
            body: branch.body.to_mainline(),
        }
    }
}

/// One data type representing all possible primops from the mainline AST, whether unary, binary or
/// multi-ary.
enum TermPrimOp {
    Unary(term::UnaryOp),
    Binary(term::BinaryOp),
    NAry(term::NAryOp),
}

impl FromAst<PrimOp> for TermPrimOp {
    fn from_ast(op: &PrimOp) -> Self {
        match op {
            PrimOp::Typeof => TermPrimOp::Unary(term::UnaryOp::Typeof),
            PrimOp::BoolAnd => TermPrimOp::Unary(term::UnaryOp::BoolAnd),
            PrimOp::BoolOr => TermPrimOp::Unary(term::UnaryOp::BoolOr),
            PrimOp::BoolNot => TermPrimOp::Unary(term::UnaryOp::BoolNot),
            PrimOp::Blame => TermPrimOp::Unary(term::UnaryOp::Blame),
            PrimOp::EnumEmbed(loc_ident) => TermPrimOp::Unary(term::UnaryOp::EnumEmbed(*loc_ident)),
            PrimOp::RecordStatAccess(loc_ident) => {
                TermPrimOp::Unary(term::UnaryOp::RecordAccess(*loc_ident))
            }
            PrimOp::ArrayMap => TermPrimOp::Unary(term::UnaryOp::ArrayMap),
            PrimOp::RecordMap => TermPrimOp::Unary(term::UnaryOp::RecordMap),
            PrimOp::LabelFlipPol => TermPrimOp::Unary(term::UnaryOp::LabelFlipPol),
            PrimOp::LabelPol => TermPrimOp::Unary(term::UnaryOp::LabelPol),
            PrimOp::LabelGoDom => TermPrimOp::Unary(term::UnaryOp::LabelGoDom),
            PrimOp::LabelGoCodom => TermPrimOp::Unary(term::UnaryOp::LabelGoCodom),
            PrimOp::LabelGoArray => TermPrimOp::Unary(term::UnaryOp::LabelGoArray),
            PrimOp::LabelGoDict => TermPrimOp::Unary(term::UnaryOp::LabelGoDict),
            PrimOp::Seq => TermPrimOp::Unary(term::UnaryOp::Seq),
            PrimOp::DeepSeq => TermPrimOp::Unary(term::UnaryOp::DeepSeq),
            PrimOp::ArrayLength => TermPrimOp::Unary(term::UnaryOp::ArrayLength),
            PrimOp::ArrayGen => TermPrimOp::Unary(term::UnaryOp::ArrayGen),
            PrimOp::RecordFields(record_op_kind) => {
                TermPrimOp::Unary(term::UnaryOp::RecordFields(*record_op_kind))
            }
            PrimOp::RecordValues => TermPrimOp::Unary(term::UnaryOp::RecordValues),
            PrimOp::StringTrim => TermPrimOp::Unary(term::UnaryOp::StringTrim),
            PrimOp::StringChars => TermPrimOp::Unary(term::UnaryOp::StringChars),
            PrimOp::StringUppercase => TermPrimOp::Unary(term::UnaryOp::StringUppercase),
            PrimOp::StringLowercase => TermPrimOp::Unary(term::UnaryOp::StringLowercase),
            PrimOp::StringLength => TermPrimOp::Unary(term::UnaryOp::StringLength),
            PrimOp::ToString => TermPrimOp::Unary(term::UnaryOp::ToString),
            PrimOp::NumberFromString => TermPrimOp::Unary(term::UnaryOp::NumberFromString),
            PrimOp::EnumFromString => TermPrimOp::Unary(term::UnaryOp::EnumFromString),
            PrimOp::StringIsMatch => TermPrimOp::Unary(term::UnaryOp::StringIsMatch),
            PrimOp::StringFind => TermPrimOp::Unary(term::UnaryOp::StringFind),
            PrimOp::StringFindAll => TermPrimOp::Unary(term::UnaryOp::StringFindAll),
            PrimOp::Force {
                ignore_not_exported,
            } => TermPrimOp::Unary(term::UnaryOp::Force {
                ignore_not_exported: *ignore_not_exported,
            }),
            PrimOp::RecordEmptyWithTail => TermPrimOp::Unary(term::UnaryOp::RecordEmptyWithTail),
            PrimOp::Trace => TermPrimOp::Unary(term::UnaryOp::Trace),
            PrimOp::LabelPushDiag => TermPrimOp::Unary(term::UnaryOp::LabelPushDiag),
            PrimOp::EnumGetArg => TermPrimOp::Unary(term::UnaryOp::EnumGetArg),
            PrimOp::EnumMakeVariant => TermPrimOp::Unary(term::UnaryOp::EnumMakeVariant),
            PrimOp::EnumIsVariant => TermPrimOp::Unary(term::UnaryOp::EnumIsVariant),
            PrimOp::EnumGetTag => TermPrimOp::Unary(term::UnaryOp::EnumGetTag),
            PrimOp::ContractCustom => TermPrimOp::Unary(term::UnaryOp::ContractCustom),
            PrimOp::NumberArcCos => TermPrimOp::Unary(term::UnaryOp::NumberArcCos),
            PrimOp::NumberArcSin => TermPrimOp::Unary(term::UnaryOp::NumberArcSin),
            PrimOp::NumberArcTan => TermPrimOp::Unary(term::UnaryOp::NumberArcTan),
            PrimOp::NumberCos => TermPrimOp::Unary(term::UnaryOp::NumberCos),
            PrimOp::NumberSin => TermPrimOp::Unary(term::UnaryOp::NumberSin),
            PrimOp::NumberTan => TermPrimOp::Unary(term::UnaryOp::NumberTan),
            #[cfg(feature = "nix-experimental")]
            PrimOp::EvalNix => TermPrimOp::Unary(term::UnaryOp::EvalNix),

            // Binary operations
            PrimOp::Plus => TermPrimOp::Binary(term::BinaryOp::Plus),
            PrimOp::Sub => TermPrimOp::Binary(term::BinaryOp::Sub),
            PrimOp::Mult => TermPrimOp::Binary(term::BinaryOp::Mult),
            PrimOp::Div => TermPrimOp::Binary(term::BinaryOp::Div),
            PrimOp::Modulo => TermPrimOp::Binary(term::BinaryOp::Modulo),
            PrimOp::NumberArcTan2 => TermPrimOp::Binary(term::BinaryOp::NumberArcTan2),
            PrimOp::NumberLog => TermPrimOp::Binary(term::BinaryOp::NumberLog),
            PrimOp::Pow => TermPrimOp::Binary(term::BinaryOp::Pow),
            PrimOp::StringConcat => TermPrimOp::Binary(term::BinaryOp::StringConcat),
            PrimOp::Eq => TermPrimOp::Binary(term::BinaryOp::Eq),
            PrimOp::LessThan => TermPrimOp::Binary(term::BinaryOp::LessThan),
            PrimOp::LessOrEq => TermPrimOp::Binary(term::BinaryOp::LessOrEq),
            PrimOp::GreaterThan => TermPrimOp::Binary(term::BinaryOp::GreaterThan),
            PrimOp::GreaterOrEq => TermPrimOp::Binary(term::BinaryOp::GreaterOrEq),
            PrimOp::ContractApply => TermPrimOp::Binary(term::BinaryOp::ContractApply),
            PrimOp::ContractCheck => TermPrimOp::Binary(term::BinaryOp::ContractCheck),
            PrimOp::LabelWithErrorData => TermPrimOp::Binary(term::BinaryOp::LabelWithErrorData),
            PrimOp::LabelGoField => TermPrimOp::Binary(term::BinaryOp::LabelGoField),
            PrimOp::RecordInsert(record_op_kind) => {
                TermPrimOp::Binary(term::BinaryOp::RecordInsert {
                    metadata: Default::default(),
                    pending_contracts: Vec::new(),
                    ext_kind: term::RecordExtKind::WithValue,
                    op_kind: *record_op_kind,
                })
            }
            PrimOp::RecordRemove(record_op_kind) => {
                TermPrimOp::Binary(term::BinaryOp::RecordRemove(*record_op_kind))
            }
            PrimOp::RecordGet => TermPrimOp::Binary(term::BinaryOp::RecordGet),
            PrimOp::RecordHasField(record_op_kind) => {
                TermPrimOp::Binary(term::BinaryOp::RecordHasField(*record_op_kind))
            }
            PrimOp::RecordFieldIsDefined(record_op_kind) => {
                TermPrimOp::Binary(term::BinaryOp::RecordFieldIsDefined(*record_op_kind))
            }
            PrimOp::RecordSplitPair => TermPrimOp::Binary(term::BinaryOp::RecordSplitPair),
            PrimOp::RecordDisjointMerge => TermPrimOp::Binary(term::BinaryOp::RecordDisjointMerge),
            PrimOp::ArrayConcat => TermPrimOp::Binary(term::BinaryOp::ArrayConcat),
            PrimOp::ArrayAt => TermPrimOp::Binary(term::BinaryOp::ArrayAt),
            PrimOp::Merge(merge_kind) => {
                // [^merge-label-span] The mainline AST requires a `MergeLabel` object, itself
                // demanding a `RawSpan` that we can't provide here - it's stored higher up in the
                // AST, at the `PrimOpApp` node. We generate a dummy span and rely on the caller
                // (in practice `FromAst<Node<'_>>`) to post-process a merge primop application,
                // setting the span of the dummy merge label correctly.
                let dummy_label: label::MergeLabel = label::Label::dummy().into();

                TermPrimOp::Binary(term::BinaryOp::Merge(label::MergeLabel {
                    kind: *merge_kind,
                    ..dummy_label
                }))
            }
            PrimOp::Hash => TermPrimOp::Binary(term::BinaryOp::Hash),
            PrimOp::Serialize => TermPrimOp::Binary(term::BinaryOp::Serialize),
            PrimOp::Deserialize => TermPrimOp::Binary(term::BinaryOp::Deserialize),
            PrimOp::StringSplit => TermPrimOp::Binary(term::BinaryOp::StringSplit),
            PrimOp::StringContains => TermPrimOp::Binary(term::BinaryOp::StringContains),
            PrimOp::StringCompare => TermPrimOp::Binary(term::BinaryOp::StringCompare),
            PrimOp::ContractArrayLazyApp => {
                TermPrimOp::Binary(term::BinaryOp::ContractArrayLazyApp)
            }
            PrimOp::ContractRecordLazyApp => {
                TermPrimOp::Binary(term::BinaryOp::ContractRecordLazyApp)
            }
            PrimOp::LabelWithMessage => TermPrimOp::Binary(term::BinaryOp::LabelWithMessage),
            PrimOp::LabelWithNotes => TermPrimOp::Binary(term::BinaryOp::LabelWithNotes),
            PrimOp::LabelAppendNote => TermPrimOp::Binary(term::BinaryOp::LabelAppendNote),
            PrimOp::LabelLookupTypeVar => TermPrimOp::Binary(term::BinaryOp::LabelLookupTypeVar),

            // N-ary operations
            PrimOp::StringReplace => TermPrimOp::NAry(term::NAryOp::StringReplace),
            PrimOp::StringReplaceRegex => TermPrimOp::NAry(term::NAryOp::StringReplaceRegex),
            PrimOp::StringSubstr => TermPrimOp::NAry(term::NAryOp::StringSubstr),
            PrimOp::MergeContract => TermPrimOp::NAry(term::NAryOp::MergeContract),
            PrimOp::RecordSealTail => TermPrimOp::NAry(term::NAryOp::RecordSealTail),
            PrimOp::RecordUnsealTail => TermPrimOp::NAry(term::NAryOp::RecordUnsealTail),
            PrimOp::LabelInsertTypeVar => TermPrimOp::NAry(term::NAryOp::LabelInsertTypeVar),
            PrimOp::ArraySlice => TermPrimOp::NAry(term::NAryOp::ArraySlice),
        }
    }
}

impl<'ast> FromAst<Node<'ast>> for term::Term {
    fn from_ast(node: &Node<'ast>) -> Self {
        use term::Term;

        match node {
            Node::Null => Term::Null,
            Node::Bool(b) => Term::Bool(*b),
            Node::Number(n) => Term::Num((**n).clone()),
            Node::String(s) => Term::Str((*s).into()),
            Node::StrChunks(chunks) => {
                let chunks = chunks
                    .iter()
                    .map(|chunk| match chunk {
                        StrChunk::Literal(s) => term::StrChunk::Literal(s.clone()),
                        StrChunk::Expr(expr, indent) => {
                            term::StrChunk::Expr(expr.to_mainline(), *indent)
                        }
                    })
                    .collect();

                Term::StrChunks(chunks)
            }
            Node::Fun { arg, body } => match arg.data {
                PatternData::Any(id) => Term::Fun(id, body.to_mainline()),
                _ => Term::FunPattern((*arg).to_mainline(), body.to_mainline()),
            },
            Node::Let {
                bindings,
                body,
                rec,
            } => {
                // We try to collect all patterns as single identifiers. If this works, we can emit
                // a simpler / more compact `Let`.
                let try_bindings = bindings
                    .iter()
                    .map(|(pat, term)| match pat.data {
                        PatternData::Any(id) => Some((id, term.to_mainline())),
                        _ => None,
                    })
                    .collect::<Option<SmallVec<_>>>();

                let body = body.to_mainline();
                let attrs = term::LetAttrs {
                    rec: *rec,
                    ..Default::default()
                };

                if let Some(bindings) = try_bindings {
                    Term::Let(bindings, body, attrs)
                } else {
                    let bindings = bindings
                        .iter()
                        .map(|(pat, term)| (pat.to_mainline(), term.to_mainline()))
                        .collect();

                    Term::LetPattern(bindings, body, attrs)
                }
            }
            Node::App { fun, args } => {
                // unwrap(): the position of Ast should always be set (we might move to `RawSpan`
                // instead of `TermPos` soon)
                let fun_span = fun.pos.unwrap();

                let rterm = args.iter().fold(fun.to_mainline(), |result, arg| {
                    // This case is a bit annoying: we need to extract the position of the sub
                    // application to satisfy the old AST structure, but this information isn't
                    // available directly.
                    // What we do here is to fuse the span of the term being built and the one of
                    // the current argument, which should be a reasonable approximation (if not
                    // exactly the same thing).
                    // unwrap(): the position of Ast should always be set (we might move to `RawSpan`
                    // instead of `TermPos` soon)
                    let span_arg = arg.pos.unwrap();
                    let span = fun_span.fuse(span_arg);

                    term::RichTerm::new(Term::App(result, arg.to_mainline()), span.into())
                });

                rterm.term.into_owned()
            }
            Node::Var(loc_ident) => Term::Var(*loc_ident),
            Node::EnumVariant { tag, arg } => {
                if let Some(arg) = arg {
                    Term::EnumVariant {
                        tag: *tag,
                        arg: arg.to_mainline(),
                        attrs: term::EnumVariantAttrs::default(),
                    }
                } else {
                    Term::Enum(*tag)
                }
            }
            Node::Record(_) => todo!(),
            Node::IfThenElse {
                cond,
                then_branch,
                else_branch,
            } => term::make::if_then_else(
                term::RichTerm::from_ast(cond),
                term::RichTerm::from_ast(then_branch),
                term::RichTerm::from_ast(else_branch),
            )
            .term
            .into_owned(),
            Node::Match(data) => {
                let branches = data.branches.iter().map(ToMainline::to_mainline).collect();

                Term::Match(term::MatchData { branches })
            }
            Node::Array(array) => {
                let array = array.iter().map(ToMainline::to_mainline).collect();
                Term::Array(array, term::array::ArrayAttrs::default())
            }
            Node::PrimOpApp { op, args } => match (*op).to_mainline() {
                TermPrimOp::Unary(op) => Term::Op1(op, args[0].to_mainline()),
                // If `op` is `Merge`, we need to patch the span of the merge label with the
                // correct value. Unfortunately, we still don't have access to the right span,
                // which is the position of this whole node. We delegate this to the caller, that
                // is `from_ast::<Ast<'ast>>`. See [^merge-label-span].
                TermPrimOp::Binary(op) => {
                    Term::Op2(op, args[0].to_mainline(), args[1].to_mainline())
                }
                TermPrimOp::NAry(op) => {
                    Term::OpN(op, args.iter().map(|arg| (*arg).to_mainline()).collect())
                }
            },
            Node::Annotated { annot, inner } => {
                Term::Annotated((*annot).to_mainline(), inner.to_mainline())
            }
            Node::Import { path, format } => Term::Import {
                path: (*path).clone(),
                format: *format,
            },
            Node::Type(typ) => {
                let typ: mline_type::Type = (*typ).to_mainline();

                let contract = typ
                    .contract()
                    // It would be painful to change the interface of `ToMainline` and make it
                    // fallible just for this one special case. Instead, if the contract
                    // conversion causes an unbound variable error (which it shouldn't anyway if
                    // the term has been correctly typechecked), we pack this error as parse
                    // error in the AST.
                    .unwrap_or_else(|err| {
                        Term::ParseError(ParseError::UnboundTypeVariables(vec![err.0])).into()
                    });

                Term::Type { typ, contract }
            }
            Node::ParseError(error) => Term::ParseError((*error).clone()),
        }
    }
}

impl<'ast> FromAst<Ast<'ast>> for term::RichTerm {
    fn from_ast(ast: &Ast<'ast>) -> Self {
        let mut result = term::RichTerm::new(ast.node.to_mainline(), ast.pos);
        // See [^merge-label-span]
        if let term::Term::Op2(term::BinaryOp::Merge(ref mut label), _, _) =
            term::SharedTerm::make_mut(&mut result.term)
        {
            // unwrap(): we expect all position to be set in the new AST (should be using span
            // directly in the future)
            label.span = ast.pos.unwrap();
        }

        result
    }
}

impl<'ast> FromAst<&'ast Ast<'ast>> for term::RichTerm {
    fn from_ast(ast: &&'ast Ast<'ast>) -> Self {
        FromAst::from_ast(*ast)
    }
}
