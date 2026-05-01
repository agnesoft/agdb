use agdb::Comparison;
use agdb::CountComparison;
use agdb::DbElement;
use agdb::DbF64;
use agdb::DbId;
use agdb::DbKeyOrder;
use agdb::DbKeyOrders;
use agdb::DbKeyValue;
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

use crate::AdminStatus;
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
use crate::ServerDatabase;
use crate::UserCredentials;
use crate::UserLogin;
use crate::UserStatus;

pub struct Api;

impl Api {
    pub fn type_defs() -> Vec<Type> {
        vec![
            // agdb DB types
            DbElement::type_def(),
            DbF64::type_def(),
            DbId::type_def(),
            DbKeyOrder::type_def(),
            DbKeyOrders::type_def(),
            DbKeyValue::type_def(),
            DbValue::type_def(),
            DbValues::type_def(),
            // agdb query types
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
            // agdb QueryBuilder types
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
            // Generic search/where builder types (monomorphised with SearchQuery)
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
            // agdb_api traits
            SearchQueryBuilderDef::type_def(),
            HttpClientDef::type_def(),
            AgdbApiClientDef::type_def(),
            // agdb_api types
            AgdbApiError::type_def(),
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
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agdb::type_def::Function;
    use agdb::type_def::Generic;
    use std::collections::HashSet;

    fn collect_from_generic(g: &Generic, out: &mut Vec<fn() -> Type>) {
        out.extend_from_slice(g.bounds);
    }

    fn collect_from_function(f: &Function, out: &mut Vec<fn() -> Type>) {
        out.push(f.ret);
        for arg in f.args {
            if let Some(ty) = arg.ty {
                out.push(ty);
            }
        }
        for g in f.generics {
            collect_from_generic(g, out);
        }
    }

    fn collect_from_impl(i: &agdb::type_def::Impl, out: &mut Vec<fn() -> Type>) {
        out.push(i.ty);
        if let Some(t) = i.trait_ {
            out.push(t);
        }
        for g in i.generics {
            collect_from_generic(g, out);
        }
        for f in i.functions {
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
                for v in e.variants {
                    if let Some(f) = v.ty {
                        out.push(f);
                    }
                }
                for g in e.generics {
                    collect_from_generic(g, out);
                }
            }
            Type::Struct(s) => {
                for v in s.fields {
                    if let Some(f) = v.ty {
                        out.push(f);
                    }
                }
                for g in s.generics {
                    collect_from_generic(g, out);
                }
            }
            Type::Function(f) | Type::Test(f) => collect_from_function(f, out),
            Type::Impl(i) => collect_from_impl(i, out),
            Type::Trait(t) => {
                out.extend_from_slice(t.bounds);
                for g in t.generics {
                    collect_from_generic(g, out);
                }
                for f in t.functions {
                    collect_from_function(f, out);
                }
            }
            Type::Generic(g) => collect_from_generic(g, out),
            Type::Literal(_) | Type::SelfType(_) => {}
        }
    }

    #[test]
    fn all_fn_ptrs_resolvable() {
        let roots = Api::type_defs();
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

    fn type_name(ty: &Type) -> Option<&'static str> {
        match ty {
            Type::Struct(s) => Some(s.name),
            Type::Enum(e) => Some(e.name),
            Type::Trait(t) => Some(t.name),
            _ => None,
        }
    }

    #[test]
    fn all_named_types_in_catalog() {
        let roots = Api::type_defs();

        // Build set of all root type names.
        let root_names: HashSet<&'static str> = roots.iter().map(|ty| ty.name()).collect();

        let mut visited: HashSet<usize> = HashSet::new();
        let mut queue: Vec<fn() -> Type> = Vec::new();

        for ty in &roots {
            collect_fn_ptrs(ty, &mut queue);
        }

        let mut missing: Vec<&'static str> = Vec::new();

        while let Some(f) = queue.pop() {
            let addr = f as usize;
            if !visited.insert(addr) {
                continue;
            }
            let ty = f();

            if let Some(name) = type_name(&ty) {
                let in_catalog = root_names.contains(name);
                let is_external = EXTERNAL_TRAIT_ALLOWLIST.contains(&name);
                if !in_catalog && !is_external && !missing.contains(&name) {
                    missing.push(name);
                }
            }

            collect_fn_ptrs(&ty, &mut queue);
        }

        assert!(
            missing.is_empty(),
            "The following types are referenced in the API type graph \
             but are not listed in Api::type_defs():\n  {}\n\
             Add them to the catalog or to EXTERNAL_TRAIT_ALLOWLIST if they are external.",
            missing.join(", ")
        );
    }
}
