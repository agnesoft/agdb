use super::Rewrite;
use super::RewriteContext;
use agdb::type_def::Type;

pub struct StripPointerTypes;

impl Rewrite for StripPointerTypes {
    fn name(&self) -> &str {
        "strip_pointer_types"
    }

    fn rewrite_type(&self, ty: Type, _ctx: &RewriteContext) -> Type {
        match ty {
            Type::Pointer(p) => (p.ty)(),
            Type::Reference(r) => (r.ty)(),
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::RewritePipeline;
    use super::*;
    use agdb::type_def::Literal;
    use agdb::type_def::Pointer;
    use agdb::type_def::PointerKind;
    use agdb::type_def::Reference;

    #[test]
    fn arc_pointer_unwrapped() {
        let mut ty = Type::Pointer(Pointer {
            kind: PointerKind::Arc,
            ty: || Type::Literal(Literal::I32),
        });

        let pipeline = RewritePipeline::new(vec![Box::new(StripPointerTypes)]);
        pipeline.rewrite_type(&mut ty, &RewriteContext::default());

        assert!(matches!(ty, Type::Literal(Literal::I32)));
    }

    #[test]
    fn reference_unwrapped() {
        let mut ty = Type::Reference(Reference {
            mutable: false,
            lifetime: None,
            ty: || Type::Literal(Literal::String),
        });

        let pipeline = RewritePipeline::new(vec![Box::new(StripPointerTypes)]);
        pipeline.rewrite_type(&mut ty, &RewriteContext::default());

        assert!(matches!(ty, Type::Literal(Literal::String)));
    }

    #[test]
    fn non_pointer_unchanged() {
        let mut ty = Type::Literal(Literal::Bool);

        let pipeline = RewritePipeline::new(vec![Box::new(StripPointerTypes)]);
        pipeline.rewrite_type(&mut ty, &RewriteContext::default());

        assert!(matches!(ty, Type::Literal(Literal::Bool)));
    }
}
