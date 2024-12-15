import { computed, ref } from "vue";

export type TRow = {
    [key: string]: string | number | Date | boolean;
};

export type Column<T extends TRow> = {
    key: string;
    title: string;
    cellClass?: string | ((row: T) => string);
    sortable?: boolean;
    filterable?: boolean;
    cellComponent?: string | ((row: T) => string);
};

export type Table<T extends TRow> = {
    name: string;
    columns: Column<T>[];
    data?: Map<string, T>;
};

const tables = ref<Map<string, Table<TRow>>>(new Map<string, Table<TRow>>());

const addTable = (name: string, columns: Column<TRow>[]): void => {
    tables.value.set(name, { name, columns, data: new Map() });
};

const removeTable = (name: string): void => {
    tables.value.delete(name);
};

const getTable = <T extends TRow>(name: string): Table<T> | undefined => {
    return tables.value.get(name) as Table<T> | undefined;
};

const getTableColumns = <T extends TRow>(
    name: string,
): Column<T>[] | undefined => {
    const table = getTable<T>(name);
    return table?.columns;
};

const tableExists = (name: string): boolean => {
    return tables.value.has(name);
};

const tableNames = computed(() => {
    return Array.from(tables.value.keys());
});

const clearTables = (): void => {
    tables.value.clear();
};

export {
    getTable,
    addTable,
    removeTable,
    tableExists,
    tableNames,
    clearTables,
    getTableColumns,
};
