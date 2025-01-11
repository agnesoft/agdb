import { ref } from "vue";
import type { Column, Table, TRow } from "./types";

const tables = ref<Map<Symbol | string, Table<TRow>>>(
    new Map<Symbol | string, Table<TRow>>(),
);

type AddTableProps<T extends TRow> = {
    name: Symbol | string;
    columns: Column<T>[];
    rowDetailsComponent?: AsyncComponent;
};

const addTable = ({
    name,
    columns,
    rowDetailsComponent,
}: AddTableProps<TRow>): void => {
    const columnMap = new Map<string, Column<TRow>>();
    columns.forEach((column) => {
        columnMap.set(column.key, column);
    });
    tables.value.set(name, {
        name,
        columns: columnMap,
        data: new Map(),
        rowDetailsComponent,
    });
};

const removeTable = (name: Symbol | string): void => {
    tables.value.delete(name);
};

const getTable = <T extends TRow>(
    name: Symbol | string,
): Table<T> | undefined => {
    return tables.value.get(name) as Table<T> | undefined;
};

const getTableColumns = <T extends TRow>(
    name: Symbol | string,
): Map<string, Column<T>> => {
    const table = getTable<T>(name);
    return table?.columns ?? new Map<string, Column<T>>();
};

const getTableColumnsArray = <T extends TRow>(
    name: Symbol | string,
): Column<T>[] => {
    const table = getTable<T>(name);
    return Array.from(table?.columns.values() ?? []);
};

const tableExists = (name: Symbol | string): boolean => {
    return tables.value.has(name);
};

const clearTables = (): void => {
    tables.value.clear();
};

export {
    getTable,
    addTable,
    removeTable,
    tableExists,
    clearTables,
    getTableColumns,
    getTableColumnsArray,
};
