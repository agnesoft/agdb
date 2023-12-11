import Vector from "@/composables/graph/vector";
import { describe, it, expect, beforeEach } from "vitest";

describe("Vector", () => {
    let vector: Vector;

    beforeEach(() => {
        vector = new Vector([2, 3]);
    });

    it("constructor should set the current vector to the given values from array", () => {
        const result = new Vector([4, 5]);
        expect(result.x).toBe(4);
        expect(result.y).toBe(5);
    });

    it("constructor should set the current vector to the given values from object", () => {
        const result = new Vector({ x: 4, y: 5 });
        expect(result.x).toBe(4);
        expect(result.y).toBe(5);
    });

    it("constructor should set the current vector to zeros", () => {
        const result = new Vector();
        expect(result.x).toBe(0);
        expect(result.y).toBe(0);
    });

    it("add method should add the given vector to the current vector", () => {
        const v = new Vector([4, 5]);
        const result = vector.add(v);
        expect(result.x).toBe(6);
        expect(result.y).toBe(8);
    });

    it("sub method should subtract the given vector from the current vector", () => {
        const v = new Vector([4, 5]);
        const result = vector.sub(v);
        expect(result.x).toBe(-2);
        expect(result.y).toBe(-2);
    });

    it("mult method should multiply the current vector by the given number", () => {
        const result = vector.mult(2);
        expect(result.x).toBe(4);
        expect(result.y).toBe(6);
    });

    it("div method should divide the current vector by the given number", () => {
        const result = vector.div(2);
        expect(result.x).toBe(1);
        expect(result.y).toBe(1.5);
    });

    it("dist method should return the distance between the current vector and the given vector", () => {
        const v = new Vector([4, 6]);
        const result = vector.dist(v);
        expect(result).toBe(3.605551275463989);
    });

    it("copy method should return a copy of the current vector", () => {
        const result = vector.copy();
        expect(result.x).toBe(2);
        expect(result.y).toBe(3);
    });

    it("set method should set the current vector to the given values", () => {
        const result = vector.set(4, 5);
        expect(result.x).toBe(4);
        expect(result.y).toBe(5);
    });

    it("static add method should add the given vectors", () => {
        const v1 = new Vector([2, 3]);
        const v2 = new Vector([4, 5]);
        const result = Vector.add(v1, v2);
        expect(result.x).toBe(6);
        expect(result.y).toBe(8);
    });

    it("static sub method should subtract the given vectors", () => {
        const v1 = new Vector([2, 3]);
        const v2 = new Vector([4, 5]);
        const result = Vector.sub(v1, v2);
        expect(result.x).toBe(-2);
        expect(result.y).toBe(-2);
    });

    it("static mult method should multiply the given vector by the given number", () => {
        const v = new Vector([2, 3]);
        const result = Vector.mult(v, 2);
        expect(result.x).toBe(4);
        expect(result.y).toBe(6);
    });

    it("static div method should divide the given vector by the given number", () => {
        const v = new Vector([2, 3]);
        const result = Vector.div(v, 2);
        expect(result.x).toBe(1);
        expect(result.y).toBe(1.5);
    });
});
