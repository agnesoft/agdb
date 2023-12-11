import { ref } from "vue";
import Line from "./line";
import Vector from "./vector";

export type Node = {
    index: number;
    pos: Vector;
    next?: Vector;
    connections: number[];
    connectionIndexes: number[];
    velocity?: Vector;
};

export type GraphEdge = {
    toNode: number;
    index: number;
};

export type GraphNode = {
    index: number;
    edges: GraphEdge[];
};

export type Graph = {
    nodes: GraphNode[];
};

const ATTRACTION_CONSTANT = 0.1;
const REPULSION_CONSTANT = 1000.0;
const SPRING_LENGTH = 100.0;
const DAMPER = 0.5;

export default function useForceDirectedGraph() {
    const graph = ref<Graph>();
    const nodes = ref<Node[]>([]);
    const angle = ref(0.1);

    const setGraph = (value: Graph): void => {
        graph.value = value;
    };

    const getNodes = (): Node[] => {
        return nodes.value;
    };

    const simulate = (): void => {
        load();

        let iterations = 0;

        while (applyForces() && iterations < 500) {
            iterations++;
        }
    };

    const nextPos = (): Vector => {
        angle.value += 0.1;
        const distance = 10.0 * angle.value;
        return new Vector([Math.cos(angle.value) * distance, Math.sin(angle.value) * distance]);
    };

    const load = (): void => {
        if (!graph.value) return;

        nodes.value = graph.value.nodes.map((node) => {
            const _node: Node = {
                index: node.index,
                pos: nextPos(),
                connections: [],
                connectionIndexes: [],
            };

            for (const edge of node.edges) {
                if (edge.toNode !== node.index) {
                    _node.connections.push(edge.toNode);
                    _node.connectionIndexes.push(edge.index);
                }
            }

            return _node;
        });
    };

    const attractionForce = (node1: Node, node2: Node): Vector => {
        const line = new Line(node1.pos, node2.pos);
        const distance = Math.max(line.getLength(), 1.0);
        const force = ATTRACTION_CONSTANT * Math.max(distance - SPRING_LENGTH, 0.0);
        const angle = line.getAngle();
        return new Vector([force, angle]);
    };

    const repulsionForce = (node1: Node, node2: Node): Vector => {
        const line = new Line(node1.pos, node2.pos);
        const distance = Math.max(line.getLength(), 1.0);
        const force = -REPULSION_CONSTANT / (distance * distance);
        const angle = line.getAngle();
        return new Vector([force, angle]);
    };

    // Return true if the simulation should continue, false otherwise
    const applyForces = (): boolean => {
        if (!graph.value) return false;
        let totalMovement = 0.0;

        for (const node of nodes.value) {
            const line = new Line(new Vector(), node.pos);
            const currentPosition = new Vector([line.getLength(), line.getAngle()]);
            const netForce: Vector = new Vector();

            node.velocity = new Vector();

            for (const connection of node.connections) {
                netForce.add(
                    attractionForce(node, nodes.value.find((n) => n.index === connection) as Node),
                );
            }

            for (const otherNode of nodes.value) {
                if (otherNode.index !== node.index) {
                    netForce.add(repulsionForce(node, otherNode));
                }
            }

            node.velocity.add(netForce).mult(DAMPER);
            node.next = currentPosition.add(node.velocity);
        }

        for (const node of nodes.value) {
            totalMovement += node.pos.dist(node.next as Vector);
            node.pos = node.next as Vector;
        }
        return totalMovement >= 10.0;
    };

    return { getNodes, setGraph, simulate };
}
