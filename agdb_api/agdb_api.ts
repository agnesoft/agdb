// GENERATED CODE - DO NOT EDIT

// PREAMBLE

export class Option<T> {
  public value: T | null;

  constructor(value: T | null) {
    this.value = value;
  }
}

export function Some<T>(value: T): Option<T> {
  return new Option(value);
}

export function None<T>(): Option<T> {
  return new Option<T>(null);
}

export class Result<T, E> {
  public value: T | E;

  constructor(value: T | E) {
    this.value = value;
  }
}

export function Ok<T, E>(value: T): Result<T, E> {
  return new Result<T, E>(value);
}

export function Err<T, E>(error: E): Result<T, E> {
  return new Result<T, E>(error);
}

export class reqwest_Client {
  // This is a placeholder for the actual reqwest.Client type
}

// END OF PREAMBLE

export class DbElement {
  public id: DbId;
  public from: DbId;
  public to: DbId;
  public values: DbKeyValue[];

  constructor(id: DbId, from: DbId, to: DbId, values: DbKeyValue[]) {
    this.id = id;
    this.from = from;
    this.to = to;
    this.values = values;
  }
}

export type DbF64 = number;

export type DbId = number;

export type DbKeyOrder = { Asc: DbValue } | { Desc: DbValue };

export type DbKeyOrders = DbKeyOrder[];

export class DbKeyValue {
  public key: DbValue;
  public value: DbValue;

  constructor(key: DbValue, value: DbValue) {
    this.key = key;
    this.value = value;
  }
}

export type DbValue =
  | { Bytes: number[] }
  | { I64: number }
  | { U64: number }
  | { F64: DbF64 }
  | { String: string }
  | { VecI64: number[] }
  | { VecU64: number[] }
  | { VecF64: DbF64[] }
  | { VecString: string[] };

export type DbValues = DbValue[];

export type QueryType =
  | { InsertAlias: InsertAliasesQuery }
  | { InsertEdges: InsertEdgesQuery }
  | { InsertIndex: InsertIndexQuery }
  | { InsertNodes: InsertNodesQuery }
  | { InsertValues: InsertValuesQuery }
  | { Remove: RemoveQuery }
  | { RemoveAliases: RemoveAliasesQuery }
  | { RemoveIndex: RemoveIndexQuery }
  | { RemoveValues: RemoveValuesQuery }
  | { Search: SearchQuery }
  | { SelectAliases: SelectAliasesQuery }
  | { SelectAllAliases: SelectAllAliasesQuery }
  | { SelectEdgeCount: SelectEdgeCountQuery }
  | { SelectIndexes: SelectIndexesQuery }
  | { SelectKeys: SelectKeysQuery }
  | { SelectKeyCount: SelectKeyCountQuery }
  | { SelectNodeCount: SelectNodeCountQuery }
  | { SelectValues: SelectValuesQuery };

export type QueryAliases = string[];

export class InsertAliasesQuery {
  public ids: QueryIds;
  public aliases: string[];

  constructor(ids: QueryIds, aliases: string[]) {
    this.ids = ids;
    this.aliases = aliases;
  }
}

export class InsertEdgesQuery {
  public from: QueryIds;
  public to: QueryIds;
  public ids: QueryIds;
  public values: QueryValues;
  public each: boolean;

  constructor(
    from: QueryIds,
    to: QueryIds,
    ids: QueryIds,
    values: QueryValues,
    each: boolean,
  ) {
    this.from = from;
    this.to = to;
    this.ids = ids;
    this.values = values;
    this.each = each;
  }
}

export type InsertIndexQuery = DbValue;

export class InsertNodesQuery {
  public count: number;
  public values: QueryValues;
  public aliases: string[];
  public ids: QueryIds;

  constructor(
    count: number,
    values: QueryValues,
    aliases: string[],
    ids: QueryIds,
  ) {
    this.count = count;
    this.values = values;
    this.aliases = aliases;
    this.ids = ids;
  }
}

export class InsertValuesQuery {
  public ids: QueryIds;
  public values: QueryValues;

  constructor(ids: QueryIds, values: QueryValues) {
    this.ids = ids;
    this.values = values;
  }
}

export type Comparison =
  | { Equal: DbValue }
  | { GreaterThan: DbValue }
  | { GreaterThanOrEqual: DbValue }
  | { LessThan: DbValue }
  | { LessThanOrEqual: DbValue }
  | { NotEqual: DbValue }
  | { Contains: DbValue }
  | { StartsWith: DbValue }
  | { EndsWith: DbValue };

export type CountComparison =
  | { Equal: number }
  | { GreaterThan: number }
  | { GreaterThanOrEqual: number }
  | { LessThan: number }
  | { LessThanOrEqual: number }
  | { NotEqual: number };

