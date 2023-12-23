import { Node, type Coordinates } from "./node";
import { Edge } from "./edge";

export type ForceDirectedGraphOptions = {
    is2d: boolean;
};

const ITERATION_COUNT = 500;
const ATTRACTION_CONSTANT = 0.1;
const REPULSION_CONSTANT = 1000.0;
const SPRING_LENGTH = 100.0;
const GRAVITY = 0.1;

export interface ForceDirectedGraph {
    loadGraph(graph: Graph): void;

    simulate(): void;

    getPerformance(): number;

    getIterations(): number;

    getNodes(): Node[];

    getEdges(): Edge[];

    nextPos(): Coordinates;

    findNode(id: number): Node | undefined;

    damperIncrease(): void;

    step(): boolean;

    applyForces(): void;

    moveNodes(): boolean;

    applyAttractionForces(): void;

    applyRepulsionForces(): void;

    applyGravity(): void;
}

export const ForceDirectedGraph = (function () {
    let nodes: Node[] = [];
    let edges: Edge[] = [];
    let is2d: boolean;

    let angle1 = 0.1;
    let angle2 = 0.1;

    let damper = 0.5;

    let startTimestamp = 0;
    let endTimestamp = 0;

    let iterations = 0;

    const ForceDirectedGraph = function (options: ForceDirectedGraphOptions) {
        is2d = options.is2d;
    };

    ForceDirectedGraph.prototype.loadGraph = function (graph: Graph): void {
        nodes = [];
        edges = [];
        angle1 = 0.1;
        angle2 = 0.1;
        for (const element of graph.elements) {
            if (element.id < 0) {
                // element is an edge
                const edge = element as GraphEdge;
                const from = this.findNode(edge.from);
                const to = this.findNode(edge.to);

                edges.push(
                    new (Edge as any)({ id: edge.id, from: from, to: to, values: edge.values }),
                );
            } else {
                // element is a node
                const node = element as GraphNode;
                nodes.push(
                    new (Node as any)({
                        id: node.id,
                        values: node.values,
                        coordinates: this.nextPos(),
                    }),
                );
            }
        }
    };

    ForceDirectedGraph.prototype.simulate = function (): void {
        startTimestamp = Date.now();
        iterations = 0;
        while (this.step() && iterations < ITERATION_COUNT) {
            iterations++;
        }
        endTimestamp = Date.now();
    };

    ForceDirectedGraph.prototype.getPerformance = function (): number {
        return endTimestamp - startTimestamp;
    };

    ForceDirectedGraph.prototype.getIterations = function (): number {
        return iterations;
    };

    ForceDirectedGraph.prototype.getNodes = function (): Node[] {
        return nodes;
    };

    ForceDirectedGraph.prototype.getEdges = function (): Edge[] {
        return edges;
    };

    ForceDirectedGraph.prototype.nextPos = function (): Coordinates {
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

    ForceDirectedGraph.prototype.findNode = function (id: number): Node | undefined {
        return nodes.find((node) => node.getId() === id);
    };

    ForceDirectedGraph.prototype.damperIncrease = function (): void {
        damper += 0.1;
    };

    ForceDirectedGraph.prototype.step = function (): boolean {
        this.applyForces();
        this.damperIncrease();
        return this.moveNodes();
    };

    ForceDirectedGraph.prototype.applyForces = function (): void {
        this.applyAttractionForces();
        this.applyRepulsionForces();
        this.applyGravity();
    };

    ForceDirectedGraph.prototype.moveNodes = function (): boolean {
        let totalMovement = 0.0;
        for (const node of nodes) {
            totalMovement += node.getVelocityLength();
            node.move(damper);
        }
        return totalMovement >= 10.0;
    };

    ForceDirectedGraph.prototype.applyAttractionForces = function (): void {
        for (const edge of edges) {
            const from = edge.getFrom();
            const to = edge.getTo();
            if (from === undefined || to === undefined) {
                continue;
            }
            const dx = edge.getDx();
            const dy = edge.getDy();
            const dz = edge.getDz();
            const distance = edge.getLength();
            const force = ATTRACTION_CONSTANT * Math.max(distance - SPRING_LENGTH, 0.0);

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

    ForceDirectedGraph.prototype.applyRepulsionForces = function (): void {
        for (let i = 0; i < nodes.length; i++) {
            const nodeA = nodes[i];
            for (let j = i + 1; j < nodes.length; j++) {
                const nodeB = nodes[j];
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

    ForceDirectedGraph.prototype.applyGravity = function (): void {
        for (const node of nodes) {
            node.addVelocity(
                -node.getX() * GRAVITY,
                -node.getY() * GRAVITY,
                -node.getZ() * GRAVITY,
            );
        }
    };

    return ForceDirectedGraph;
})();
