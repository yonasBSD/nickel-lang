use lsp_server::{RequestId, Response, ResponseError};
use lsp_types::{Hover, HoverContents, HoverParams, LanguageString, MarkedString, Range};
use nickel_lang_core::{
    combine::Combine,
    identifier::Ident,
    position::RawSpan,
    term::{record::FieldMetadata, LabeledType, RichTerm, Term, UnaryOp},
    typ::Type,
};
use serde_json::Value;

use crate::{
    cache::CacheExt,
    diagnostic::LocationCompat,
    field_walker::{Def, FieldResolver, Record},
    identifier::LocIdent,
    server::Server,
    utils::dedup,
    world::World,
};

#[derive(Debug, Default)]
struct HoverData {
    values: Vec<RichTerm>,
    metadata: Vec<FieldMetadata>,
    span: Option<RawSpan>,
    ty: Option<Type>,
}

impl Combine for HoverData {
    fn combine(mut left: Self, mut right: Self) -> Self {
        left.values.append(&mut right.values);
        left.metadata.append(&mut right.metadata);
        left.ty = left.ty.or(right.ty);
        left.span = left.span.or(right.span);
        left
    }
}

fn annotated_contracts(rt: &RichTerm) -> &[LabeledType] {
    match rt.as_ref() {
        Term::Annotated(annot, _) => &annot.contracts,
        _ => &[],
    }
}

fn nickel_string(s: String) -> MarkedString {
    MarkedString::LanguageString(LanguageString {
        language: "nickel".to_owned(),
        value: s,
    })
}

fn values_and_metadata_from_field(
    parents: Vec<Record>,
    ident: Ident,
) -> (Vec<RichTerm>, Vec<FieldMetadata>) {
    let mut values = Vec::new();
    let mut metadata = Vec::new();
    for parent in parents {
        if let Some(field) = parent.field(ident) {
            values.extend(field.value.iter().cloned());
            metadata.push(field.metadata.clone());
        }
    }
    (values, metadata)
}

fn ident_hover(ident: LocIdent, world: &World) -> Option<HoverData> {
    let ty = world.analysis.get_type_for_ident(&ident).cloned();
    let span = ident.pos.into_opt()?;
    let mut ret = HoverData {
        values: Vec::new(),
        metadata: Vec::new(),
        span: Some(span),
        ty,
    };

    if let Some(def) = world.analysis.get_def(&ident) {
        let resolver = FieldResolver::new(world);
        if let Some(((last, path), val)) = def.path().split_last().zip(def.value()) {
            let parents = resolver.resolve_path(val, path.iter().copied());
            let (values, metadata) = values_and_metadata_from_field(parents, *last);
            ret.values = values;
            ret.metadata = metadata;
        } else if def.path().is_empty() {
            let cousins = resolver.cousin_defs(def);
            if cousins.is_empty() {
                ret.values.extend(def.value().into_iter().cloned());
            } else {
                for (_, cousin) in cousins {
                    if let Some(val) = cousin.value {
                        ret.values.push(val);
                    }
                    ret.metadata.push(cousin.metadata);
                }
            }

            if let Def::Field { metadata, .. } = def {
                ret.metadata.push(metadata.clone());
            }
        }
    }

    Some(ret)
}

fn term_hover(rt: &RichTerm, world: &World) -> Option<HoverData> {
    let ty = world.analysis.get_type(rt).cloned();
    let span = rt.pos.into_opt();

    match rt.as_ref() {
        Term::Op1(UnaryOp::RecordAccess(id), parent) => {
            let resolver = FieldResolver::new(world);
            let parents = resolver.resolve_record(parent);
            let (values, metadata) = values_and_metadata_from_field(parents, id.ident());
            Some(HoverData {
                values,
                metadata,
                span,
                ty,
            })
        }
        _ => Some(HoverData {
            values: vec![rt.clone()],
            metadata: vec![],
            span,
            ty,
        }),
    }
}

pub fn handle(
    params: HoverParams,
    req_id: RequestId,
    server: &mut Server,
) -> Result<(), ResponseError> {
    let pos = server
        .world
        .cache
        .position(&params.text_document_position_params)?;

    let ident_hover_data = server
        .world
        .lookup_ident_by_position(pos)?
        .and_then(|ident| ident_hover(ident, &server.world));

    let term = server.world.lookup_term_by_position(pos)?;
    let term_hover_data = term.and_then(|rt| term_hover(rt, &server.world));

    // We combine the hover information from the term (which can have better type information)
    // and the ident (which can have better metadata), but only when hovering over a `Var`.
    // In general, the term and the ident can have different meanings (like when hovering over
    // the `x` in `let x = ... in y`) and so it would be confusing to combine them.
    let hover_data = if matches!(term.map(AsRef::as_ref), Some(Term::Var(_))) {
        Combine::combine(ident_hover_data, term_hover_data)
    } else {
        ident_hover_data.or(term_hover_data)
    };

    if let Some(hover) = hover_data {
        let mut contents = Vec::new();

        // Collect all the type and contract annotations we can find. We don't distinguish between them
        // (and we deduplicate annotations if they're present as both types and contracts). However, we
        // do give some special attention to the inferred static type if there is one: we list it first.
        let mut annotations: Vec<_> = hover
            .metadata
            .iter()
            .flat_map(|m| m.annotation.iter().map(|typ| typ.typ.to_string()))
            .chain(
                hover
                    .values
                    .iter()
                    .flat_map(annotated_contracts)
                    .map(|contract| contract.label.typ.to_string()),
            )
            .collect();
        dedup(&mut annotations);

        let ty = hover
            .ty
            .as_ref()
            .map(Type::to_string)
            .unwrap_or_else(|| "Dyn".to_owned());

        // There's no point in repeating the static type in the annotations.
        if let Some(idx) = annotations.iter().position(|a| a == &ty) {
            annotations.remove(idx);
        }

        // Only report a Dyn type if there's no more useful information.
        if ty != "Dyn" || annotations.is_empty() {
            contents.push(nickel_string(ty));
        }

        contents.extend(annotations.into_iter().map(nickel_string));

        // Not sure how to do documentation merging yet, so pick the first non-empty one.
        let doc = hover.metadata.iter().find_map(|m| m.doc.as_ref());
        if let Some(doc) = doc {
            contents.push(MarkedString::String(doc.to_owned()));
        }

        server.reply(Response::new_ok(
            req_id,
            Hover {
                contents: HoverContents::Array(contents),
                range: hover
                    .span
                    .and_then(|s| Range::from_span(&s, server.world.cache.files())),
            },
        ));
    } else {
        server.reply(Response::new_ok(req_id, Value::Null));
    }
    Ok(())
}