export class KeyValueComparison {
  public key: DbValue;
  public value: Comparison;

  constructor(key: DbValue, value: Comparison) {
    this.key = key;
    this.value = value;
  }
}

export class QueryCondition {
  public logic: QueryConditionLogic;
  public modifier: QueryConditionModifier;
  public data: QueryConditionData;

  constructor(
    logic: QueryConditionLogic,
    modifier: QueryConditionModifier,
    data: QueryConditionData,
  ) {
    this.logic = logic;
    this.modifier = modifier;
    this.data = data;
  }
}

export type QueryConditionData =
  | { Distance: CountComparison }
  | { Edge: void }
  | { EdgeCount: CountComparison }
  | { EdgeCountFrom: CountComparison }
  | { EdgeCountTo: CountComparison }
  | { Ids: QueryId[] }
  | { KeyValue: KeyValueComparison }
  | { Keys: DbValue[] }
  | { Node: void }
  | { Where: QueryCondition[] };

export type QueryConditionLogic = { And: void } | { Or: void };

export type QueryConditionModifier =
  | { None: void }
  | { Beyond: void }
  | { Not: void }
  | { NotBeyond: void };

export type QueryId = { Id: DbId } | { Alias: string };

export type QueryIds = { Ids: QueryId[] } | { Search: SearchQuery };

export class QueryResult {
  public result: number;
  public elements: DbElement[];

  constructor(result: number, elements: DbElement[]) {
    this.result = result;
    this.elements = elements;
  }
}

export type QueryValues = { Single: DbKeyValue[] } | { Multi: DbKeyValue[][] };

export type SingleValues = DbKeyValue[];

export type MultiValues = DbKeyValue[][];

export type RemoveAliasesQuery = string[];

export type RemoveIndexQuery = DbValue;

export type RemoveQuery = QueryIds;

export type RemoveValuesQuery = SelectValuesQuery;

export class SearchQuery {
  public algorithm: SearchQueryAlgorithm;
  public origin: QueryId;
  public destination: QueryId;
  public limit: number;
  public offset: number;
  public order_by: DbKeyOrder[];
  public conditions: QueryCondition[];

  constructor(
    algorithm: SearchQueryAlgorithm,
    origin: QueryId,
    destination: QueryId,
    limit: number,
    offset: number,
    order_by: DbKeyOrder[],
    conditions: QueryCondition[],
  ) {
    this.algorithm = algorithm;
    this.origin = origin;
    this.destination = destination;
    this.limit = limit;
    this.offset = offset;
    this.order_by = order_by;
    this.conditions = conditions;
  }
}

export type SearchQueryAlgorithm =
  | { BreadthFirst: void }
  | { DepthFirst: void }
  | { Index: void }
  | { Elements: void };

export type SelectAliasesQuery = QueryIds;

export type SelectAllAliasesQuery = [];

export class SelectEdgeCountQuery {
  public ids: QueryIds;
  public from: boolean;
  public to: boolean;

  constructor(ids: QueryIds, from: boolean, to: boolean) {
    this.ids = ids;
    this.from = from;
    this.to = to;
  }
}

export type SelectIndexesQuery = [];

export type SelectKeyCountQuery = QueryIds;

export type SelectKeysQuery = QueryIds;

export type SelectNodeCountQuery = [];

export class SelectValuesQuery {
  public keys: DbValue[];
  public ids: QueryIds;

  constructor(keys: DbValue[], ids: QueryIds) {
    this.keys = keys;
    this.ids = ids;
  }
}

export type QueryBuilder = [];

export type Insert = [];

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

export type Remove = [];

export type RemoveAliases = RemoveAliasesQuery;

export type RemoveIds = RemoveQuery;

export type RemoveIndex = DbValue;

export type RemoveValues = RemoveValuesQuery;

export type RemoveValuesIds = RemoveValuesQuery;

export type Search<T extends SearchQueryBuilder> = T;

export type SearchAlgorithm<T extends SearchQueryBuilder> = T;

export type SearchFrom<T extends SearchQueryBuilder> = T;

export type SearchTo<T extends SearchQueryBuilder> = T;

export class SearchIndex<T extends SearchQueryBuilder> {
  public index: DbValue;
  public query: T;

  constructor(index: DbValue, query: T) {
    this.index = index;
    this.query = query;
  }
}

export type SearchIndexValue<T extends SearchQueryBuilder> = T;

export type SearchOrderBy<T extends SearchQueryBuilder> = T;

export type SelectLimit<T extends SearchQueryBuilder> = T;

export type SelectOffset<T extends SearchQueryBuilder> = T;

