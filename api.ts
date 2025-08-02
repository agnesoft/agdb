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
        return new Insert;
    }
    remove(): Remove {
        return new Remove;
    }
    search(): Search {
        return new Search { data: SearchQuery.new() };
    }
    select(): Select {
        return new Select;
    }
}
export class Insert {

    aliases(names: QueryAliases): InsertAliases {
        return InsertAliases(new InsertAliasesQuery {
    ids: QueryIds.Ids([]),
    aliases: Into.into(names).0,
}
)    }
    edges(): InsertEdges {
        return InsertEdges(new InsertEdgesQuery {
    from: QueryIds.Ids([]),
    to: QueryIds.Ids([]),
    ids: QueryIds.Ids([]),
    values: QueryValues.Single([]),
    each: false,
}
)    }
    element(elem: any): InsertValuesIds {
        return InsertValuesIds(new InsertValuesQuery {
    ids: QueryIds.Ids([]),
    values: QueryValues.Multi([]),
}
)    }
    elements(elems: any[]): InsertValuesIds {
const ids = []const values = []ids.reserve(elems.len())values.reserve(elems.len())elems.iter().for_each((): () => {
ids.push(v.db_id().unwrap_or_default())values.push(v.to_db_values())}
)        return InsertValuesIds(new InsertValuesQuery {
    ids: QueryIds.Ids(ids),
    values: QueryValues.Multi(values),
}
)    }
    index(key: DbValue): InsertIndex {
        return InsertIndex(key.into())    }
    nodes(): InsertNodes {
        return InsertNodes(new InsertNodesQuery {
    count: 0,
    values: QueryValues.Single([]),
    aliases: [],
    ids: QueryIds.Ids([]),
}
)    }
    values(key_values: MultiValues): InsertValues {
        return InsertValues(new InsertValuesQuery {
    ids: QueryIds.Ids([]),
    values: QueryValues.Multi(Into.into(key_values).0),
}
)    }
    values_uniform(key_values: SingleValues): InsertValues {
        return InsertValues(new InsertValuesQuery {
    ids: QueryIds.Ids([]),
    values: QueryValues.Single(Into.into(key_values).0),
}
)    }
}
export class InsertAliases {
    data: InsertAliasesQuery;

    ids(ids: QueryIds): InsertAliasesIds {
self.0.ids = ids.into()        return InsertAliasesIds(self.0)    }
}
export class InsertAliasesIds {
    data: InsertAliasesQuery;

    query(): InsertAliasesQuery {
        return self.0    }
}
export class InsertEdges {
    data: InsertEdgesQuery;

    from(ids: QueryIds): InsertEdgesFrom {
self.0.from = ids.into()        return InsertEdgesFrom(self.0)    }
    ids(ids: QueryIds): InsertEdgesIds {
self.0.ids = ids.into()        return InsertEdgesIds(self.0)    }
}
export class InsertEdgesEach {
    data: InsertEdgesQuery;

    query(): InsertEdgesQuery {
        return self.0    }
    values(key_values: MultiValues): InsertEdgesValues {
self.0.values = QueryValues.Multi(Into.into(key_values).0)        return InsertEdgesValues(self.0)    }
    values_uniform(key_values: SingleValues): InsertEdgesValues {
self.0.values = QueryValues.Single(Into.into(key_values).0)        return InsertEdgesValues(self.0)    }
}
export class InsertEdgesFrom {
    data: InsertEdgesQuery;

    to(ids: QueryIds): InsertEdgesFromTo {
self.0.to = ids.into()        return InsertEdgesFromTo(self.0)    }
}
export class InsertEdgesFromTo {
    data: InsertEdgesQuery;

    each(): InsertEdgesEach {
self.0.each = true        return InsertEdgesEach(self.0)    }
    query(): InsertEdgesQuery {
        return self.0    }
    values(key_values: MultiValues): InsertEdgesValues {
self.0.values = QueryValues.Multi(Into.into(key_values).0)        return InsertEdgesValues(self.0)    }
    values_uniform(key_values: SingleValues): InsertEdgesValues {
self.0.values = QueryValues.Single(Into.into(key_values).0)        return InsertEdgesValues(self.0)    }
}
export class InsertEdgesIds {
    data: InsertEdgesQuery;

    from(ids: QueryIds): InsertEdgesFrom {
self.0.from = ids.into()        return InsertEdgesFrom(self.0)    }
}
export class InsertEdgesValues {
    data: InsertEdgesQuery;

