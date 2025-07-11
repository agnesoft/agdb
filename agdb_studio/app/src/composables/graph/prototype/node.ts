export type Coordinates = {
  x: number;
  y: number;
  z: number;
};

type NodeOptions = {
  id: number;
  values: Record<string, string>;
  coordinates: Coordinates;
};

export interface Node {
  getId(): number;

  getCoordinates(): Coordinates;

  setCoordinates(x: number, y: number, z?: number): Node;

  getValues(): Map<string, string>;

  getValuesObject(): Record<string, string>;

  getValuesEntries(): IterableIterator<[string, string]>;

  getVelocity(): Coordinates;

  getVelocityLength(): number;

  setVelocity(vx: number, vy: number, vz: number): Node;

  move(damper: number): void;

  resetVelocity(): void;

  addVelocity(vx: number, vy: number, vz: number): Node;

  getX(): number;

  getY(): number;

  getZ(): number;

  dist(v: Node): number;
}

interface ThisNode {
  [key: string]: any;
}

export const Node = (function () {
  const symbol = Symbol("Node");

  const Node = function (this: ThisNode, options: NodeOptions): void {
    this.init(options);
  };

  Node.prototype.init = function (options: NodeOptions): void {
    this[symbol] = {
      id: options.id,
      values: new Map<string, string>(Object.entries(options.values)),
      x: options.coordinates.x,
      y: options.coordinates.y,
      z: options.coordinates.z,
      vx: 0,
      vy: 0,
      vz: 0,
    };
  };

  Node.prototype.getId = function (): number {
    return this[symbol].id;
  };

  Node.prototype.getCoordinates = function (): Coordinates {
    return { x: this[symbol].x, y: this[symbol].y, z: this[symbol].z };
  };

  Node.prototype.setCoordinates = function (
    _x: number,
    _y: number,
    _z: number,
  ): Node {
    this[symbol].x = _x;
    this[symbol].y = _y;
    this[symbol].z = _z;
    return this[symbol];
  };

  Node.prototype.getValues = function (): Map<string, string> {
    return this[symbol].values;
  };

  Node.prototype.getValuesObject = function (): Record<string, string> {
    const result: Record<string, string> = {};
    for (const [key, value] of this[symbol].values.entries()) {
      result[key] = value;
    }
    return result;
  };

  Node.prototype.getValuesEntries = function (): IterableIterator<
    [string, string]
  > {
    return this[symbol].values.entries();
  };

  Node.prototype.getVelocity = function (): Coordinates {
    return { x: this[symbol].vx, y: this[symbol].vy, z: this[symbol].vz };
  };

  Node.prototype.getVelocityLength = function (): number {
    return Math.sqrt(
      Math.pow(this[symbol].vx, 2) +
        Math.pow(this[symbol].vy, 2) +
        Math.pow(this[symbol].vz, 2),
    );
  };

  Node.prototype.setVelocity = function (
    _vx: number,
    _vy: number,
    _vz: number,
  ): Node {
    this[symbol].vx = _vx;
    this[symbol].vy = _vy;
    this[symbol].vz = _vz;
    return this;
  };

  Node.prototype.move = function (damper: number): void {
    this[symbol].x += this[symbol].vx * damper;
    this[symbol].y += this[symbol].vy * damper;
    this[symbol].z += this[symbol].vz * damper;
    this.resetVelocity();
  };

  Node.prototype.resetVelocity = function (): void {
    this[symbol].vx = 0;
    this[symbol].vy = 0;
    this[symbol].vz = 0;
  };

  Node.prototype.addVelocity = function (
    _vx: number,
    _vy: number,
    _vz: number,
  ): Node {
    this[symbol].vx += _vx;
    this[symbol].vy += _vy;
    this[symbol].vz += _vz;
    return this;
  };

  Node.prototype.getX = function (): number {
    return this[symbol].x;
  };

  Node.prototype.getY = function (): number {
    return this[symbol].y;
  };

  Node.prototype.getZ = function (): number {
    return this[symbol].z;
  };

  Node.prototype.dist = function (v: Node): number {
    return Math.sqrt(
      Math.pow(this[symbol].x - v.getX(), 2) +
        Math.pow(this[symbol].y - v.getY(), 2) +
        Math.pow(this[symbol].z - v.getZ(), 2),
    );
  };

  return Node;
})();
