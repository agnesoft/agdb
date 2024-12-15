type TableFilter = {
    filters: Map<string, string>;
    sort: Map<string, "asc" | "desc">;
};

const tableFilters = new Map<string, TableFilter>();

const getTableFilter = (tableName: string): TableFilter => {
    let tableFilter = tableFilters.get(tableName);
    if (!tableFilter) {
        tableFilter = { filters: new Map(), sort: new Map() };
        tableFilters.set(tableName, tableFilter);
    }
    return tableFilter;
};

const setTableFilter = (tableName: string, tableFilter: TableFilter): void => {
    tableFilters.set(tableName, tableFilter);
};

const addFilter = (
    tableName: string,
    columnKey: string,
    value: string,
): void => {
    const tableFilter = getTableFilter(tableName);
    tableFilter.filters.set(columnKey, value);
};

const removeFilter = (tableName: string, filterKey: string): void => {
    const tableFilter = getTableFilter(tableName);
    tableFilter.filters.delete(filterKey);
};

const clearTableFilter = (tableName: string): void => {
    tableFilters.delete(tableName);
};

export {
    setTableFilter,
    getTableFilter,
    clearTableFilter,
    addFilter,
    removeFilter,
};
