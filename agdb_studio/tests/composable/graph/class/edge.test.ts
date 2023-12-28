import Edge from "@/composables/graph/class/edge";
import Node from "@/composables/graph/class/node";
import { describe, it, expect, beforeEach } from "vitest";

describe("Edge", () => {
    let edge: Edge;
    let fromNode: Node;
    let toNode: Node;

    beforeEach(() => {
        fromNode = new Node({ id: 1, coordinates: { x: 0, y: 0, z: 0 }, values: {} });
        toNode = new Node({ id: 2, coordinates: { x: 3, y: 4, z: 5 }, values: {} });
        edge = new Edge({ id: 1, from: fromNode, to: toNode, values: { foo: "bar" } });
    });

    it("should return the correct id", () => {
        expect(edge.getId()).toBe(1);
    });

    it("should return the correct length", () => {
        expect(edge.getLength()).toBeCloseTo(7.071068, 6);
    });

    it("should return the correct dx", () => {
        expect(edge.getDx()).toBe(3);
    });

    it("should return the correct dy", () => {
        expect(edge.getDy()).toBe(4);
    });

    it("should return the correct dz", () => {
        expect(edge.getDz()).toBe(5);
    });

    it("should return the correct values", () => {
        expect(edge.getValues().get("foo")).toBe("bar");
    });

    it("should return the correct values object", () => {
        expect(edge.getValuesObject()).toEqual({ foo: "bar" });
    });

    it("should return the correct values", () => {
        expect(edge.getValuesEntries().next().value).toEqual(["foo", "bar"]);
    });

    it("should return the correct from node", () => {
        expect(edge.getFrom()).toBe(fromNode);
    });

    it("should set the from node correctly", () => {
        const newFromNode = new Node({ id: 3, coordinates: { x: 1, y: 2, z: 3 }, values: {} });
        edge.setFrom(newFromNode);
        expect(edge.getFrom()).toBe(newFromNode);
    });

    it("should return the correct to node", () => {
        expect(edge.getTo()).toBe(toNode);
    });

    it("should set the to node correctly", () => {
        const newToNode = new Node({ id: 4, coordinates: { x: 4, y: 5, z: 6 }, values: {} });
        edge.setTo(newToNode);
        expect(edge.getTo()).toBe(newToNode);
    });
});

describe("Edge with undefined from or to", () => {
    let node: Node;

    beforeEach(() => {
        node = new Node({ id: 2, coordinates: { x: 3, y: 4, z: 5 }, values: {} });
    });

    it("should return zero length if the from node is undefined", () => {
        const edge = new Edge({ id: 1, from: undefined, to: node, values: {} });
        expect(edge.getLength()).toBe(0);
    });

    it("should return zero length if the to node is undefined", () => {
        const edge = new Edge({ id: 1, from: node, to: undefined, values: {} });
        expect(edge.getLength()).toBe(0);
    });

    it("should return zero dx if the from node is undefined", () => {
        const edge = new Edge({ id: 1, from: undefined, to: node, values: {} });
        expect(edge.getDx()).toBe(0);
    });

    it("should return zero dx if the to node is undefined", () => {
        const edge = new Edge({ id: 1, from: node, to: undefined, values: {} });
        expect(edge.getDx()).toBe(0);
    });

    it("should return zero dy if the from node is undefined", () => {
        const edge = new Edge({ id: 1, from: undefined, to: node, values: {} });
        expect(edge.getDy()).toBe(0);
    });

    it("should return zero dy if the to node is undefined", () => {
        const edge = new Edge({ id: 1, from: node, to: undefined, values: {} });
        expect(edge.getDy()).toBe(0);
    });

    it("should return zero dz if the from node is undefined", () => {
        const edge = new Edge({ id: 1, from: undefined, to: node, values: {} });
        expect(edge.getDz()).toBe(0);
    });

    it("should return zero dz if the to node is undefined", () => {
        const edge = new Edge({ id: 1, from: node, to: undefined, values: {} });
        expect(edge.getDz()).toBe(0);
    });
});
