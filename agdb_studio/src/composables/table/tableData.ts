import type { TRow } from "./tableConfig";
import { getTable } from "./tableConfig";
import { getTableFilter } from "./tableFilter";

const setTableData = <T extends TRow>(tableName: string, data: T[]): void => {
    const table = getTable<TRow>(tableName);
    if (!table) {
        return;
    }
    table.data = new Map<string, T>();
    for (const rowIndex in data) {
        const rowData: TRow = {};
        for (const column of table.columns) {
            rowData[column.key] = data[rowIndex][column.key];
        }

        table.data.set(rowIndex, rowData);
    }
};

const addRow = <T extends TRow>(
    tableName: string,
    rowKey: string,
    row: T,
): void => {
    const table = getTable<T>(tableName);
    table?.data?.set(rowKey, row);
};

const removeRow = <T extends TRow>(tableName: string, rowKey: string): void => {
    const table = getTable<T>(tableName);
    table?.data?.delete(rowKey);
};

const clearTableData = <T extends TRow>(tableName: string): void => {
    const table = getTable<T>(tableName);
    table?.data?.clear();
};

const getRows = <T extends TRow>(name: string): [string, T][] => {
    const table = getTable<T>(name);
    if (!table) {
        return [];
    }
    const filter = getTableFilter(name);
    const filteredRows = Array.from(table.data ?? []).filter(([, row]) => {
        for (const [filterKey, filterValue] of filter.filters) {
            if (row[filterKey] !== filterValue) {
                return false;
            }
        }
        return true;
    });
    const sortedRows = filteredRows.sort((a, b) => {
        for (const [sortKey, sortOrder] of filter.sort) {
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
