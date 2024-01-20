export type Coordinates = {
    x: number;
    y: number;
    z: number;
};

export type NodeOptions = {
    id: number;
    values: { [key: string]: string };
    coordinates: Coordinates;
};

export type Node = {
    getId: () => number;
    getCoordinates: () => Coordinates;
    getValuesEntries: () => IterableIterator<[string, string]>;
    getVelocity: () => Coordinates;
    getVelocityLength: () => number;
    setCoordinates: (x: number, y: number, z: number) => void;
    setVelocity: (vx: number, vy: number, vz: number) => void;
    move: (damper: number) => void;
    resetVelocity: () => void;
    addVelocity: (vx: number, vy: number, vz: number) => void;
    getX: () => number;
    getY: () => number;
    getZ: () => number;
    dist: (v: Node) => number;
    getValues: () => Map<string, string>;
    getValuesObject: () => { [key: string]: string };
};

function useNode(options: NodeOptions): Node {
    const id = options.id;
    const values = new Map<string, string>(Object.entries(options.values));
    let x = options.coordinates.x;
    let y = options.coordinates.y;
    let z = options.coordinates.z;
    let vx = 0;
    let vy = 0;
    let vz = 0;

    const getId = (): number => {
        return id;
    };

    const getCoordinates = (): Coordinates => {
        return { x, y, z };
    };

    const getValues = (): Map<string, string> => {
        return values;
    };

    const getValuesObject = (): { [key: string]: string } => {
        const result: { [key: string]: string } = {};
        for (const [key, value] of values.entries()) {
            result[key] = value;
        }
        return result;
    };

    const getValuesEntries = (): IterableIterator<[string, string]> => {
        return values.entries();
    };

    const getVelocity = (): Coordinates => {
        return { x: vx, y: vy, z: vz };
    };

    const getVelocityLength = (): number => {
        return Math.sqrt(vx ** 2 + vy ** 2 + vz ** 2);
    };

    const setCoordinates = (_x: number, _y: number, _z: number = 0): void => {
        x = _x;
        y = _y;
        z = _z;
    };

    const setVelocity = (_vx: number, _vy: number, _vz: number): void => {
        vx = _vx;
        vy = _vy;
        vz = _vz;
    };

    const move = (damper: number): void => {
        x += vx * damper;
        y += vy * damper;
        z += vz * damper;
        resetVelocity();
    };

    const resetVelocity = (): void => {
        vx = 0;
        vy = 0;
        vz = 0;
    };

    const addVelocity = (_vx: number, _vy: number, _vz: number): void => {
        vx += _vx;
        vy += _vy;
        vz += _vz;
    };

    const getX = (): number => {
        return x;
    };

    const getY = (): number => {
        return y;
    };

    const getZ = (): number => {
        return z;
    };

    const dist = (v: Node): number => {
        return Math.sqrt(
            (x - v.getX()) ** 2 + (y - v.getY()) ** 2 + (z - v.getZ()) ** 2,
        );
    };

    return {
        getId,
        getCoordinates,
        getValuesEntries,
        getVelocity,
        getVelocityLength,
        setCoordinates,
        setVelocity,
        move,
        resetVelocity,
        addVelocity,
        getX,
        getY,
        getZ,
        dist,
        getValues,
        getValuesObject,
    };
}

export default useNode;