export type Select = [];

export type SelectAliases = SelectAliasesQuery;

export type SelectAliasesIds = SelectAliasesQuery;

export type SelectEdgeCount = SelectEdgeCountQuery;

export type SelectEdgeCountIds = SelectEdgeCountQuery;

export type SelectIds = SelectValuesQuery;

export type SelectIndexes = [];

export type SelectKeyCount = SelectKeyCountQuery;

export type SelectKeyCountIds = SelectKeyCountQuery;

export type SelectKeys = SelectKeysQuery;

export type SelectKeysIds = SelectKeysQuery;

export type SelectNodeCount = [];

export class SelectValues {
  public query: SelectValuesQuery;
  public element_id: Option<DbValue>;
  public limit: number;

  constructor(
    query: SelectValuesQuery,
    element_id: Option<DbValue>,
    limit: number,
  ) {
    this.query = query;
    this.element_id = element_id;
    this.limit = limit;
  }
}

export type SelectValuesIds = SelectValuesQuery;

export class Where<T extends SearchQueryBuilder> {
  public logic: QueryConditionLogic;
  public modifier: QueryConditionModifier;
  public conditions: QueryCondition[][];
  public query: T;

  constructor(
    logic: QueryConditionLogic,
    modifier: QueryConditionModifier,
    conditions: QueryCondition[][],
    query: T,
  ) {
    this.logic = logic;
    this.modifier = modifier;
    this.conditions = conditions;
    this.query = query;
  }
}

export class WhereKey<T extends SearchQueryBuilder> {
  public key: DbValue;
  public where_: Where<T>;

  constructor(key: DbValue, where_: Where<T>) {
    this.key = key;
    this.where_ = where_;
  }
}

export type WhereLogicOperator<T extends SearchQueryBuilder> = Where<T>;

export interface SearchQueryBuilder {}

export interface HttpClient {}

export interface AgdbApiClient {}

export class AgdbApiError {
  public status: number;
  public description: string;

  constructor(status: number, description: string) {
    this.status = status;
    this.description = description;
  }
}

export class ReqwestClient {
  public client: reqwest_Client;
  public user_agent: string;

  constructor(client: reqwest_Client, user_agent: string) {
    this.client = client;
    this.user_agent = user_agent;
  }
}

export class AgdbApi<T extends AgdbApiClient> {
  public client: T;
  public address: string;
  public base_url: string;
  public token: Option<string>;

  constructor(
    client: T,
    address: string,
    base_url: string,
    token: Option<string>,
  ) {
    this.client = client;
    this.address = address;
    this.base_url = base_url;
    this.token = token;
  }
}

export class AdminStatus {
  public uptime: number;
  public dbs: number;
  public users: number;
  public logged_in_users: number;
  public size: number;
  public memory: number;
  public log_level: LogLevelFilter;

  constructor(
    uptime: number,
    dbs: number,
    users: number,
    logged_in_users: number,
    size: number,
    memory: number,
    log_level: LogLevelFilter,
  ) {
    this.uptime = uptime;
    this.dbs = dbs;
    this.users = users;
    this.logged_in_users = logged_in_users;
    this.size = size;
    this.memory = memory;
    this.log_level = log_level;
  }
}

export class ChangePassword {
  public password: string;
  public new_password: string;

  constructor(password: string, new_password: string) {
    this.password = password;
    this.new_password = new_password;
  }
}

export class ClusterStatus {
  public address: string;
  public status: boolean;
  public leader: boolean;

  constructor(address: string, status: boolean, leader: boolean) {
    this.address = address;
    this.status = status;
    this.leader = leader;
  }
}

export type DbAudit = QueryAudit[];

export type DbKind = { Memory: void } | { Mapped: void } | { File: void };

export type DbResource =
  | { All: void }
  | { Db: void }
  | { Audit: void }
  | { Backup: void };

export class DbUser {
  public username: string;
  public role: DbUserRole;

  constructor(username: string, role: DbUserRole) {
    this.username = username;
    this.role = role;
  }
}

export type DbUserRole = { Admin: void } | { Write: void } | { Read: void };

export type LogLevelFilter =
  | { Off: void }
  | { Error: void }
  | { Warn: void }
  | { Info: void }
  | { Debug: void }
  | { Trace: void };

export class QueryAudit {
  public timestamp: number;
  public username: string;
  public query: QueryType;

  constructor(timestamp: number, username: string, query: QueryType) {
    this.timestamp = timestamp;
    this.username = username;
    this.query = query;
  }
}

export class ServerDatabase {
  public db: string;
  public owner: string;
  public db_type: DbKind;
  public role: DbUserRole;
  public size: number;
  public backup: number;
  public created: number;

