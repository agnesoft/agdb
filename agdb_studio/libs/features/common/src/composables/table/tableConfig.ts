import { ref } from "vue";
import type { Column, Table, TRow } from "./types";

const tables = ref<Map<symbol | string, Table<TRow>>>(
  new Map<symbol | string, Table<TRow>>(),
);

type AddTableProps<T extends TRow> = {
  name: symbol | string;
  columns: Column<T>[];
  uniqueKey?: string | ((row: T) => string);
  fetchData: () => Promise<void>;
  onRowClick?: (row: T) => void;
};

const addTable = ({
  name,
  columns,
  uniqueKey,
  fetchData,
  onRowClick,
}: AddTableProps<TRow>): void => {
  const columnMap = new Map<string, Column<TRow>>();
  columns.forEach((column) => {
    columnMap.set(column.key, column);
  });
  tables.value.set(name, {
    name,
    columns: columnMap,
    data: new Map(),
    uniqueKey,
    fetchData,
    onRowClick,
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

  /* v8 ignore else -- @preserve */
  if (table) {
    await table.fetchData();
  }
};

const getTableRowClickHandler = <T extends TRow>(
  name: symbol | string,
): ((row: T) => void) | undefined => {
  const table = getTable<T>(name);
  return table?.onRowClick;
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
  getTableRowClickHandler,
};
