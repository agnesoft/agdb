type GraphElementBase = {
  id: number;
  values: Record<string, string>;
};

type GraphNode = GraphElementBase;

type GraphEdge = GraphElementBase & {
  from: number;
  to: number;
};

type GraphElement = GraphNode | GraphEdge;

type Graph = {
  result: number;
  elements: GraphElement[];
};
