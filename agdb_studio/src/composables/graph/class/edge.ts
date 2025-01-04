import Node from "./node";

export type EdgeOptions = {
    id: number;
    from?: Node;
    to?: Node;
    values: Record<string, string>;
};

export default class Edge {
    private id: number;
    private from: Node | undefined;
    private to: Node | undefined;
    private values: Map<string, string>;

    constructor({ id, from, to, values }: EdgeOptions) {
        this.from = from;
        this.to = to;
        this.values = new Map<string, string>(Object.entries(values));
        this.id = id;
    }

    public getId(): number {
        return this.id;
    }

    public getLength(): number {
        if (this.from === undefined || this.to === undefined) {
            return 0;
        }

        return this.from.dist(this.to);
    }

    public getDx(): number {
        if (this.from === undefined || this.to === undefined) {
            return 0;
        }

        return this.to.getX() - this.from.getX();
    }

    public getDy(): number {
        if (this.from === undefined || this.to === undefined) {
            return 0;
        }

        return this.to.getY() - this.from.getY();
    }

    public getDz(): number {
        if (this.from === undefined || this.to === undefined) {
            return 0;
        }

        return this.to.getZ() - this.from.getZ();
    }

    public getValues(): Map<string, string> {
        return this.values;
    }

    public getValuesObject(): Record<string, string> {
        const result: Record<string, string> = {};
        for (const [key, value] of this.values.entries()) {
            result[key] = value;
        }
        return result;
    }

    public getValuesEntries(): IterableIterator<[string, string]> {
        return this.values.entries();
    }

    public getFrom(): Node | undefined {
        return this.from;
    }

    public setFrom(from: Node | undefined): void {
        this.from = from;
    }

    public getTo(): Node | undefined {
        return this.to;
    }

    public setTo(to: Node | undefined): void {
        this.to = to;
    }
}
