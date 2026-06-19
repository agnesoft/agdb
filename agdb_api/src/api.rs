use agdb::Comparison;
use agdb::CountComparison;
use agdb::DbElement;
use agdb::DbError;
use agdb::DbErrorCategory;
use agdb::DbErrorType;
use agdb::DbF64;
use agdb::DbId;
use agdb::DbKeyOrder;
use agdb::DbKeyOrders;
use agdb::DbKeyValue;
use agdb::DbTypeDef;
use agdb::DbValue;
use agdb::DbValues;
use agdb::Insert;
use agdb::InsertAliases;
use agdb::InsertAliasesIds;
use agdb::InsertAliasesQuery;
use agdb::InsertEdges;
use agdb::InsertEdgesEach;
use agdb::InsertEdgesFrom;
use agdb::InsertEdgesFromTo;
use agdb::InsertEdgesIds;
use agdb::InsertEdgesQuery;
use agdb::InsertEdgesValues;
use agdb::InsertIndex;
use agdb::InsertIndexQuery;
use agdb::InsertNodes;
use agdb::InsertNodesAliases;
use agdb::InsertNodesCount;
use agdb::InsertNodesIds;
use agdb::InsertNodesQuery;
use agdb::InsertNodesValues;
use agdb::InsertValues;
use agdb::InsertValuesIds;
use agdb::InsertValuesQuery;
use agdb::KeyValueComparison;
use agdb::MultiValues;
use agdb::QueryAliases;
use agdb::QueryBuilder;
use agdb::QueryCondition;
use agdb::QueryConditionData;
use agdb::QueryConditionLogic;
use agdb::QueryConditionModifier;
use agdb::QueryId;
use agdb::QueryIds;
use agdb::QueryResult;
use agdb::QueryType;
use agdb::QueryValues;
use agdb::Remove;
use agdb::RemoveAliases;
use agdb::RemoveAliasesQuery;
use agdb::RemoveIds;
use agdb::RemoveIndex;
use agdb::RemoveIndexQuery;
use agdb::RemoveQuery;
use agdb::RemoveValues;
use agdb::RemoveValuesIds;
use agdb::RemoveValuesQuery;
use agdb::Search;
use agdb::SearchAlgorithm;
use agdb::SearchFrom;
use agdb::SearchIndexBuilder;
use agdb::SearchIndexValue;
use agdb::SearchOrderBy;
use agdb::SearchQuery;
use agdb::SearchQueryAlgorithm;
use agdb::SearchQueryBuilderDef;
use agdb::SearchTo;
use agdb::Select;
use agdb::SelectAliases;
use agdb::SelectAliasesIds;
use agdb::SelectAliasesQuery;
use agdb::SelectAllAliasesQuery;
use agdb::SelectEdgeCount;
use agdb::SelectEdgeCountIds;
use agdb::SelectEdgeCountQuery;
use agdb::SelectIds;
use agdb::SelectIndexes;
use agdb::SelectIndexesQuery;
use agdb::SelectKeyCount;
use agdb::SelectKeyCountIds;
use agdb::SelectKeyCountQuery;
use agdb::SelectKeys;
use agdb::SelectKeysIds;
use agdb::SelectKeysQuery;
use agdb::SelectLimit;
use agdb::SelectNodeCount;
use agdb::SelectNodeCountQuery;
use agdb::SelectOffset;
use agdb::SelectValues;
use agdb::SelectValuesIds;
use agdb::SelectValuesQuery;
use agdb::SingleValues;
use agdb::Where;
use agdb::WhereKey;
use agdb::WhereLogicOperator;
use agdb::type_def::Type;
use agdb::type_def::TypeDefinition;
use std::panic::Location;
use std::sync::atomic::AtomicU16;
use std::time::Duration;

use crate::AdminStatus;
use crate::AgdbApi;
use crate::AgdbApiClientDef;
use crate::AgdbApiError;
use crate::ChangePassword;
use crate::ClusterStatus;
use crate::DbAudit;
use crate::DbKind;
use crate::DbResource;
use crate::DbUser;
use crate::DbUserRole;
use crate::HttpClientDef;
use crate::LogLevelFilter;
use crate::QueryAudit;
use crate::ReqwestClient;
use crate::ServerDatabase;
use crate::UserCredentials;
use crate::UserLogin;
use crate::UserSession;
use crate::UserStatus;
use crate::config_impl::ConfigImpl;
use crate::http_client::ReqwestClientTypeDef;

