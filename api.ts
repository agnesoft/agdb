declare namespace agdb {
export type DbF64 = number;
export type DbId = number;
export interface QueryId {
    Id: DbId,
    Alias: string,
}
export interface QueryIds {
    Ids: QueryId[],
    Search: SearchQuery,
}
export interface QueryValues {
    Single: DbKeyValue[],
    Multi: DbKeyValue[][],
}
export interface DbValue {
    Bytes: number[],
    I64: number,
    U64: number,
    F64: DbF64,
    String: string,
    VecI64: number[],
    VecU64: number[],
    VecF64: DbF64[],
    VecString: string[],
}
export type DbValues = DbValue[];
export type DbKeyValue = {
    key: DbValue,
    value: DbValue,
}
export type QueryAliases = string[];
export type SingleValues = DbKeyValue[];
export type MultiValues = DbKeyValue[][];
export type InsertAliasesQuery = {
    ids: QueryIds,
    aliases: string[],
}
export type InsertEdgesQuery = {
    from: QueryIds,
    to: QueryIds,
    ids: QueryIds,
    values: QueryValues,
    each: number,
}
export type InsertIndexQuery = DbValue;
export type InsertNodesQuery = {
    count: number,
    values: QueryValues,
    aliases: string[],
    ids: QueryIds,
}
export type InsertValuesQuery = {
    ids: QueryIds,
    values: QueryValues,
}
export type RemoveAliasesQuery = string[];
export type RemoveIndexQuery = DbValue;
export type RemoveQuery = QueryIds;
export type RemoveValuesQuery = SelectValuesQuery;
export type SearchQuery = {
    algorithm: SearchQueryAlgorithm,
    origin: QueryId,
    destination: QueryId,
    limit: number,
    offset: number,
    order_by: DbKeyOrder[],
    conditions: QueryCondition[],
}
export interface SearchQueryAlgorithm {
    BreadthFirst: undefined,
    DepthFirst: undefined,
    Index: undefined,
    Elements: undefined,
}
export type SelectAliasesQuery = QueryIds;
export type SelectAllAliasesQuery = {
}
export type SelectEdgeCountQuery = {
    ids: QueryIds,
    from: number,
    to: number,
}
export type SelectIndexesQuery = {
}
export type SelectKeyCountQuery = QueryIds;
export type SelectKeysQuery = QueryIds;
export type SelectNodeCountQuery = {
}
export type SelectValuesQuery = {
    keys: DbValue[],
    ids: QueryIds,
}
export interface DbKeyOrder {
    Asc: DbValue,
    Desc: DbValue,
}
export type QueryCondition = {
    logic: QueryConditionLogic,
    modifier: QueryConditionModifier,
    data: QueryConditionData,
}
export interface QueryConditionLogic {
    And: undefined,
    Or: undefined,
}
export interface QueryConditionModifier {
    None: undefined,
    Beyond: undefined,
    Not: undefined,
    NotBeyond: undefined,
}
export interface QueryConditionData {
    Distance: CountComparison,
    Edge: undefined,
    EdgeCount: CountComparison,
    EdgeCountFrom: CountComparison,
    EdgeCountTo: CountComparison,
    Ids: QueryId[],
    KeyValue: KeyValueComparison,
    Keys: DbValue[],
    Node: undefined,
    Where: QueryCondition[],
}
export interface CountComparison {
    Equal: number,
    GreaterThan: number,
    GreaterThanOrEqual: number,
    LessThan: number,
    LessThanOrEqual: number,
    NotEqual: number,
}
export interface Comparison {
    Equal: DbValue,
    GreaterThan: DbValue,
    GreaterThanOrEqual: DbValue,
    LessThan: DbValue,
    LessThanOrEqual: DbValue,
    NotEqual: DbValue,
    Contains: DbValue,
}
export type KeyValueComparison = {
    key: DbValue,
    value: Comparison,
}
export type QueryBuilder = {
}
export type Insert = {
}
export type InsertAliases = InsertAliasesQuery;
export type InsertAliasesIds = InsertAliasesQuery;
export type InsertEdges = InsertEdgesQuery;
export type InsertEdgesEach = InsertEdgesQuery;
export type InsertEdgesFrom = InsertEdgesQuery;
export type InsertEdgesFromTo = InsertEdgesQuery;
export type InsertEdgesIds = InsertEdgesQuery;
export type InsertEdgesValues = InsertEdgesQuery;
export type InsertIndex = DbValue;
export type InsertNodes = InsertNodesQuery;
export type InsertNodesAliases = InsertNodesQuery;
export type InsertNodesCount = InsertNodesQuery;
export type InsertNodesIds = InsertNodesQuery;
export type InsertNodesValues = InsertNodesQuery;
export type InsertValues = InsertValuesQuery;
export type InsertValuesIds = InsertValuesQuery;
export type Remove = {
}
export type RemoveAliases = RemoveAliasesQuery;
export type RemoveIds = RemoveQuery;
export type RemoveIndex = DbValue;
export type RemoveValues = RemoveValuesQuery;
export type RemoveValuesIds = RemoveValuesQuery;
export type Select = {
}
export type SelectAliases = SelectAliasesQuery;
export type SelectAliasesIds = SelectAliasesQuery;
export type SelectEdgeCount = SelectEdgeCountQuery;
export type SelectEdgeCountIds = SelectEdgeCountQuery;
export type SelectIds = SelectValuesQuery;
export type SelectIndexes = {
}
export type SelectKeys = SelectKeysQuery;
export type SelectKeysIds = SelectKeysQuery;
export type SelectKeyCount = SelectKeyCountQuery;
export type SelectKeyCountIds = SelectKeyCountQuery;
export type SelectNodeCount = {
}
export type SelectValues = SelectValuesQuery;
export type SelectValuesIds = SelectValuesQuery;
export type Search = SearchQueryBuilderHelper;
export type SearchAlgorithm = SearchQueryBuilderHelper;
export type SearchFrom = SearchQueryBuilderHelper;
export type SearchTo = SearchQueryBuilderHelper;
export type SearchIndex = {
    index: DbValue,
    query: SearchQueryBuilderHelper,
}
export type SearchIndexValue = SearchQueryBuilderHelper;
export type SearchOrderBy = SearchQueryBuilderHelper;
export type SelectLimit = SearchQueryBuilderHelper;
export type SelectOffset = SearchQueryBuilderHelper;
export type Where = {
    logic: QueryConditionLogic,
    modifier: QueryConditionModifier,
    conditions: QueryCondition[][],
    query: SearchQueryBuilderHelper,
}
export type WhereKey = {
    key: DbValue,
    where_: Where,
}
export type WhereLogicOperator = Where;
export type SearchQueryBuilderHelper = {
    search: SearchQuery,
}
}
