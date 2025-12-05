import type { TRow } from "./types";
import { getTable } from "./tableConfig";
import { getTableFilter } from "./tableFilter";

const getRowKey = <T extends TRow>(
  row: T,
  uniqueKey?: string | ((row: T) => string),
): string => {
  const rowKey =
    typeof uniqueKey === "string" ? String(row[uniqueKey]) : uniqueKey?.(row);
  return rowKey ?? (Date.now() + Math.random()).toString();
};

const setTableData = <T extends TRow>(
  tableName: symbol | string,
  data: T[],
): void => {
  const table = getTable<TRow>(tableName);
  if (!table) {
    return;
  }
  table.data = new Map<string, T>();
  for (const rowIndex in data) {
    const rowData: TRow = {};
    const row = data[rowIndex];
    /* v8 ignore if -- @preserve */
    if (!row) {
      continue;
    }
    table.columns.forEach((column) => {
      /* v8 ignore next -- @preserve */
      rowData[column.key] = row[column.key] ?? "";
    });

    table.data.set(getRowKey(rowData, table.uniqueKey), rowData);
  }
};

const addRow = <T extends TRow>(tableName: symbol | string, row: T): void => {
  const table = getTable<T>(tableName);

  const rowKey = getRowKey(row, table?.uniqueKey);

  table?.data?.set(rowKey, row);
};

const removeRow = <T extends TRow>(
  tableName: symbol | string,
  rowKey: string,
): void => {
  const table = getTable<T>(tableName);
  table?.data?.delete(rowKey);
};

const clearTableData = <T extends TRow>(tableName: symbol | string): void => {
  const table = getTable<T>(tableName);
  table?.data?.clear();
};

const getRows = <T extends TRow>(tableName: symbol | string): [string, T][] => {
  const table = getTable<T>(tableName);
  if (!table?.data) {
    return [];
  }
  const filter = getTableFilter(tableName);
  const filteredRows = Array.from(table.data).filter(([, row]) => {
    if (filter.filters.size === 0) {
      return true;
    }
    for (const [filterKey, filterValue] of filter.filters) {
      if (row[filterKey] !== filterValue) {
        return false;
      }
    }
    return true;
  });
  const sortedRows = filteredRows.sort((a, b) => {
    for (const [sortKey, sortOrder] of filter.sort) {
      /* v8 ignore if -- @preserve */
      if (a[1][sortKey] === undefined || b[1][sortKey] === undefined) {
        continue;
      }
      if (a[1][sortKey] < b[1][sortKey]) {
        return sortOrder === "asc" ? -1 : 1;
      }
      if (a[1][sortKey] > b[1][sortKey]) {
        return sortOrder === "asc" ? 1 : -1;
      }
    }
    return 0;
  });
  return sortedRows;
};

export { setTableData, addRow, removeRow, clearTableData, getRows };