pub struct Api;

impl Api {
    pub fn types() -> Vec<Type> {
        let mut defs = Self::db_types();
        defs.extend(Self::query_types());
        defs.extend(Self::query_builder_types());
        defs.extend(Self::api_types());
        defs.extend(Self::misc_types());
        defs
    }

    #[cfg(feature = "test_server")]
    pub fn tests() -> Vec<(String, Vec<Type>)> {
        let mut defs = vec![(String::new(), Self::test_infra())];
        defs.extend(Self::test_defs());
        defs
    }

    fn db_types() -> Vec<Type> {
        vec![
            DbElement::type_def(),
            DbF64::type_def(),
            DbId::type_def(),
            DbKeyOrder::type_def(),
            DbKeyOrders::type_def(),
            DbKeyValue::type_def(),
            DbValue::type_def(),
            DbValues::type_def(),
            DbErrorCategory::type_def(),
            DbErrorType::type_def(),
            DbError::type_def(),
            DbTypeDef::type_def(), // trait
        ]
    }

    fn query_types() -> Vec<Type> {
        vec![
            QueryType::type_def(),
            QueryAliases::type_def(),
            InsertAliasesQuery::type_def(),
            InsertEdgesQuery::type_def(),
            InsertIndexQuery::type_def(),
            InsertNodesQuery::type_def(),
            InsertValuesQuery::type_def(),
            Comparison::type_def(),
            CountComparison::type_def(),
            KeyValueComparison::type_def(),
            QueryCondition::type_def(),
            QueryConditionData::type_def(),
            QueryConditionLogic::type_def(),
            QueryConditionModifier::type_def(),
            QueryId::type_def(),
            QueryIds::type_def(),
            QueryResult::type_def(),
            QueryValues::type_def(),
            SingleValues::type_def(),
            MultiValues::type_def(),
            RemoveAliasesQuery::type_def(),
            RemoveIndexQuery::type_def(),
            RemoveQuery::type_def(),
            RemoveValuesQuery::type_def(),
            SearchQuery::type_def(),
            SearchQueryAlgorithm::type_def(),
            SelectAliasesQuery::type_def(),
            SelectAllAliasesQuery::type_def(),
            SelectEdgeCountQuery::type_def(),
            SelectIndexesQuery::type_def(),
            SelectKeyCountQuery::type_def(),
            SelectKeysQuery::type_def(),
            SelectNodeCountQuery::type_def(),
            SelectValuesQuery::type_def(),
        ]
    }

    fn query_builder_types() -> Vec<Type> {
        vec![
            QueryBuilder::type_def(),
            Insert::type_def(),
            InsertAliases::type_def(),
            InsertAliasesIds::type_def(),
            InsertEdges::type_def(),
            InsertEdgesEach::type_def(),
            InsertEdgesFrom::type_def(),
            InsertEdgesFromTo::type_def(),
            InsertEdgesIds::type_def(),
            InsertEdgesValues::type_def(),
            InsertIndex::type_def(),
            InsertNodes::type_def(),
            InsertNodesAliases::type_def(),
            InsertNodesCount::type_def(),
            InsertNodesIds::type_def(),
            InsertNodesValues::type_def(),
            InsertValues::type_def(),
            InsertValuesIds::type_def(),
            Remove::type_def(),
            RemoveAliases::type_def(),
            RemoveIds::type_def(),
            RemoveIndex::type_def(),
            RemoveValues::type_def(),
            RemoveValuesIds::type_def(),
            Search::<SearchQuery>::type_def(),
            SearchAlgorithm::<SearchQuery>::type_def(),
            SearchFrom::<SearchQuery>::type_def(),
            SearchTo::<SearchQuery>::type_def(),
            SearchIndexBuilder::<SearchQuery>::type_def(),
            SearchIndexValue::<SearchQuery>::type_def(),
            SearchOrderBy::<SearchQuery>::type_def(),
            SelectLimit::<SearchQuery>::type_def(),
            SelectOffset::<SearchQuery>::type_def(),
            Select::type_def(),
            SelectAliases::type_def(),
            SelectAliasesIds::type_def(),
            SelectEdgeCount::type_def(),
            SelectEdgeCountIds::type_def(),
            SelectIds::type_def(),
            SelectIndexes::type_def(),
            SelectKeyCount::type_def(),
            SelectKeyCountIds::type_def(),
            SelectKeys::type_def(),
            SelectKeysIds::type_def(),
            SelectNodeCount::type_def(),
            SelectValues::type_def(),
            SelectValuesIds::type_def(),
            Where::<SearchQuery>::type_def(),
            WhereKey::<SearchQuery>::type_def(),
            WhereLogicOperator::<SearchQuery>::type_def(),
            SearchQueryBuilderDef::type_def(), //trait
        ]
    }

