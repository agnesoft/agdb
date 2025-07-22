declare namespace agdb {
export type DbF64 = number;
export type DbId = number;
export type QueryId = DbId | string;
export type QueryIds = QueryId[] | SearchQuery;
export type QueryValues = DbKeyValue[] | DbKeyValue[][];
export type DbValue = number[] | number | number | DbF64 | string | number[] | number[] | DbF64[] | string[];
export type DbValues = DbValue[];
export type DbKeyValue = {
    key: DbValue;
    value: DbValue;
}
export type QueryAliases = string[];
export type SingleValues = DbKeyValue[];
export type MultiValues = DbKeyValue[][];
export type InsertAliasesQuery = {
    ids: QueryIds;
    aliases: string[];
}
export type InsertEdgesQuery = {
    from: QueryIds;
    to: QueryIds;
    ids: QueryIds;
    values: QueryValues;
    each: number;
}
export type InsertIndexQuery = DbValue;
export type InsertNodesQuery = {
    count: number;
    values: QueryValues;
    aliases: string[];
    ids: QueryIds;
}
export type InsertValuesQuery = {
    ids: QueryIds;
    values: QueryValues;
}
export type RemoveAliasesQuery = string[];
export type RemoveIndexQuery = DbValue;
export type RemoveQuery = QueryIds;
export type RemoveValuesQuery = SelectValuesQuery;
export type SearchQuery = {
    algorithm: SearchQueryAlgorithm;
    origin: QueryId;
    destination: QueryId;
    limit: number;
    offset: number;
    order_by: DbKeyOrder[];
    conditions: QueryCondition[];
}
export type SearchQueryAlgorithm = undefined | undefined | undefined | undefined;
export type SelectAliasesQuery = QueryIds;
export type SelectAllAliasesQuery = {
}
export type SelectEdgeCountQuery = {
    ids: QueryIds;
    from: number;
    to: number;
}
export type SelectIndexesQuery = {
}
export type SelectKeyCountQuery = QueryIds;
export type SelectKeysQuery = QueryIds;
export type SelectNodeCountQuery = {
}
export type SelectValuesQuery = {
    keys: DbValue[];
    ids: QueryIds;
}
export type DbKeyOrder = DbValue | DbValue;
export type QueryCondition = {
    logic: QueryConditionLogic;
    modifier: QueryConditionModifier;
    data: QueryConditionData;
}
export type QueryConditionLogic = undefined | undefined;
export type QueryConditionModifier = undefined | undefined | undefined | undefined;
export type QueryConditionData = CountComparison | undefined | CountComparison | CountComparison | CountComparison | QueryId[] | KeyValueComparison | DbValue[] | undefined | QueryCondition[];
export type CountComparison = number | number | number | number | number | number;
export type Comparison = DbValue | DbValue | DbValue | DbValue | DbValue | DbValue | DbValue;
export type KeyValueComparison = {
    key: DbValue;
    value: Comparison;
}
export class QueryBuilder {

    insert(): Insert {
    }
    remove(): Remove {
    }
    search(): Search {
    }
    select(): Select {
    }
}
export class Insert {

    aliases(names: QueryAliases): InsertAliases {
    }
    edges(): InsertEdges {
    }
    element(elem: any): InsertValuesIds {
    }
    elements(elems: any[]): InsertValuesIds {
    }
    index(key: DbValue): InsertIndex {
    }
    nodes(): InsertNodes {
    }
    values(key_values: MultiValues): InsertValues {
    }
    values_uniform(key_values: SingleValues): InsertValues {
    }
}
export class InsertAliases {
    data: InsertAliasesQuery;

    ids(ids: QueryIds): InsertAliasesIds {
    }
}
export class InsertAliasesIds {
    data: InsertAliasesQuery;

    query(): InsertAliasesQuery {
    }
}
export class InsertEdges {
    data: InsertEdgesQuery;

    from(ids: QueryIds): InsertEdgesFrom {
    }
    ids(ids: QueryIds): InsertEdgesIds {
    }
}
export class InsertEdgesEach {
    data: InsertEdgesQuery;

    query(): InsertEdgesQuery {
    }
    values(key_values: MultiValues): InsertEdgesValues {
    }
    values_uniform(key_values: SingleValues): InsertEdgesValues {
    }
}
export class InsertEdgesFrom {
    data: InsertEdgesQuery;

    to(ids: QueryIds): InsertEdgesFromTo {
    }
}
export class InsertEdgesFromTo {
    data: InsertEdgesQuery;

    each(): InsertEdgesEach {
    }
    query(): InsertEdgesQuery {
    }
    values(key_values: MultiValues): InsertEdgesValues {
    }
    values_uniform(key_values: SingleValues): InsertEdgesValues {
    }
}
export class InsertEdgesIds {
    data: InsertEdgesQuery;

    from(ids: QueryIds): InsertEdgesFrom {
    }
}
export class InsertEdgesValues {
    data: InsertEdgesQuery;

    query(): InsertEdgesQuery {
    }
}
export class InsertIndex {
    data: DbValue;

    query(): InsertIndexQuery {
    }
}
export class InsertNodes {
    data: InsertNodesQuery;

    aliases(names: QueryAliases): InsertNodesAliases {
    }
    count(num: number): InsertNodesCount {
    }
    ids(ids: QueryIds): InsertNodesIds {
    }
    values(key_values: MultiValues): InsertNodesValues {
    }
}
export class InsertNodesAliases {
    data: InsertNodesQuery;

    query(): InsertNodesQuery {
    }
    values(key_values: MultiValues): InsertNodesValues {
    }
    values_uniform(key_values: SingleValues): InsertNodesValues {
    }
}
export class InsertNodesCount {
    data: InsertNodesQuery;

    query(): InsertNodesQuery {
    }
    values_uniform(key_values: SingleValues): InsertNodesValues {
    }
}
export class InsertNodesIds {
    data: InsertNodesQuery;

    aliases(names: QueryAliases): InsertNodesAliases {
    }
    count(num: number): InsertNodesCount {
    }
    values(key_values: MultiValues): InsertNodesValues {
    }
    values_uniform(key_values: SingleValues): InsertNodesValues {
    }
}
export class InsertNodesValues {
    data: InsertNodesQuery;

    query(): InsertNodesQuery {
    }
}
export class InsertValues {
    data: InsertValuesQuery;

    ids(ids: QueryIds): InsertValuesIds {
    }
    search(): Search {
    }
}
export class InsertValuesIds {
    data: InsertValuesQuery;

    query(): InsertValuesQuery {
    }
}
export class Remove {

    aliases(names: QueryAliases): RemoveAliases {
    }
    ids(ids: QueryIds): RemoveIds {
    }
    index(key: DbValue): RemoveIndex {
    }
    search(): Search {
    }
    values(keys: DbValues): RemoveValues {
    }
}
export class RemoveAliases {
    data: RemoveAliasesQuery;

    query(): RemoveAliasesQuery {
    }
}
export class RemoveIds {
    data: RemoveQuery;

    query(): RemoveQuery {
    }
}
export class RemoveIndex {
    data: DbValue;

    query(): RemoveIndexQuery {
    }
}
export class RemoveValues {
    data: RemoveValuesQuery;

    ids(ids: QueryIds): RemoveValuesIds {
    }
    search(): Search {
    }
}
export class RemoveValuesIds {
    data: RemoveValuesQuery;

    query(): RemoveValuesQuery {
    }
}
export class Select {

    aliases(): SelectAliases {
    }
    edge_count(): SelectEdgeCount {
    }
    edge_count_from(): SelectEdgeCount {
    }
    edge_count_to(): SelectEdgeCount {
    }
    elements(): SelectValues {
    }
    ids(ids: QueryIds): SelectIds {
    }
    indexes(): SelectIndexes {
    }
    keys(): SelectKeys {
    }
    key_count(): SelectKeyCount {
    }
    node_count(): SelectNodeCount {
    }
    search(): Search {
    }
    values(keys: DbValues): SelectValues {
    }
}
export class SelectAliases {
    data: SelectAliasesQuery;

    ids(ids: QueryIds): SelectAliasesIds {
    }
    search(): Search {
    }
    query(): SelectAllAliasesQuery {
    }
}
export class SelectAliasesIds {
    data: SelectAliasesQuery;

    query(): SelectAliasesQuery {
    }
}
export class SelectEdgeCount {
    data: SelectEdgeCountQuery;

    ids(ids: QueryIds): SelectEdgeCountIds {
    }
    search(): Search {
    }
}
export class SelectEdgeCountIds {
    data: SelectEdgeCountQuery;

    query(): SelectEdgeCountQuery {
    }
}
export class SelectIds {
    data: SelectValuesQuery;

    query(): SelectValuesQuery {
    }
}
export class SelectIndexes {

    query(): SelectIndexesQuery {
    }
}
export class SelectKeys {
    data: SelectKeysQuery;

    ids(ids: QueryIds): SelectKeysIds {
    }
    search(): Search {
    }
}
export class SelectKeysIds {
    data: SelectKeysQuery;

    query(): SelectKeysQuery {
    }
}
export class SelectKeyCount {
    data: SelectKeyCountQuery;

    ids(ids: QueryIds): SelectKeyCountIds {
    }
    search(): Search {
    }
}
export class SelectKeyCountIds {
    data: SelectKeyCountQuery;

    query(): SelectKeyCountQuery {
    }
}
export class SelectNodeCount {

    query(): SelectNodeCountQuery {
    }
}
export class SelectValues {
    data: SelectValuesQuery;

    ids(ids: QueryIds): SelectValuesIds {
    }
    search(): Search {
    }
}
export class SelectValuesIds {
    data: SelectValuesQuery;

    query(): SelectValuesQuery {
    }
}
export class Search {
    data: SearchQueryBuilderHelper;

    breadth_first(): SearchAlgorithm {
    }
    depth_first(): SearchAlgorithm {
    }
    elements(): SearchTo {
    }
    index(key: DbValue): SearchIndex {
    }
    from(id: QueryId): SearchFrom {
    }
    to(id: QueryId): SearchTo {
    }
}
export class SearchAlgorithm {
    data: SearchQueryBuilderHelper;

    from(id: QueryId): SearchFrom {
    }
    to(id: QueryId): SearchTo {
    }
}
export class SearchFrom {
    data: SearchQueryBuilderHelper;

    limit(value: number): SelectLimit {
    }
    offset(value: number): SelectOffset {
    }
    order_by(keys: DbKeyOrders): SearchOrderBy {
    }
    query(): SearchQueryBuilderHelper {
    }
    to(id: QueryId): SearchTo {
    }
    where_(): Where {
    }
}
export class SearchTo {
    data: SearchQueryBuilderHelper;

    limit(value: number): SelectLimit {
    }
    offset(value: number): SelectOffset {
    }
    order_by(keys: DbKeyOrders): SearchOrderBy {
    }
    query(): SearchQueryBuilderHelper {
    }
    where_(): Where {
    }
}
export class SearchIndex {
    index: DbValue;
    query: SearchQueryBuilderHelper;

    value(value: DbValue): SearchIndexValue {
    }
}
export class SearchIndexValue {
    data: SearchQueryBuilderHelper;

    query(): SearchQueryBuilderHelper {
    }
}
export class SearchOrderBy {
    data: SearchQueryBuilderHelper;

    limit(value: number): SelectLimit {
    }
    offset(value: number): SelectOffset {
    }
    query(): SearchQueryBuilderHelper {
    }
    where_(): Where {
    }
}
export class SelectLimit {
    data: SearchQueryBuilderHelper;

    query(): SearchQueryBuilderHelper {
    }
    where_(): Where {
    }
}
export class SelectOffset {
    data: SearchQueryBuilderHelper;

    limit(value: number): SelectLimit {
    }
    query(): SearchQueryBuilderHelper {
    }
    where_(): Where {
    }
}
export class Where {
    logic: QueryConditionLogic;
    modifier: QueryConditionModifier;
    conditions: QueryCondition[][];
    query: SearchQueryBuilderHelper;

    beyond(): Where {
    }
    distance(comparison: CountComparison): WhereLogicOperator {
    }
    edge(): WhereLogicOperator {
    }
    edge_count(comparison: CountComparison): WhereLogicOperator {
    }
    edge_count_from(comparison: CountComparison): WhereLogicOperator {
    }
    edge_count_to(comparison: CountComparison): WhereLogicOperator {
    }
    ids(ids: QueryIds): WhereLogicOperator {
    }
    key(key: DbValue): WhereKey {
    }
    keys(keys: DbValues): WhereLogicOperator {
    }
    node(): WhereLogicOperator {
    }
    not(): Where {
    }
    not_beyond(): Where {
    }
    where_(): Where {
    }
    new(query: SearchQueryBuilderHelper): Where {
    }
    add_condition(condition: QueryCondition) {
    }
    collapse_conditions(): number {
    }
}
export class WhereKey {
    key: DbValue;
    where_: Where;

    value(comparison: Comparison): WhereLogicOperator {
    }
}
export class WhereLogicOperator {
    data: Where;

    and(): Where {
    }
    end_where(): WhereLogicOperator {
    }
    or(): Where {
    }
    query(): SearchQueryBuilderHelper {
    }
}
export type SearchQueryBuilderHelper = {
    search: SearchQuery;
}
}
