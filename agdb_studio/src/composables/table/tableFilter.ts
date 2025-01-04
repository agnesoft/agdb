type TableFilter = {
    filters: Map<string, string>;
    sort: Map<string, "asc" | "desc">;
};

const tableFilters = new Map<Symbol | string, TableFilter>();

const getTableFilter = (tableName: Symbol | string): TableFilter => {
    let tableFilter = tableFilters.get(tableName);
    if (!tableFilter) {
        tableFilter = { filters: new Map(), sort: new Map() };
        tableFilters.set(tableName, tableFilter);
    }
    return tableFilter;
};

const addFilter = (
    tableName: Symbol | string,
    columnKey: string,
    value: string,
): void => {
    const tableFilter = getTableFilter(tableName);
    tableFilter.filters.set(columnKey, value);
};

const removeFilter = (tableName: Symbol | string, filterKey: string): void => {
    const tableFilter = getTableFilter(tableName);
    tableFilter.filters.delete(filterKey);
};

const setSort = (
    tableName: Symbol | string,
    columnKey: string,
    direction: "asc" | "desc",
): void => {
    const tableFilter = getTableFilter(tableName);
    tableFilter.sort.clear();
    tableFilter.sort.set(columnKey, direction);
};

export { getTableFilter, addFilter, removeFilter, setSort };
