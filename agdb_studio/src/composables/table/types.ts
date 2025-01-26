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
    actions?: Action<T>[];
    type?: "string" | "number" | "boolean";

    // TODO: possibly add these later
    // width?: string;
    // minWidth?: string;
    // maxWidth?: string;
    // align?: "left" | "center" | "right";
    // headerAlign?: "left" | "center" | "right";
    // headerComponent?: AsyncComponent;
    // filterComponent?: AsyncComponent;
    // filter?: (row: T, filter: string) => boolean;
};

export type Table<T extends TRow> = {
    name: symbol | string;
    columns: Map<string, Column<T>>;
    data?: Map<string, T>;
    rowDetailsComponent?: AsyncComponent;
    uniqueKey?: string | ((row: T) => string);
};