    fn api_types() -> Vec<Type> {
        vec![
            HttpClientDef::type_def(),    // trait
            AgdbApiClientDef::type_def(), // trait
            AgdbApiError::type_def(),
            ReqwestClient::type_def(),
            ReqwestClientTypeDef::type_def(),
            AgdbApi::<ReqwestClient>::type_def(),
            AdminStatus::type_def(),
            ChangePassword::type_def(),
            ClusterStatus::type_def(),
            DbAudit::type_def(),
            DbKind::type_def(),
            DbResource::type_def(),
            DbUser::type_def(),
            DbUserRole::type_def(),
            LogLevelFilter::type_def(),
            QueryAudit::type_def(),
            ServerDatabase::type_def(),
            UserCredentials::type_def(),
            UserLogin::type_def(),
            UserStatus::type_def(),
            UserSession::type_def(),
        ]
    }

    fn misc_types() -> Vec<Type> {
        vec![
            Duration::type_def(),
            AtomicU16::type_def(),
            Location::type_def(),
        ]
    }

    #[cfg(feature = "test_server")]
    fn test_infra() -> Vec<Type> {
        let mut defs = crate::test_server::test_defs();
        defs.push(ConfigImpl::type_def());
        defs
    }

    /// Returns all test function definitions for API reflection
    #[cfg(feature = "test_server")]
    fn test_defs() -> Vec<(String, Vec<Type>)> {
        vec![
            (
                "admin_db_add_test".to_owned(),
                crate::tests::routes::admin_db_add_test::test_defs(),
            ),
            (
                "admin_db_audit_test".to_owned(),
                crate::tests::routes::admin_db_audit_test::test_defs(),
            ),
            (
                "admin_db_backup_restore_test".to_owned(),
                crate::tests::routes::admin_db_backup_restore_test::test_defs(),
            ),
            (
                "admin_db_clear_test".to_owned(),
                crate::tests::routes::admin_db_clear_test::test_defs(),
            ),
            (
                "admin_db_convert_test".to_owned(),
                crate::tests::routes::admin_db_convert_test::test_defs(),
            ),
            (
                "admin_db_copy_test".to_owned(),
                crate::tests::routes::admin_db_copy_test::test_defs(),
            ),
            (
                "admin_db_delete_test".to_owned(),
                crate::tests::routes::admin_db_delete_test::test_defs(),
            ),
            (
                "admin_db_exec_test".to_owned(),
                crate::tests::routes::admin_db_exec_test::test_defs(),
            ),
            (
                "admin_db_list_test".to_owned(),
                crate::tests::routes::admin_db_list_test::test_defs(),
            ),
            (
                "admin_db_optimize_test".to_owned(),
                crate::tests::routes::admin_db_optimize_test::test_defs(),
            ),
            (
                "admin_db_remove_test".to_owned(),
                crate::tests::routes::admin_db_remove_test::test_defs(),
            ),
            (
                "admin_db_rename_test".to_owned(),
                crate::tests::routes::admin_db_rename_test::test_defs(),
            ),
            (
                "admin_db_user_add_test".to_owned(),
                crate::tests::routes::admin_db_user_add_test::test_defs(),
            ),
            (
                "admin_db_user_list_test".to_owned(),
                crate::tests::routes::admin_db_user_list_test::test_defs(),
            ),
            (
                "admin_db_user_remove_test".to_owned(),
                crate::tests::routes::admin_db_user_remove_test::test_defs(),
            ),
            (
                "admin_status_test".to_owned(),
                crate::tests::routes::admin_status_test::test_defs(),
            ),
            (
                "admin_user_add_test".to_owned(),
                crate::tests::routes::admin_user_add_test::test_defs(),
            ),
            (
                "admin_user_change_password_test".to_owned(),
                crate::tests::routes::admin_user_change_password_test::test_defs(),
            ),
            (
                "admin_user_delete_test".to_owned(),
                crate::tests::routes::admin_user_delete_test::test_defs(),
            ),
            (
                "admin_user_list_test".to_owned(),
                crate::tests::routes::admin_user_list_test::test_defs(),
            ),
            (
                "admin_user_logout_test".to_owned(),
                crate::tests::routes::admin_user_logout_test::test_defs(),
            ),
            (
                "db_add_test".to_owned(),
                crate::tests::routes::db_add_test::test_defs(),
            ),
            (
                "db_audit_test".to_owned(),
                crate::tests::routes::db_audit_test::test_defs(),
            ),
            (
                "db_backup_restore_test".to_owned(),
                crate::tests::routes::db_backup_restore_test::test_defs(),
            ),
            (
                "db_clear_test".to_owned(),
                crate::tests::routes::db_clear_test::test_defs(),
            ),
            (
                "db_convert_test".to_owned(),
                crate::tests::routes::db_convert_test::test_defs(),
            ),
            (
                "db_copy_test".to_owned(),
                crate::tests::routes::db_copy_test::test_defs(),
            ),
            (
                "db_delete_test".to_owned(),
                crate::tests::routes::db_delete_test::test_defs(),
            ),
            (
                "db_exec_test".to_owned(),
                crate::tests::routes::db_exec_test::test_defs(),
            ),
            (
                "db_list_test".to_owned(),
                crate::tests::routes::db_list_test::test_defs(),
            ),
            (
                "db_optimize_test".to_owned(),
                crate::tests::routes::db_optimize_test::test_defs(),
            ),
            (
                "db_remove_test".to_owned(),
                crate::tests::routes::db_remove_test::test_defs(),
            ),
            (
                "db_rename_test".to_owned(),
                crate::tests::routes::db_rename_test::test_defs(),
            ),
            (
                "db_user_add_test".to_owned(),
                crate::tests::routes::db_user_add_test::test_defs(),
            ),
            (
                "db_user_list_test".to_owned(),
                crate::tests::routes::db_user_list_test::test_defs(),
            ),
            (
                "db_user_remove_test".to_owned(),
                crate::tests::routes::db_user_remove_test::test_defs(),
            ),
            (
                "misc_routes_test".to_owned(),
                crate::tests::routes::misc_routes_test::test_defs(),
            ),
            (
                "cluster_test".to_owned(),
                crate::tests::routes::cluster_test::test_defs(),
            ),
            (
                "queries_test".to_owned(),
                crate::tests::queries::test_defs(),
            ),
            (
                "user_change_password_test".to_owned(),
                crate::tests::routes::user_change_password_test::test_defs(),
            ),
            (
                "user_login_test".to_owned(),
                crate::tests::routes::user_login_test::test_defs(),
            ),
            (
                "user_logout_test".to_owned(),
                crate::tests::routes::user_logout_test::test_defs(),
            ),
            (
                "user_logout_all_test".to_owned(),
                crate::tests::routes::user_logout_all_test::test_defs(),
            ),
            (
                "user_status_test".to_owned(),
                crate::tests::routes::user_status_test::test_defs(),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agdb::type_def::Expression;
    use agdb::type_def::Function;
    use agdb::type_def::Generic;
    use std::collections::HashSet;

    fn collect_from_generic(g: &Generic, out: &mut Vec<fn() -> Type>) {
        out.extend_from_slice(&g.bounds);
    }

    fn collect_from_function(f: &Function, out: &mut Vec<fn() -> Type>) {
        out.push(f.ret);
        for arg in &f.args {
            if let Some(ty) = arg.ty {
                out.push(ty);
            }
        }
        for g in &f.generics {
            collect_from_generic(g, out);
        }
    }

    fn collect_from_expression(expr: &Expression, out: &mut Vec<fn() -> Type>) {
        match expr {
            Expression::Array(items) | Expression::Tuple(items) | Expression::Block(items) => {
                for item in items {
                    collect_from_expression(item, out);
                }
            }
            Expression::Assign { target, value } => {
                collect_from_expression(target, out);
                collect_from_expression(value, out);
            }
            Expression::Await(inner)
            | Expression::Reference(inner)
            | Expression::Try(inner)
            | Expression::Unary { expr: inner, .. } => {
                collect_from_expression(inner, out);
            }
            Expression::Binary { left, right, .. } => {
                collect_from_expression(left, out);
                collect_from_expression(right, out);
            }
            Expression::Call {
                recipient,
                function,
                args,
            } => {
                if let Some(recipient) = recipient {
                    collect_from_expression(recipient, out);
                }
                collect_from_expression(function, out);
                for arg in args {
                    collect_from_expression(arg, out);
                }
            }
            Expression::Closure(function) => {
                collect_from_function(function, out);
                for expr in &function.body {
                    collect_from_expression(expr, out);
                }
            }
            Expression::FieldAccess { base, .. } | Expression::TupleAccess { base, .. } => {
                collect_from_expression(base, out);
            }
            Expression::For {
                pattern,
                iterable,
                body,
            } => {
                collect_from_expression(pattern, out);
                collect_from_expression(iterable, out);
                collect_from_expression(body, out);
            }
            Expression::Format { args, .. } => {
                for arg in args {
                    collect_from_expression(arg, out);
                }
            }
            Expression::If {
                condition,
                then_branch,
                else_branch,
            } => {
                collect_from_expression(condition, out);
                collect_from_expression(then_branch, out);
                if let Some(else_branch) = else_branch {
                    collect_from_expression(else_branch, out);
                }
            }
            Expression::Index { base, index } => {
                collect_from_expression(base, out);
                collect_from_expression(index, out);
            }
            Expression::Let { name, ty, value } => {
                collect_from_expression(name, out);
                if let Some(ty) = ty {
                    out.push(*ty);
                }
                if let Some(value) = value {
                    collect_from_expression(value, out);
                }
            }
            Expression::Path {
                parent, generics, ..
            } => {
                if let Some(parent) = parent {
                    collect_from_expression(parent, out);
                }
                out.extend_from_slice(generics);
            }
            Expression::Range {
                start,
                end,
                inclusive: _,
            } => {
                if let Some(start) = start {
                    collect_from_expression(start, out);
                }
                if let Some(end) = end {
                    collect_from_expression(end, out);
                }
            }
            Expression::Return(value) => {
                if let Some(value) = value {
                    collect_from_expression(value, out);
                }
            }
            Expression::Struct { name, fields } => {
                collect_from_expression(name, out);
                for (_, value) in fields {
                    collect_from_expression(value, out);
                }
            }
            Expression::StructPattern { name, fields } => {
                collect_from_expression(name, out);
                for field in fields {
                    collect_from_expression(field, out);
                }
            }
            Expression::TupleStruct { name, expressions } => {
                collect_from_expression(name, out);
                for expression in expressions {
                    collect_from_expression(expression, out);
                }
            }
            Expression::Match { scrutinee, arms } => {
                collect_from_expression(scrutinee, out);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        collect_from_expression(guard, out);
                    }
                    collect_from_expression(&arm.body, out);
                }
            }
            Expression::While { condition, body } => {
                collect_from_expression(condition, out);
                collect_from_expression(body, out);
            }
            Expression::Break
            | Expression::Continue
            | Expression::Ident(_)
            | Expression::Literal(_)
            | Expression::Wild => {}
            Expression::Match { scrutinee, arms } => {
                collect_from_expression(scrutinee, out);
                for arm in *arms {
                    if let Some(guard) = arm.guard {
                        collect_from_expression(guard, out);
                    }
                    collect_from_expression(arm.body, out);
                }
            }
        }
    }

    fn collect_from_function_body(function: &Function, out: &mut Vec<fn() -> Type>) {
        for expr in &function.body {
            collect_from_expression(expr, out);
        }
    }

    fn collect_from_impl(i: &agdb::type_def::Impl, out: &mut Vec<fn() -> Type>) {
        out.push(i.ty);
        if let Some(t) = i.trait_ {
            out.push(t);
        }
        for g in &i.generics {
            collect_from_generic(g, out);
        }
        for f in &i.functions {
            collect_from_function(f, out);
        }
    }

    fn collect_fn_ptrs(ty: &Type, out: &mut Vec<fn() -> Type>) {
        match ty {
            Type::Option(f) | Type::Slice(f) | Type::Vec(f) => out.push(*f),
            Type::Result { ok, err } => {
                out.push(*ok);
                out.push(*err);
            }
            Type::Reference(r) => out.push(r.ty),
            Type::Pointer(p) => out.push(p.ty),
            Type::Tuple(fns) => out.extend_from_slice(fns),
            Type::Enum(e) => {
                for v in &e.variants {
                    if let Some(f) = v.ty {
                        out.push(f);
                    }
                }
                for g in &e.generics {
                    collect_from_generic(g, out);
                }
                for i in &(e.impl_defs)() {
                    collect_from_impl(i, out);
                }
            }
            Type::Struct(s) => {
                for v in &s.fields {
                    if let Some(f) = v.ty {
                        out.push(f);
                    }
                }
                for g in &s.generics {
                    collect_from_generic(g, out);
                }
                for i in &(s.impl_defs)() {
                    collect_from_impl(i, out);
                }
            }
            Type::Function(f) | Type::Test(f) => collect_from_function(f, out),
            Type::Impl(i) => collect_from_impl(i, out),
            Type::Trait(t) => {
                out.extend_from_slice(&t.bounds);
                for g in &t.generics {
                    collect_from_generic(g, out);
                }
                for f in &t.functions {
                    collect_from_function(f, out);
                }
            }
            Type::Generic(g) => collect_from_generic(g, out),
            Type::Literal(_) | Type::SelfType(_) => {}
            Type::Static(s) => out.push(s.ty),
        }
    }

    fn type_name(ty: &Type) -> Option<&str> {
        match ty {
            Type::Struct(s) => Some(&s.name),
            Type::Enum(e) => Some(&e.name),
            Type::Trait(t) => Some(&t.name),
            _ => None,
        }
    }

    fn collect_missing_named_types() -> Vec<String> {
        let roots = Api::type_defs();
        let root_names: HashSet<&str> = roots.iter().map(|ty| ty.name()).collect();

        let mut visited: HashSet<usize> = HashSet::new();
        let mut queue: Vec<fn() -> Type> = Vec::new();

        for ty in &roots {
            collect_fn_ptrs(ty, &mut queue);
        }

        let mut missing: Vec<String> = Vec::new();

        while let Some(f) = queue.pop() {
            let addr = f as usize;
            if !visited.insert(addr) {
                continue;
            }
            let ty = f();

            if let Some(name) = type_name(&ty) {
                let in_catalog = root_names.contains(name);
                let is_external = EXTERNAL_TRAIT_ALLOWLIST.contains(&name);
                if !in_catalog && !is_external && !missing.iter().any(|m| m == name) {
                    missing.push(name.to_string());
                }
            }

            match &ty {
                Type::Function(function) | Type::Test(function) => {
                    collect_from_function_body(function, &mut queue);
                }
                Type::Impl(implementation) => {
                    for function in &implementation.functions {
                        collect_from_function_body(function, &mut queue);
                    }
                }
                Type::Trait(trait_def) => {
                    for function in &trait_def.functions {
                        collect_from_function_body(function, &mut queue);
                    }
                }
                _ => {}
            }

            collect_fn_ptrs(&ty, &mut queue);
        }

        missing
    }

    // Traits from external crates or Rust std that appear as generic bounds
    // but are not part of the agdb/agdb_api public API catalog.
    const EXTERNAL_TRAIT_ALLOWLIST: &[&str] = &[
        // std / core
        "Borrow",
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "Display",
        "Eq",
        "From",
        "Hash",
        "Into",
        "Iterator",
        "Ord",
        "PartialEq",
        "PartialOrd",
        "Send",
        "Sync",
        // serde
        "DeserializeOwned",
        "Deserialize",
        "Serialize",
        // agdb database traits (Rust-specific, not cross-language API types)
        "DbType",
        "DbTypeMarker",
        // agdb reflection meta-traits (infrastructure, not public API types)
        "TypeDefinition",
    ];

    #[test]
    fn into_bounds_capture_inner_type() {
        let impl_defs = Api::types();
        let insert_aliases_impl = impl_defs
            .iter()
            .find_map(|t| {
                if let Type::Struct(s) = t
                    && s.name == "InsertAliases"
                {
                    Some((s.impl_defs)())
                } else {
                    None
                }
            })
            .expect("InsertAliases impl should exist");
        let ids_fn = insert_aliases_impl[0]
            .functions
            .iter()
            .find(|f| f.name == "ids")
            .expect("ids method should exist");

        assert_eq!(ids_fn.generics.len(), 1);
        let generic = &ids_fn.generics[0];
        assert_eq!(generic.name, "T");

        let bound_type = (generic.bounds[0])();
        let Type::Trait(t) = &bound_type else {
            panic!("Expected Trait bound, got: {:?}", bound_type);
        };
        assert_eq!(t.name, "Into");
        assert!(
            !t.generics.is_empty(),
            "Into should have type arguments captured"
        );
        let inner = (t.generics[0].bounds[0])();
        assert_eq!(inner.name(), "QueryIds");
    }

    #[test]
    fn query_type_enum_variants_discoverable() {
        let type_defs = Api::types();
        let query_type = type_defs
            .iter()
            .find(|t| t.name() == "QueryType")
            .expect("QueryType should be in type_defs");
        let Type::Enum(e) = query_type else {
            panic!("QueryType should be an enum");
        };

        // Verify we can derive the variant → payload type mapping programmatically
        let mut mapping: Vec<(String, String)> = Vec::new();
        for v in &e.variants {
            if let Some(ty_fn) = v.ty {
                let ty = ty_fn();
                mapping.push((v.name.clone(), ty.name().to_string()));
            }
        }

        assert!(
            mapping
                .iter()
                .any(|(v, t)| v == "InsertAlias" && t == "InsertAliasesQuery")
        );
        assert!(
            mapping
                .iter()
                .any(|(v, t)| v == "InsertEdges" && t == "InsertEdgesQuery")
        );
        assert!(
            mapping
                .iter()
                .any(|(v, t)| v == "Remove" && t == "RemoveQuery")
        );
        assert!(
            mapping
                .iter()
                .any(|(v, t)| v == "Search" && t == "SearchQuery")
        );
        assert!(
            mapping
                .iter()
                .any(|(v, t)| v == "SelectValues" && t == "SelectValuesQuery")
        );
    }

    #[test]
    fn newtype_pattern_detectable() {
        let type_defs = Api::types();

        // RemoveAliases is a newtype: struct RemoveAliases(pub RemoveAliasesQuery)
        let remove_aliases = type_defs
            .iter()
            .find(|t| t.name() == "RemoveAliases")
            .expect("RemoveAliases should be in type_defs");
        let Type::Struct(s) = remove_aliases else {
            panic!("RemoveAliases should be a struct");
        };

        // Newtype pattern: exactly one field with empty name
        assert_eq!(s.fields.len(), 1);
        assert_eq!(s.fields[0].name, "");
        let field_type = (s.fields[0].ty.expect("field should have type"))();
        assert_eq!(field_type.name(), "RemoveAliasesQuery");
    }

    #[test]
    fn builder_method_bodies_available() {
        let impl_defs = Api::types();
        let insert_nodes_impl = impl_defs
            .iter()
            .find_map(|t| {
                if let Type::Struct(s) = t
                    && s.name == "InsertNodes"
                {
                    Some((s.impl_defs)())
                } else {
                    None
                }
            })
            .expect("InsertNodes impl should exist");

        // Methods should have non-empty bodies (body parsing works)
        for func in &insert_nodes_impl[0].functions {
            if func.name != "new" && func.name != "default" {
                assert!(
                    !func.body.is_empty(),
                    "Method '{}' should have a non-empty body",
                    func.name
                );
            }
        }
    }

    #[test]
    fn all_fn_ptrs_resolvable() {
        let roots = Api::types();
        let mut visited: HashSet<usize> = HashSet::new();
        let mut queue: Vec<fn() -> Type> = Vec::new();

        for ty in &roots {
            collect_fn_ptrs(ty, &mut queue);
        }

        while let Some(f) = queue.pop() {
            let addr = f as usize;
            if !visited.insert(addr) {
                continue;
            }
            let ty = f();
            collect_fn_ptrs(&ty, &mut queue);
        }
    }

    #[test]
    fn all_named_types_in_catalog() {
        let missing = collect_missing_named_types();

        assert!(
            missing.is_empty(),
            "The following types are referenced in the API type graph and bodies \
             but are not listed in Api::type_defs():\n  {}\n\
             Add them to the catalog or to EXTERNAL_TRAIT_ALLOWLIST if they are external.",
            missing.join(", ")
        );
    }
}
