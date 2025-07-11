import { type Node } from "./node";

export type EdgeOptions = {
  id: number;
  from?: Node;
  to?: Node;
  values: Record<string, string>;
};

export type Edge = {
  getId: () => number;
  getLength: () => number;
  getDx: () => number;
  getDy: () => number;
  getDz: () => number;
  getValuesEntries: () => IterableIterator<[string, string]>;
  getFrom: () => Node | undefined;
  setFrom: (fromNode: Node) => void;
  getTo: () => Node | undefined;
  setTo: (toNode: Node) => void;
  getValues: () => Map<string, string>;
  getValuesObject: () => Record<string, string>;
};

const useEdge = function (options: EdgeOptions): Edge {
  let from = options.from;
  let to = options.to;
  const values = new Map<string, string>(Object.entries(options.values));
  const id = options.id;

  const getId = (): number => {
    return id;
  };

  const getLength = (): number => {
    if (from === undefined || to === undefined) {
      return 0;
    }

    return from.dist(to);
  };

  const getDx = (): number => {
    if (from === undefined || to === undefined) {
      return 0;
    }

    return to.getX() - from.getX();
  };

  const getDy = (): number => {
    if (from === undefined || to === undefined) {
      return 0;
    }

    return to.getY() - from.getY();
  };

  const getDz = (): number => {
    if (from === undefined || to === undefined) {
      return 0;
    }

    return to.getZ() - from.getZ();
  };

  const getValues = (): Map<string, string> => {
    return values;
  };

  const getValuesObject = (): Record<string, string> => {
    const result: Record<string, string> = {};
    for (const [key, value] of values.entries()) {
      result[key] = value;
    }
    return result;
  };

  const getValuesEntries = (): IterableIterator<[string, string]> => {
    return values.entries();
  };

  const getFrom = (): Node | undefined => {
    return from;
  };

  const setFrom = (_from: Node | undefined): void => {
    from = _from;
  };

  const getTo = (): Node | undefined => {
    return to;
  };

  const setTo = (_to: Node | undefined): void => {
    to = _to;
  };

  return {
    getId,
    getLength,
    getDx,
    getDy,
    getDz,
    getValuesEntries,
    getFrom,
    setFrom,
    getTo,
    setTo,
    getValues,
    getValuesObject,
  };
};

export default useEdge;
