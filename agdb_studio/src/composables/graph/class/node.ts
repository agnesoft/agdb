export type Coordinates = {
    x: number;
    y: number;
    z: number;
};

type NodeOptions = {
    id: number;
    values: { [key: string]: string };
    coordinates: Coordinates;
};

export default class Node {
    private id: number;
    private values: Map<string, string>;

    private x: number;
    private y: number;
    private z: number;

    private vx: number = 0;
    private vy: number = 0;
    private vz: number = 0;

    constructor(options: NodeOptions) {
        this.id = options.id;
        this.x = options.coordinates.x;
        this.y = options.coordinates.y;
        this.z = options.coordinates.z;
        this.values = new Map<string, string>(Object.entries(options.values));
    }

    public getId(): number {
        return this.id;
    }

    public getCoordinates(): Coordinates {
        return { x: this.x, y: this.y, z: this.z };
    }

    public setCoordinates(x: number, y: number, z: number): Node {
        this.x = x;
        this.y = y;
        this.z = z;
        return this;
    }

    public getValues(): Map<string, string> {
        return this.values;
    }

    public getValuesObject(): { [key: string]: string } {
        const result: { [key: string]: string } = {};
        for (const [key, value] of this.values.entries()) {
            result[key] = value;
        }
        return result;
    }

    public getValuesEntries(): IterableIterator<[string, string]> {
        return this.values.entries();
    }

    public getVelocity(): Coordinates {
        return { x: this.vx, y: this.vy, z: this.vz };
    }

    public getVelocityLength(): number {
        return Math.sqrt(
            Math.pow(this.vx, 2) + Math.pow(this.vy, 2) + Math.pow(this.vz, 2),
        );
    }

    public setVelocity(vx: number, vy: number, vz: number): Node {
        this.vx = vx;
        this.vy = vy;
        this.vz = vz;
        return this;
    }

    public move(damper: number): void {
        this.x += this.vx * damper;
        this.y += this.vy * damper;
        this.z += this.vz * damper;
        this.resetVelocity();
    }

    public resetVelocity(): void {
        this.vx = 0;
        this.vy = 0;
        this.vz = 0;
    }

    public addVelocity(vx: number, vy: number, vz: number): Node {
        this.vx += vx;
        this.vy += vy;
        this.vz += vz;
        return this;
    }

    public getX(): number {
        return this.x;
    }

    public getY(): number {
        return this.y;
    }

    public getZ(): number {
        return this.z;
    }

    public dist(v: Node): number {
        return Math.sqrt(
            Math.pow(this.x - v.x, 2) +
                Math.pow(this.y - v.y, 2) +
                Math.pow(this.z - v.z, 2),
        );
    }
}
