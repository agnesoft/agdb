export type TCellType = string | number | Date | boolean;

export type TRow = Record<string, TCellType>;

export type Column<T extends TRow> = {
    key: string;
    title: string;
    cellClass?: string | ((row: T) => string);
    sortable?: boolean;
    filterable?: boolean;
    cellComponent?: string | ((row: T) => string);
    valueFormatter?: (value: TCellType) => TCellType;
    actions?: Action[];
};

export type Table<T extends TRow> = {
    name: string;
    columns: Map<string, Column<T>>;
    data?: Map<string, T>;
    uniqueKey?: string;
};
