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

const setSort = (
    tableName: string,
    columnKey: string,
    direction: "asc" | "desc",
): void => {
    const tableFilter = getTableFilter(tableName);
    tableFilter.sort.clear();
    tableFilter.sort.set(columnKey, direction);
};

export { getTableFilter, addFilter, removeFilter, setSort };
