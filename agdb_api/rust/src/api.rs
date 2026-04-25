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
use agdb::type_def::Impl;
use agdb::type_def::ImplDefinition;
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

macro_rules! type_entry {
    ($ty:ty) => {
        (<$ty>::type_def(), vec![])
    };
    ($ty:ty, impl) => {
        (<$ty>::type_def(), vec![<$ty>::impl_def()])
    };
}

pub struct Api;

impl Api {
    pub fn type_defs() -> Vec<(Type, Vec<Impl>)> {
        vec![
            // agdb DB types
            type_entry!(DbElement),
            type_entry!(DbF64),
            type_entry!(DbId),
            type_entry!(DbKeyOrder),
            type_entry!(DbKeyOrders),
            type_entry!(DbKeyValue),
            type_entry!(DbValue),
            type_entry!(DbValues),
            // agdb query types
            type_entry!(QueryType),
            type_entry!(QueryAliases),
            type_entry!(InsertAliasesQuery),
            type_entry!(InsertEdgesQuery),
            type_entry!(InsertIndexQuery),
            type_entry!(InsertNodesQuery),
            type_entry!(InsertValuesQuery),
            type_entry!(Comparison),
            type_entry!(CountComparison),
            type_entry!(KeyValueComparison),
            type_entry!(QueryCondition),
            type_entry!(QueryConditionData),
            type_entry!(QueryConditionLogic),
            type_entry!(QueryConditionModifier),
            type_entry!(QueryId),
            type_entry!(QueryIds),
            type_entry!(QueryResult),
            type_entry!(QueryValues),
            type_entry!(SingleValues),
            type_entry!(MultiValues),
            type_entry!(RemoveAliasesQuery),
            type_entry!(RemoveIndexQuery),
            type_entry!(RemoveQuery),
            type_entry!(RemoveValuesQuery),
            type_entry!(SearchQuery),
            type_entry!(SearchQueryAlgorithm),
            type_entry!(SelectAliasesQuery),
            type_entry!(SelectAllAliasesQuery),
            type_entry!(SelectEdgeCountQuery),
            type_entry!(SelectIndexesQuery),
            type_entry!(SelectKeyCountQuery),
            type_entry!(SelectKeysQuery),
            type_entry!(SelectNodeCountQuery),
            type_entry!(SelectValuesQuery),
            // agdb QueryBuilder types (with impl blocks)
            type_entry!(QueryBuilder, impl),
            type_entry!(Insert, impl),
            type_entry!(InsertAliases, impl),
            type_entry!(InsertAliasesIds, impl),
            type_entry!(InsertEdges, impl),
            type_entry!(InsertEdgesEach, impl),
            type_entry!(InsertEdgesFrom, impl),
            type_entry!(InsertEdgesFromTo, impl),
            type_entry!(InsertEdgesIds, impl),
            type_entry!(InsertEdgesValues, impl),
            type_entry!(InsertIndex, impl),
            type_entry!(InsertNodes, impl),
            type_entry!(InsertNodesAliases, impl),
            type_entry!(InsertNodesCount, impl),
            type_entry!(InsertNodesIds, impl),
            type_entry!(InsertNodesValues, impl),
            type_entry!(InsertValues, impl),
            type_entry!(InsertValuesIds, impl),
            type_entry!(Remove, impl),
            type_entry!(RemoveAliases, impl),
            type_entry!(RemoveIds, impl),
            type_entry!(RemoveIndex, impl),
            type_entry!(RemoveValues, impl),
            type_entry!(RemoveValuesIds, impl),
            // Generic search/where builder types (monomorphised with SearchQuery)
            type_entry!(Search<SearchQuery>, impl),
            type_entry!(SearchAlgorithm<SearchQuery>, impl),
            type_entry!(SearchFrom<SearchQuery>, impl),
            type_entry!(SearchTo<SearchQuery>, impl),
            type_entry!(SearchIndexBuilder<SearchQuery>, impl),
            type_entry!(SearchIndexValue<SearchQuery>, impl),
            type_entry!(SearchOrderBy<SearchQuery>, impl),
            type_entry!(SelectLimit<SearchQuery>, impl),
            type_entry!(SelectOffset<SearchQuery>, impl),
            type_entry!(Select, impl),
            type_entry!(SelectAliases, impl),
            type_entry!(SelectAliasesIds, impl),
            type_entry!(SelectEdgeCount, impl),
            type_entry!(SelectEdgeCountIds, impl),
            type_entry!(SelectIds, impl),
            type_entry!(SelectIndexes, impl),
            type_entry!(SelectKeyCount, impl),
            type_entry!(SelectKeyCountIds, impl),
            type_entry!(SelectKeys, impl),
            type_entry!(SelectKeysIds, impl),
            type_entry!(SelectNodeCount, impl),
            type_entry!(SelectValues, impl),
            type_entry!(SelectValuesIds, impl),
            type_entry!(Where<SearchQuery>, impl),
            type_entry!(WhereKey<SearchQuery>, impl),
            type_entry!(WhereLogicOperator<SearchQuery>, impl),
            // agdb_api traits
            type_entry!(SearchQueryBuilderDef),
            type_entry!(HttpClientDef),
            type_entry!(AgdbApiClientDef),
            // agdb_api types
            type_entry!(AgdbApiError),
            type_entry!(AdminStatus),
            type_entry!(ChangePassword),
            type_entry!(ClusterStatus),
            type_entry!(DbAudit),
            type_entry!(DbKind),
            type_entry!(DbResource),
            type_entry!(DbUser),
            type_entry!(DbUserRole),
            type_entry!(LogLevelFilter),
            type_entry!(QueryAudit),
            type_entry!(ServerDatabase),
            type_entry!(UserCredentials),
            type_entry!(UserLogin),
            type_entry!(UserStatus),
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

    fn collect_from_impl(i: &Impl, out: &mut Vec<fn() -> Type>) {
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
                for f in e.functions {
                    collect_from_function(f, out);
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
                for f in s.functions {
                    collect_from_function(f, out);
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

        for (ty, impls) in &roots {
            collect_fn_ptrs(ty, &mut queue);
            for impl_ in impls {
                collect_from_impl(impl_, &mut queue);
            }
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
        "ImplDefinition",
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
        let root_names: HashSet<&'static str> = roots.iter().map(|(ty, _)| ty.name()).collect();

        let mut visited: HashSet<usize> = HashSet::new();
        let mut queue: Vec<fn() -> Type> = Vec::new();

        for (ty, impls) in &roots {
            collect_fn_ptrs(ty, &mut queue);
            for impl_ in impls {
                collect_from_impl(impl_, &mut queue);
            }
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
