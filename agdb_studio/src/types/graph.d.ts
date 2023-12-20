type GraphElementBase = {
    id: number;
    values: { [key: string]: string };
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
