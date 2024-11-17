import type { TRow } from "./tableConfig";
import { useTableConfig } from "./tableConfig";

const { getTable } = useTableConfig();

// const setTableData = <T extends TRow>(tableName: string, data: T[]): void => {
//     const table = getTable<T>(tableName);
//     if (!table) {
//         return;
//     }
//     // table.data = new Map<string, T>(data.map((row) => [row.key, row]));
//     table.data = new Map<string, T>();
//     for (const rowIndex in data) {
//         table.data.set(rowIndex, data[rowIndex]);
//     }
// };

const addRow = <T extends TRow>(
    tableName: string,
    rowKey: string,
    row: T,
): void => {
    const table = getTable<T>(tableName);
    // table?.data?.set(row.key, row);
    table?.data?.set(rowKey, row);
    console.log("addRow", tableName, rowKey, row, table);
};

const removeRow = <T extends TRow>(tableName: string, rowKey: string): void => {
    const table = getTable<T>(tableName);
    table?.data?.delete(rowKey);
};

const clearTableData = <T extends TRow>(tableName: string): void => {
    const table = getTable<T>(tableName);
    table?.data?.clear();
};

// const getTableData = <T extends TRow>(tableName: string): T[] => {
//     const table = getTable<T>(tableName);
//     return table ? Array.from(table.data?.values() ?? []) : [];
// };

const getRows = <T extends TRow>(tableName: string): [string, T][] => {
    const table = getTable<T>(tableName);
    // todo apply sorting and filtering
    return table ? Array.from(table.data ?? []) : [];
};

// const getRowCellsSorted = <T extends TRow>(
//     name: string,
//     key: string,
// ): [string, string][] => {
//     const table = getTable<T>(name);
//     const row = table?.data?.get(key);
//     if (!row) {
//         return [];
//     }
//     return Object.entries(row);
// };

export const useTableData = () => {
    return {
        // setTableData,
        addRow,
        removeRow,
        clearTableData,
        // getTableData,
        getRows,
    };
};
