import ForceDirectedGraph from "@/composables/graph/class/forceDirectedGraph";
import { describe, it, expect, beforeEach } from "vitest";
import simpleData from "@/tests/data/simpleData.json" assert { type: "json" };
import Node from "@/composables/graph/class/node";

describe("ForceDirectedGraph 2D", () => {
  let graph: ForceDirectedGraph;
  const graphData = JSON.parse(JSON.stringify(simpleData));
  const results = {
    iterations: 63,
    x: -0.000735,
    y: 0.005062,
    z: 0,
  };

  beforeEach(() => {
    graph = new ForceDirectedGraph({ is2d: true });
  });

  it("should load the graph correctly", () => {
    graph.loadGraph(graphData);

    // count nodes and edges
    expect(graph.getNodes().length).toEqual(3);
    expect(graph.getEdges().length).toEqual(3);

    // position of third node
    const node3 = graph.getNodes()[2];
    expect(node3?.getCoordinates().x).toBeCloseTo(3.684244, 6);
    expect(node3?.getCoordinates().y).toBeCloseTo(1.557673, 6);
    expect(node3?.getCoordinates().z).toBe(0);
  });

  it("should simulate the graph correctly", () => {
    graph.loadGraph(graphData);
    graph.simulate();

    expect(graph.getIterations()).toBe(results.iterations);

    // position of third node
    const node3 = graph.getNodes()[2];
    expect(node3?.getCoordinates().x).toBeCloseTo(results.x, 6);
    expect(node3?.getCoordinates().y).toBeCloseTo(results.y, 6);
    expect(node3?.getCoordinates().z).toBe(results.z);
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
});

describe("ForceDirectedGraph undefined node handling", () => {
  it("skips undefined nodes in repulsion loop without throwing", () => {
    const graph = new ForceDirectedGraph({ is2d: true });

    // create two real nodes and insert an undefined hole between them
    const n1 = new Node({
      id: 1,
      values: {},
      coordinates: { x: 0, y: 0, z: 0 },
    });
    const n2 = new Node({
      id: 2,
      values: {},
      coordinates: { x: 10, y: 0, z: 0 },
    });

    // inject internal nodes array with an undefined entry to hit the guard
    (graph as any).nodes = [n1, undefined, n2];

    // Call the private method directly to exercise the branch
    expect(() => (graph as any).applyRepulsionForces()).not.toThrow();

    // Velocities for the defined nodes should have been updated
    expect(n1.getVelocityLength()).toBeGreaterThan(0);
    expect(n2.getVelocityLength()).toBeGreaterThan(0);
  });
});

describe("ForceDirectedGraph 3D", () => {
  let graph: ForceDirectedGraph;
  const graphData = JSON.parse(JSON.stringify(simpleData));

  beforeEach(() => {
    graph = new ForceDirectedGraph({ is2d: false });
  });

  it("should load the graph correctly", () => {
    graph.loadGraph(graphData);

    // position of third node
    const node3 = graph.getNodes()[2];
    expect(node3?.getCoordinates().x).toBeCloseTo(3.684244, 6);
    expect(node3?.getCoordinates().y).toBeCloseTo(5.241917, 6);
    expect(node3?.getCoordinates().z).toBeCloseTo(1.557673, 6);
  });

  it("should simulate the graph correctly", () => {
    graph.loadGraph(graphData);
    graph.simulate();

    expect(graph.getIterations()).toBeGreaterThan(0);
  });
});
