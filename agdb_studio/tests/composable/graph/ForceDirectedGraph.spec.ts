import useForceDirectedGraph, { type Graph } from "@/composables/graph/ForceDirectedGraph";
import { describe, it, expect } from "vitest";

describe("useForceDirectedGraph", () => {
    it("should simulate the force-directed graph", () => {
        const { setGraph, simulate, getNodes } = useForceDirectedGraph();

        const graph: Graph = {
            nodes: [
                {
                    index: 0,
                    edges: [
                        {
                            index: 0,
                            toNode: 1,
                        },
                    ],
                },
                {
                    index: 1,
                    edges: [
                        {
                            index: 0,
                            toNode: 0,
                        },
                    ],
                },
                {
                    index: 2,
                    edges: [],
                },
                {
                    index: 3,
                    edges: [
                        {
                            index: 0,
                            toNode: 0,
                        },
                        {
                            index: 1,
                            toNode: 1,
                        },
                    ],
                },
            ],
        };

        setGraph(graph);
        simulate();

        const nodes = getNodes();

        expect(nodes.length).to.equal(4);
    });

    it("should not fail if graph was not set ", () => {
        const { simulate, getNodes } = useForceDirectedGraph();

        simulate();
        expect(getNodes().length).to.equal(0);
    });
});