    query(): InsertEdgesQuery {
        return self.0    }
}
export class InsertIndex {
    data: DbValue;

    query(): InsertIndexQuery {
        return InsertIndexQuery(self.0)    }
}
export class InsertNodes {
    data: InsertNodesQuery;

    aliases(names: QueryAliases): InsertNodesAliases {
self.0.aliases = Into.into(names).0        return InsertNodesAliases(self.0)    }
    count(num: number): InsertNodesCount {
self.0.count = num        return InsertNodesCount(self.0)    }
    ids(ids: QueryIds): InsertNodesIds {
self.0.ids = ids.into()        return InsertNodesIds(self.0)    }
    values(key_values: MultiValues): InsertNodesValues {
self.0.values = QueryValues.Multi(Into.into(key_values).0)        return InsertNodesValues(self.0)    }
}
export class InsertNodesAliases {
    data: InsertNodesQuery;

    query(): InsertNodesQuery {
        return self.0    }
    values(key_values: MultiValues): InsertNodesValues {
self.0.values = QueryValues.Multi(Into.into(key_values).0)        return InsertNodesValues(self.0)    }
    values_uniform(key_values: SingleValues): InsertNodesValues {
self.0.values = QueryValues.Single(Into.into(key_values).0)        return InsertNodesValues(self.0)    }
}
export class InsertNodesCount {
    data: InsertNodesQuery;

    query(): InsertNodesQuery {
        return self.0    }
    values_uniform(key_values: SingleValues): InsertNodesValues {
self.0.values = QueryValues.Single(Into.into(key_values).0)        return InsertNodesValues(self.0)    }
}
export class InsertNodesIds {
    data: InsertNodesQuery;

    aliases(names: QueryAliases): InsertNodesAliases {
self.0.aliases = Into.into(names).0        return InsertNodesAliases(self.0)    }
    count(num: number): InsertNodesCount {
self.0.count = num        return InsertNodesCount(self.0)    }
    values(key_values: MultiValues): InsertNodesValues {
self.0.values = QueryValues.Multi(Into.into(key_values).0)        return InsertNodesValues(self.0)    }
    values_uniform(key_values: SingleValues): InsertNodesValues {
self.0.values = QueryValues.Single(Into.into(key_values).0)        return InsertNodesValues(self.0)    }
}
export class InsertNodesValues {
    data: InsertNodesQuery;

    query(): InsertNodesQuery {
        return self.0    }
}
export class InsertValues {
    data: InsertValuesQuery;

    ids(ids: QueryIds): InsertValuesIds {
self.0.ids = ids.into()        return InsertValuesIds(self.0)    }
    search(): Search {
self.0.ids = QueryIds.Search(SearchQuery.new())        return Search(self.0)    }
}
export class InsertValuesIds {
    data: InsertValuesQuery;

    query(): InsertValuesQuery {
        return self.0    }
}
export class Remove {

    aliases(names: QueryAliases): RemoveAliases {
        return RemoveAliases(RemoveAliasesQuery(Into.into(names).0))    }
    ids(ids: QueryIds): RemoveIds {
        return RemoveIds(RemoveQuery(ids.into()))    }
    index(key: DbValue): RemoveIndex {
        return RemoveIndex(key.into())    }
    search(): Search {
        return Search(RemoveQuery(QueryIds.Search(SearchQuery.new())))    }
    values(keys: DbValues): RemoveValues {
        return RemoveValues(RemoveValuesQuery(new SelectValuesQuery {
    keys: Into.into(keys).0,
    ids: QueryIds.Ids([]),
}
))    }
}
export class RemoveAliases {
    data: RemoveAliasesQuery;

    query(): RemoveAliasesQuery {
        return self.0    }
}
export class RemoveIds {
    data: RemoveQuery;

    query(): RemoveQuery {
        return self.0    }
}
export class RemoveIndex {
    data: DbValue;

    query(): RemoveIndexQuery {
        return RemoveIndexQuery(self.0)    }
}
export class RemoveValues {
    data: RemoveValuesQuery;

    ids(ids: QueryIds): RemoveValuesIds {
self.0.0.ids = ids.into()        return RemoveValuesIds(self.0)    }
    search(): Search {
self.0.0.ids = QueryIds.Search(SearchQuery.new())        return Search(self.0)    }
}
export class RemoveValuesIds {
    data: RemoveValuesQuery;

    query(): RemoveValuesQuery {
        return self.0    }
}
export class Select {

