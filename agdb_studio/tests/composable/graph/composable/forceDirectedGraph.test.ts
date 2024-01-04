import useForceDirectedGraph, {
    type ForceDirectedGraph,
} from "@/composables/graph/composable/forceDirectedGraph";
import { describe, it, expect, beforeEach } from "vitest";
import simpleData from "@/testData/simpleData.json" assert { type: "json" };

describe("useForceDirectedGraph 2D", () => {
    let graph: ForceDirectedGraph;
    const graphData = JSON.parse(JSON.stringify(simpleData));
    const results = {
        iterations: 63,
        x: -0.000735,
        y: 0.005062,
        z: 0,
    };

    beforeEach(() => {
        graph = useForceDirectedGraph({ is2d: true });
    });

    it("should load the graph correctly", () => {
        graph.loadGraph(graphData);

        // count nodes and edges
        expect(graph.getNodes().length).toEqual(3);
        expect(graph.getEdges().length).toEqual(3);

        // position of third node
        const node3 = graph.getNodes()[2];
        expect(node3.getCoordinates().x).toBeCloseTo(3.684244, 6);
        expect(node3.getCoordinates().y).toBeCloseTo(1.557673, 6);
        expect(node3.getCoordinates().z).toBe(0);
    });

    it("should simulate the graph correctly", () => {
        graph.loadGraph(graphData);
        graph.simulate();

        expect(graph.getIterations()).toBe(results.iterations);

        // position of third node
        const node3 = graph.getNodes()[2];
        expect(node3.getCoordinates().x).toBeCloseTo(results.x, 6);
        expect(node3.getCoordinates().y).toBeCloseTo(results.y, 6);
        expect(node3.getCoordinates().z).toBe(results.z);
    });

    it("should calculate the performance", () => {
        graph.loadGraph(graphData);
        graph.simulate();
        const performance = graph.getPerformance();

        expect(performance).toBeGreaterThan(-0.01);
    });

    it("should find a node correctly", () => {
        graph.loadGraph(graphData);
        const foundNode = graph.findNode(118);
        expect(foundNode?.getId()).toBe(118);
    });

    it("should process empty data correctly", () => {
        graph.loadGraph({
            result: 0,
            elements: [],
        });
        expect(graph.getNodes().length).toBe(0);
        expect(graph.getEdges().length).toBe(0);
    });

    it("should process data with only nodes correctly", () => {
        graph.loadGraph({
            result: 1,
            elements: [
                {
                    id: 1,
                    values: {},
                },
            ],
        });
        expect(graph.getNodes().length).toBe(1);
        expect(graph.getEdges().length).toBe(0);
    });
});

describe("useForceDirectedGraph 3D", () => {
    let graph: ForceDirectedGraph;
    const graphData = JSON.parse(JSON.stringify(simpleData));

    beforeEach(() => {
        graph = useForceDirectedGraph({ is2d: false });
    });

    it("should load the graph correctly", () => {
        graph.loadGraph(graphData);

        // position of third node
        const node3 = graph.getNodes()[2];
        expect(node3.getCoordinates().x).toBeCloseTo(3.684244, 6);
        expect(node3.getCoordinates().y).toBeCloseTo(5.241917, 6);
        expect(node3.getCoordinates().z).toBeCloseTo(1.557673, 6);
    });

    it("should simulate the graph correctly", () => {
        graph.loadGraph(graphData);
        graph.simulate();

        expect(graph.getIterations()).toBeGreaterThan(0);
    });
});
