export type TCellType = string | number | Date | boolean;

export type TRow = {
    [key: string]: TCellType;
};

export type Action<T extends TRow> = {
    label: string;
    action: (row: T) => void;
};

export type Column<T extends TRow> = {
    key: string;
    title: string;
    cellClass?: string | ((row: T) => string);
    sortable?: boolean;
    filterable?: boolean;
    cellComponent?: string | ((row: T) => string);
    valueFormatter?: (value: TCellType) => TCellType;
    actions?: Action<T>[];
};

export type Table<T extends TRow> = {
    name: string;
    columns: Map<string, Column<T>>;
    data?: Map<string, T>;
    uniqueKey?: string;
};