    aliases(): SelectAliases {
        return SelectAliases(SelectAliasesQuery(QueryIds.Ids([])))    }
    edge_count(): SelectEdgeCount {
        return SelectEdgeCount(new SelectEdgeCountQuery {
    ids: QueryIds.Ids([]),
    from: true,
    to: true,
}
)    }
    edge_count_from(): SelectEdgeCount {
        return SelectEdgeCount(new SelectEdgeCountQuery {
    ids: QueryIds.Ids([]),
    from: true,
    to: false,
}
)    }
    edge_count_to(): SelectEdgeCount {
        return SelectEdgeCount(new SelectEdgeCountQuery {
    ids: QueryIds.Ids([]),
    from: false,
    to: true,
}
)    }
    elements(): SelectValues {
        return SelectValues(new SelectValuesQuery {
    keys: T.db_keys(),
    ids: QueryIds.Ids([]),
}
)    }
    ids(ids: QueryIds): SelectIds {
        return SelectIds(new SelectValuesQuery {
    keys: [],
    ids: ids.into(),
}
)    }
    indexes(): SelectIndexes {
        return new SelectIndexes;
    }
    keys(): SelectKeys {
        return SelectKeys(SelectKeysQuery(QueryIds.Ids([])))    }
    key_count(): SelectKeyCount {
        return SelectKeyCount(SelectKeyCountQuery(QueryIds.Ids([])))    }
    node_count(): SelectNodeCount {
        return new SelectNodeCount;
    }
    search(): Search {
        return Search(new SelectValuesQuery {
    keys: [],
    ids: QueryIds.Search(SearchQuery.new()),
}
)    }
    values(keys: DbValues): SelectValues {
        return SelectValues(new SelectValuesQuery {
    keys: Into.into(keys).0,
    ids: QueryIds.Ids([]),
}
)    }
}
export class SelectAliases {
    data: SelectAliasesQuery;

    ids(ids: QueryIds): SelectAliasesIds {
self.0.0 = ids.into()        return SelectAliasesIds(self.0)    }
    search(): Search {
self.0.0 = QueryIds.Search(SearchQuery.new())        return Search(self.0)    }
    query(): SelectAllAliasesQuery {
        return new SelectAllAliasesQuery;
    }
}
export class SelectAliasesIds {
    data: SelectAliasesQuery;

    query(): SelectAliasesQuery {
        return self.0    }
}
export class SelectEdgeCount {
    data: SelectEdgeCountQuery;

    ids(ids: QueryIds): SelectEdgeCountIds {
self.0.ids = ids.into()        return SelectEdgeCountIds(self.0)    }
    search(): Search {
self.0.ids = QueryIds.Search(SearchQuery.new())        return Search(self.0)    }
}
export class SelectEdgeCountIds {
    data: SelectEdgeCountQuery;

    query(): SelectEdgeCountQuery {
        return self.0    }
}
export class SelectIds {
    data: SelectValuesQuery;

    query(): SelectValuesQuery {
        return self.0    }
}
export class SelectIndexes {

    query(): SelectIndexesQuery {
        return new SelectIndexesQuery;
    }
}
export class SelectKeys {
    data: SelectKeysQuery;

    ids(ids: QueryIds): SelectKeysIds {
self.0.0 = ids.into()        return SelectKeysIds(self.0)    }
    search(): Search {
        return Search(SelectKeysQuery(QueryIds.Search(SearchQuery.new())))    }
}
export class SelectKeysIds {
    data: SelectKeysQuery;

    query(): SelectKeysQuery {
        return self.0    }
}
export class SelectKeyCount {
    data: SelectKeyCountQuery;

    ids(ids: QueryIds): SelectKeyCountIds {
self.0.0 = ids.into()        return SelectKeyCountIds(self.0)    }
    search(): Search {
        return Search(SelectKeyCountQuery(QueryIds.Search(SearchQuery.new())))    }
}
export class SelectKeyCountIds {
    data: SelectKeyCountQuery;

    query(): SelectKeyCountQuery {
        return self.0    }
}
export class SelectNodeCount {

    query(): SelectNodeCountQuery {
        return new SelectNodeCountQuery;
    }
}
export class SelectValues {
    data: SelectValuesQuery;

    ids(ids: QueryIds): SelectValuesIds {
self.0.ids = ids.into()        return SelectValuesIds(self.0)    }
    search(): Search {
self.0.ids = QueryIds.Search(SearchQuery.new())        return Search(self.0)    }
}
export class SelectValuesIds {
    data: SelectValuesQuery;

