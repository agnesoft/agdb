import { ref } from "vue";
import type { Column, Table, TRow } from "./types";
import type { AsyncComponent } from "../../types/asyncComponents";

const tables = ref<Map<symbol | string, Table<TRow>>>(
  new Map<symbol | string, Table<TRow>>(),
);

type AddTableProps<T extends TRow> = {
  name: symbol | string;
  columns: Column<T>[];
  // rowDetailsComponent?: AsyncComponent;
  uniqueKey?: string | ((row: T) => string);
  fetchData: () => Promise<void>;
};

const addTable = ({
  name,
  columns,
  // rowDetailsComponent,
  uniqueKey,
  fetchData,
}: AddTableProps<TRow>): void => {
  const columnMap = new Map<string, Column<TRow>>();
  columns.forEach((column) => {
    columnMap.set(column.key, column);
  });
  tables.value.set(name, {
    name,
    columns: columnMap,
    data: new Map(),
    // rowDetailsComponent,
    uniqueKey,
    fetchData,
  });
};

const removeTable = (name: symbol | string): void => {
  tables.value.delete(name);
};

const getTable = <T extends TRow>(
  name: symbol | string,
): Table<T> | undefined => {
  return tables.value.get(name) as Table<T> | undefined;
};

const getTableColumns = <T extends TRow>(
  name: symbol | string,
): Map<string, Column<T>> => {
  const table = getTable<T>(name);
  return table?.columns ?? new Map<string, Column<T>>();
};

const getTableColumnsArray = <T extends TRow>(
  name: symbol | string,
): Column<T>[] => {
  const table = getTable<T>(name);
  return Array.from(table?.columns.values() ?? []);
};

const tableExists = (name: symbol | string): boolean => {
  return tables.value.has(name);
};

const clearTables = (): void => {
  tables.value.clear();
};

const fetchData = async (name: symbol | string | undefined): Promise<void> => {
  if (!name) {
    tables.value.forEach(async (table) => {
      await table.fetchData();
    });
    return;
  }
  const table = getTable(name);
  if (table) {
    await table.fetchData();
  }
};

export {
  getTable,
  addTable,
  removeTable,
  tableExists,
  clearTables,
  getTableColumns,
  getTableColumnsArray,
  fetchData,
};
