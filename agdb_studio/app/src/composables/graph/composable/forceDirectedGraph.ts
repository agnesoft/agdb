import { shallowRef, triggerRef } from "vue";
import useNode, { type Coordinates, type Node } from "./node";
import useEdge, { type Edge } from "./edge";

export type ForceDirectedGraphOptions = {
  is2d: boolean;
};

const ITERATION_COUNT = 500;
const ATTRACTION_CONSTANT = 0.1;
const REPULSION_CONSTANT = 100000.0;
const SPRING_LENGTH = 100.0;
const GRAVITY = 0.1;
const DAMPER = 0.5;

export type ForceDirectedGraph = {
  loadGraph: (graph: Graph) => void;
  simulate: () => void;
  simulateAsync: () => Promise<void>;
  getPerformance: () => number;
  getIterations: () => number;
  getNodes: () => Node[];
  getEdges: () => Edge[];
  findNode: (id: number) => Node | undefined;
};

const useForceDirectedGraph = function (
  options: ForceDirectedGraphOptions,
): ForceDirectedGraph {
  const nodes = shallowRef<Node[]>([]);
  const edges = shallowRef<Edge[]>([]);
  const is2d: boolean = options.is2d;

  let angle1 = 0.1;
  let angle2 = 0.1;

  let startTimestamp = 0;
  let endTimestamp = 0;

  let iterations = 0;

  const loadGraph = (graph: Graph): void => {
    nodes.value = [];
    edges.value = [];
    angle1 = 0.1;
    angle2 = 0.1;
    for (const element of graph.elements) {
      if (element.id < 0) {
        // element is an edge
        const edge = element as GraphEdge;
        const from = findNode(edge.from);
        const to = findNode(edge.to);

        edges.value.push(
          useEdge({
            id: edge.id,
            from: from,
            to: to,
            values: edge.values,
          }),
        );
      } else {
        // element is a node
        const node = element as GraphNode;
        nodes.value.push(
          useNode({
            id: node.id,
            values: node.values,
            coordinates: nextPos(),
          }),
        );
      }
    }
    triggerRef(nodes);
    triggerRef(edges);
  };

  const normalizeNodes = (): void => {
    let minX = Infinity;
    let minY = Infinity;
    let minZ = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;
    let maxZ = -Infinity;

    for (const node of nodes.value) {
      const coordinates = node.getCoordinates();
      minX = Math.min(minX, coordinates.x);
      minY = Math.min(minY, coordinates.y);
      minZ = Math.min(minZ, coordinates.z);
      maxX = Math.max(maxX, coordinates.x);
      maxY = Math.max(maxY, coordinates.y);
      maxZ = Math.max(maxZ, coordinates.z);
    }

    const centerX = (minX + maxX) / 2.0;
    const centerY = (minY + maxY) / 2.0;
    const centerZ = (minZ + maxZ) / 2.0;

    const scaleX = 1.0 / (maxX - minX);
    const scaleY = 1.0 / (maxY - minY);
    const scaleZ = maxZ !== minZ ? 1.0 / (maxZ - minZ) : 0.0;

    for (const node of nodes.value) {
      const coordinates = node.getCoordinates();
      node.setCoordinates(
        (coordinates.x - centerX) * scaleX,
        (coordinates.y - centerY) * scaleY,
        (coordinates.z - centerZ) * scaleZ,
      );
    }
  };

  const simulate = (): void => {
    startTimestamp = Date.now();
    iterations = 0;
    while (step() && iterations < ITERATION_COUNT) {
      iterations++;
    }
    normalizeNodes();
    endTimestamp = Date.now();
    triggerRef(nodes);
    triggerRef(edges);
  };

  const simulateAsync = async (): Promise<void> => {
    startTimestamp = Date.now();
    iterations = 0;
    while (step() && iterations < ITERATION_COUNT) {
      iterations++;
      if (iterations % 50 === 0) {
        await new Promise((resolve) => setTimeout(resolve, 0));
      }
    }
    normalizeNodes();
    endTimestamp = Date.now();
    triggerRef(nodes);
    triggerRef(edges);
  };

  const getPerformance = (): number => {
    return endTimestamp - startTimestamp;
  };

  const getIterations = (): number => {
    return iterations;
  };

  const getNodes = (): Node[] => {
    return nodes.value;
  };

  const getEdges = (): Edge[] => {
    return edges.value;
  };

  const findNode = (id: number): Node | undefined => {
    return nodes.value.find((node) => node.getId() === id);
  };

  const nextPos = (): Coordinates => {
    angle1 += 0.1;
    angle2 += 0.1;
    const distance = 10.0 * angle1;
    if (is2d) {
      return {
        x: Math.cos(angle1) * distance,
        y: Math.sin(angle1) * distance,
        z: 0,
      };
    }

    return {
      x: Math.cos(angle1) * distance,
      y: Math.sin(angle1) * distance + Math.cos(angle2) * distance,
      z: Math.sin(angle2) * distance,
    };
  };

  const step = (): boolean => {
    applyForces();
    return moveNodes();
  };

  const applyForces = (): void => {
    applyAttractionForces();
    applyRepulsionForces();
    applyGravity();
  };

  const moveNodes = (): boolean => {
    let totalMovement = 0.0;
    for (const node of nodes.value) {
      totalMovement += node.getVelocityLength();
      node.move(DAMPER);
    }
    return totalMovement >= 10.0;
  };

  const applyAttractionForces = (): void => {
    for (const edge of edges.value) {
      const from = edge.getFrom();
      const to = edge.getTo();
      if (from === undefined || to === undefined) {
        continue;
      }
      const dx = edge.getDx();
      const dy = edge.getDy();
      const dz = edge.getDz();
      const distance = edge.getLength();
      const force =
        ATTRACTION_CONSTANT * Math.max(distance - SPRING_LENGTH, 0.0);

      // Apply force to both from and to nodes
      from.addVelocity(
        (force * dx) / distance,
        (force * dy) / distance,
        is2d ? 0 : (force * dz) / distance,
      );
      to.addVelocity(
        -(force * dx) / distance,
        -(force * dy) / distance,
        is2d ? 0 : -(force * dz) / distance,
      );
    }
  };

  const applyRepulsionForces = (): void => {
    for (let i = 0; i < nodes.value.length; i++) {
      const nodeA = nodes.value[i];
      for (let j = i + 1; j < nodes.value.length; j++) {
        const nodeB = nodes.value[j];
        /* v8 ignore next -- @preserve */
        if (!nodeA || !nodeB) continue;
        const dx = nodeB.getX() - nodeA.getX();
        const dy = nodeB.getY() - nodeA.getY();
        const dz = nodeB.getZ() - nodeA.getZ();
        const distance = nodeA.dist(nodeB);
        const force = REPULSION_CONSTANT / (distance * distance);

        // Apply force to both nodes
        nodeA.addVelocity(
          -(force * dx) / distance,
          -(force * dy) / distance,
          is2d ? 0 : -(force * dz) / distance,
        );
        nodeB.addVelocity(
          (force * dx) / distance,
          (force * dy) / distance,
          is2d ? 0 : (force * dz) / distance,
        );
      }
    }
  };

  const applyGravity = (): void => {
    for (const node of nodes.value) {
      node.addVelocity(
        -node.getX() * GRAVITY,
        -node.getY() * GRAVITY,
        -node.getZ() * GRAVITY,
      );
    }
  };

  return {
    loadGraph,
    simulate,
    simulateAsync,
    getPerformance,
    getIterations,
    getNodes,
    getEdges,
    findNode,
  };
};

export default useForceDirectedGraph;