    query(): SelectValuesQuery {
        return self.0    }
}
export class Search {
    data: SearchQueryBuilderHelper;

    breadth_first(): SearchAlgorithm {
        self.0.search_mut().algorithm = BreadthFirst        return SearchAlgorithm(self.0)    }
    depth_first(): SearchAlgorithm {
        self.0.search_mut().algorithm = DepthFirst        return SearchAlgorithm(self.0)    }
    elements(): SearchTo {
        self.0.search_mut().algorithm = Elements        return SearchTo(self.0)    }
    index(key: DbValue): SearchIndex {
        return new SearchIndex {
    index: key.into(),
    query: self.0,
}
    }
    from(id: QueryId): SearchFrom {
self.0.search_mut().origin = id.into()        return SearchFrom(self.0)    }
    to(id: QueryId): SearchTo {
self.0.search_mut().destination = id.into()        return SearchTo(self.0)    }
}
export class SearchAlgorithm {
    data: SearchQueryBuilderHelper;

    from(id: QueryId): SearchFrom {
self.0.search_mut().origin = id.into()        return SearchFrom(self.0)    }
    to(id: QueryId): SearchTo {
self.0.search_mut().destination = id.into()        return SearchTo(self.0)    }
}
export class SearchFrom {
    data: SearchQueryBuilderHelper;

    limit(value: number): SelectLimit {
self.0.search_mut().limit = value        return SelectLimit(self.0)    }
    offset(value: number): SelectOffset {
self.0.search_mut().offset = value        return SelectOffset(self.0)    }
    order_by(keys: DbKeyOrders): SearchOrderBy {
self.0.search_mut().order_by = Into.into(keys).0        return SearchOrderBy(self.0)    }
    query(): SearchQueryBuilderHelper {
        return self.0    }
    to(id: QueryId): SearchTo {
self.0.search_mut().destination = id.into()        return SearchTo(self.0)    }
    where_(): Where {
        return Where.new(self.0)    }
}
export class SearchTo {
    data: SearchQueryBuilderHelper;

    limit(value: number): SelectLimit {
self.0.search_mut().limit = value        return SelectLimit(self.0)    }
    offset(value: number): SelectOffset {
self.0.search_mut().offset = value        return SelectOffset(self.0)    }
    order_by(keys: DbKeyOrders): SearchOrderBy {
self.0.search_mut().order_by = Into.into(keys).0        return SearchOrderBy(self.0)    }
    query(): SearchQueryBuilderHelper {
        return self.0    }
    where_(): Where {
        return Where.new(self.0)    }
}
export class SearchIndex {
    index: DbValue;
    query: SearchQueryBuilderHelper;

    value(value: DbValue): SearchIndexValue {
self.query.search_mut().algorithm = Indexself.query.search_mut().conditions.push(new QueryCondition {
    data: QueryConditionData.KeyValue(new KeyValueComparison {
    key: self.index,
    value: Comparison.Equal(value.into()),
}
),
    logic: And,
    modifier: None,
}
)        return SearchIndexValue(self.query)    }
}
export class SearchIndexValue {
    data: SearchQueryBuilderHelper;

    query(): SearchQueryBuilderHelper {
        return self.0    }
}
export class SearchOrderBy {
    data: SearchQueryBuilderHelper;

    limit(value: number): SelectLimit {
self.0.search_mut().limit = value        return SelectLimit(self.0)    }
    offset(value: number): SelectOffset {
self.0.search_mut().offset = value        return SelectOffset(self.0)    }
    query(): SearchQueryBuilderHelper {
        return self.0    }
    where_(): Where {
        return Where.new(self.0)    }
}
export class SelectLimit {
    data: SearchQueryBuilderHelper;

    query(): SearchQueryBuilderHelper {
        return self.0    }
    where_(): Where {
        return Where.new(self.0)    }
}
export class SelectOffset {
    data: SearchQueryBuilderHelper;

    limit(value: number): SelectLimit {
self.0.search_mut().limit = value        return SelectLimit(self.0)    }
    query(): SearchQueryBuilderHelper {
        return self.0    }
    where_(): Where {
        return Where.new(self.0)    }
}
export class Where {
    logic: QueryConditionLogic;
    modifier: QueryConditionModifier;
    conditions: QueryCondition[][];
    query: SearchQueryBuilderHelper;

