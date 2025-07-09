import { Node } from "./node";

export type EdgeOptions = {
  id: number;
  from?: Node;
  to?: Node;
  values: Record<string, string>;
};

export interface Edge {
  getLength(): number;

  getDx(): number;

  getDy(): number;

  getDz(): number;

  getValues(): Map<string, string>;

  getValuesObject(): Record<string, string>;

  getValuesEntries(): IterableIterator<[string, string]>;

  getFrom(): Node | undefined;

  setFrom(fromNode: Node): void;

  getTo(): Node | undefined;

  setTo(toNode: Node): void;

  getId(): number;
}

interface ThisEdge {
  [key: string]: any;
}

export const Edge = (function () {
  const symbol = Symbol("Edge");

  const Edge = function (this: ThisEdge, options: EdgeOptions) {
    this.init(options);
  };

  Edge.prototype.init = function (options: EdgeOptions): void {
    this[symbol] = {
      id: options.id,
      from: options.from,
      to: options.to,
      values: new Map<string, string>(Object.entries(options.values)),
    };
  };

  Edge.prototype.getLength = function (): number {
    const { from, to } = this[symbol];
    if (from === undefined || to === undefined) {
      return 0;
    }

    return from.dist(to);
  };

  Edge.prototype.getDx = function (): number {
    const { from, to } = this[symbol];
    if (from === undefined || to === undefined) {
      return 0;
    }

    return to.getX() - from.getX();
  };

  Edge.prototype.getDy = function (): number {
    const { from, to } = this[symbol];
    if (from === undefined || to === undefined) {
      return 0;
    }

    return to.getY() - from.getY();
  };

  Edge.prototype.getDz = function (): number {
    const { from, to } = this[symbol];
    if (from === undefined || to === undefined) {
      return 0;
    }

    return to.getZ() - from.getZ();
  };

  Edge.prototype.getValues = function (): Map<string, string> {
    return this[symbol].values;
  };

  Edge.prototype.getValuesObject = function (): Record<string, string> {
    const result: Record<string, string> = {};
    for (const [key, value] of this[symbol].values.entries()) {
      result[key] = value;
    }
    return result;
  };

  Edge.prototype.getValuesEntries = function (): IterableIterator<
    [string, string]
  > {
    const { values } = this[symbol];
    return values.entries();
  };

  Edge.prototype.getFrom = function (): Node | undefined {
    return this[symbol].from;
  };

  Edge.prototype.setFrom = function (fromNode: Node) {
    this[symbol].from = fromNode;
  };

  Edge.prototype.getTo = function (): Node | undefined {
    return this[symbol].to;
  };

  Edge.prototype.setTo = function (toNode: Node) {
    this[symbol].to = toNode;
  };

  Edge.prototype.getId = function (): number {
    return this[symbol].id;
  };

  return Edge;
})();