  constructor(
    db: string,
    owner: string,
    db_type: DbKind,
    role: DbUserRole,
    size: number,
    backup: number,
    created: number,
  ) {
    this.db = db;
    this.owner = owner;
    this.db_type = db_type;
    this.role = role;
    this.size = size;
    this.backup = backup;
    this.created = created;
  }
}

export class UserCredentials {
  public password: string;

  constructor(password: string) {
    this.password = password;
  }
}

export class UserLogin {
  public username: string;
  public password: string;

  constructor(username: string, password: string) {
    this.username = username;
    this.password = password;
  }
}

export class UserStatus {
  public username: string;
  public login: boolean;
  public admin: boolean;
  public sessions: UserSession[];

  constructor(
    username: string,
    login: boolean,
    admin: boolean,
    sessions: UserSession[],
  ) {
    this.username = username;
    this.login = login;
    this.admin = admin;
    this.sessions = sessions;
  }
}

export class UserSession {
  public session: string;
  public agent: string;
  public created: number;
  public expires_at: number;

  constructor(
    session: string,
    agent: string,
    created: number,
    expires_at: number,
  ) {
    this.session = session;
    this.agent = agent;
    this.created = created;
    this.expires_at = expires_at;
  }
}

export class Duration {
  public secs: number;
  public nanos: number;

  constructor(secs: number, nanos: number) {
    this.secs = secs;
    this.nanos = nanos;
  }
}

export class AtomicU16 {
  public value: number;

  constructor(value: number) {
    this.value = value;
  }
}

export const ADMIN: string = undefined;

export const CONFIG_FILE: string = undefined;

export const SERVER_DATA_DIR: string = undefined;

export const HOST: string = undefined;

export const BINARY: string = undefined;

export const DEFAULT_PORT: number = undefined;

export const POLL_INTERVAL: number = undefined;

export const RETRY_TIMEOUT: Duration = undefined;

export const RETRY_ATTEMPS: number = undefined;

export const SHUTDOWN_RETRY_TIMEOUT: Duration = undefined;

export const SHUTDOWN_RETRY_ATTEMPTS: number = undefined;

export const TEST_TIMEOUT: number = undefined;

export const CLIENT_TIMEOUT: Duration = undefined;

export const PORT: AtomicU16 = undefined;

export const COUNTER: AtomicU16 = undefined;

export const SERVER: TestServerImpl = undefined;

export type TestServerProcess = [];

export function server_bin(): Result<PathBuf, TestError> {
  // TODO: implement
}

export function next_user_name(): string {
  // TODO: implement
}

export function next_db_name(): string {
  // TODO: implement
}

export function audit_file(
  data_dir: string,
  owner: string,
  db: string,
): string {
  // TODO: implement
}

export function backup_audit_file(
  data_dir: string,
  owner: string,
  db: string,
): string {
  // TODO: implement
}

export function audit_entries(path: string): Result<number, TestError> {
  // TODO: implement
}

export function wait_for_ready(
  api: AgdbApi<ReqwestClient>,
): Promise<Result<[], TestError>> {
  // TODO: implement
}

export class TestError {
  public description: string;

  constructor(description: string) {
    this.description = description;
  }
}

export class PathBuf {
  public inner: number[];

  constructor(inner: number[]) {
    this.inner = inner;
  }
}

export class TestServer {
  public dir: string;
  public data_dir: string;
  public api: AgdbApi<ReqwestClient>;
  public server: TestServerImpl;

  constructor(
    dir: string,
    data_dir: string,
    api: AgdbApi<ReqwestClient>,
    server: TestServerImpl,
  ) {
    this.dir = dir;
    this.data_dir = data_dir;
    this.api = api;
    this.server = server;
  }
}

export class TestServerImpl {
  public dir: string;
  public data_dir: string;
  public address: string;
  public process: Option<TestServerProcess>;

  constructor(
    dir: string,
    data_dir: string,
    address: string,
    process: Option<TestServerProcess>,
  ) {
    this.dir = dir;
    this.data_dir = data_dir;
    this.address = address;
    this.process = process;
  }
}

export const CLUSTER: TestServerImpl[] = undefined;

export class TestCluster {
  public cluster: TestServerImpl[];

  constructor(cluster: TestServerImpl[]) {
    this.cluster = cluster;
  }
}

export function wait_for_leader(
  api: AgdbApi<ReqwestClient>,
): Promise<Result<ClusterStatus[], TestError>> {
  // TODO: implement
}

export function create_cluster(
  nodes: number,
  tls: boolean,
): Promise<Result<TestServerImpl[], TestError>> {
  // TODO: implement
}

export function cluster_data_dir(address: string): string {
  // TODO: implement
}