    beyond(): Where {
self.modifier = Beyond        return self    }
    distance(comparison: CountComparison): WhereLogicOperator {
self.add_condition(new QueryCondition {
    logic: self.logic,
    modifier: self.modifier,
    data: QueryConditionData.Distance(comparison),
}
)        return WhereLogicOperator(self)    }
    edge(): WhereLogicOperator {
self.add_condition(new QueryCondition {
    logic: self.logic,
    modifier: self.modifier,
    data: Edge,
}
)        return WhereLogicOperator(self)    }
    edge_count(comparison: CountComparison): WhereLogicOperator {
self.add_condition(new QueryCondition {
    logic: self.logic,
    modifier: self.modifier,
    data: QueryConditionData.EdgeCount(comparison),
}
)        return WhereLogicOperator(self)    }
    edge_count_from(comparison: CountComparison): WhereLogicOperator {
self.add_condition(new QueryCondition {
    logic: self.logic,
    modifier: self.modifier,
    data: QueryConditionData.EdgeCountFrom(comparison),
}
)        return WhereLogicOperator(self)    }
    edge_count_to(comparison: CountComparison): WhereLogicOperator {
self.add_condition(new QueryCondition {
    logic: self.logic,
    modifier: self.modifier,
    data: QueryConditionData.EdgeCountTo(comparison),
}
)        return WhereLogicOperator(self)    }
    ids(ids: QueryIds): WhereLogicOperator {
self.add_condition(new QueryCondition {
    logic: self.logic,
    modifier: self.modifier,
    data: QueryConditionData.Ids(Into.into(ids).get_ids()),
}
)        return WhereLogicOperator(self)    }
    key(key: DbValue): WhereKey {
        return new WhereKey {
    key: key.into(),
    where_: self,
}
    }
    keys(keys: DbValues): WhereLogicOperator {
self.add_condition(new QueryCondition {
    logic: self.logic,
    modifier: self.modifier,
    data: QueryConditionData.Keys(Into.into(keys).0),
}
)        return WhereLogicOperator(self)    }
    node(): WhereLogicOperator {
self.add_condition(new QueryCondition {
    logic: self.logic,
    modifier: self.modifier,
    data: Node,
}
)        return WhereLogicOperator(self)    }
    not(): Where {
self.modifier = Not        return self    }
    not_beyond(): Where {
self.modifier = NotBeyond        return self    }
    where_(): Where {
self.add_condition(new QueryCondition {
    logic: self.logic,
    modifier: self.modifier,
    data: QueryConditionData.Where([]),
}
)self.conditions.push([])        return new Self {
    logic: And,
    modifier: None,
    conditions: self.conditions,
    query: self.query,
}
    }
    new(query: SearchQueryBuilderHelper): Where {
        return new Self {
    logic: And,
    modifier: None,
    conditions: [],
    query: query,
}
    }
    add_condition(condition: QueryCondition) {
self.conditions.last_mut().unwrap().push(condition)    }
    collapse_conditions(): number {
if (self.conditions.len() > 1) {
{
const last_conditions = self.conditions.pop().unwrap_or_default()const current_conditions = self.conditions.last_mut().unwrap()if (const Some(QueryCondition
{ logic : _, modifier : _, data : QueryConditionData :: Where(conditions), }) = current_conditions.last_mut()) {
{
!conditions = last_conditionsreturn true}
}}
}        return false    }
}
export class WhereKey {
    key: DbValue;
    where_: Where;

    value(comparison: Comparison): WhereLogicOperator {
const condition = new QueryCondition {
    logic: self.where_.logic,
    modifier: self.where_.modifier,
    data: QueryConditionData.KeyValue(new KeyValueComparison {
    key: self.key,
    value: comparison,
}
),
}
self.where_.add_condition(condition)        return WhereLogicOperator(self.where_)    }
}
export class WhereLogicOperator {
    data: Where;

    and(): Where {
        return new Where {
    logic: And,
    modifier: None,
    conditions: self.0.conditions,
    query: self.0.query,
}
    }
    end_where(): WhereLogicOperator {
self.0.collapse_conditions()        return WhereLogicOperator(self.0)    }
    or(): Where {
        return new Where {
    logic: Or,
    modifier: None,
    conditions: self.0.conditions,
    query: self.0.query,
}
    }
    query(): SearchQueryBuilderHelper {
while (self.0.collapse_conditions()) {
{
}
}
mem.swap(self.0.query.search_mut().conditions, self.0.conditions[0])        return self.0.query    }
}
export type SearchQueryBuilderHelper = {
    search: SearchQuery;
}
}
