export type TCellType = string | number | Date | boolean;

export type TRow = Record<string, TCellType>;

export type Column<T extends TRow> = {
    key: string;
    title: string;
    cellClass?: string | ((row: T) => string);
    sortable?: boolean;
    filterable?: boolean;
    cellComponent?: AsyncComponent | ((row: T) => AsyncComponent);
    valueFormatter?: (value: TCellType) => TCellType;
    actions?: Action[];
};

export type Table<T extends TRow> = {
    name: Symbol | string;
    columns: Map<string, Column<T>>;
    data?: Map<string, T>;
    rowDetailsComponent?: AsyncComponent;
    uniqueKey?: string | ((row: T) => string);
};
