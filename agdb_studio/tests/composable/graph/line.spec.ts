import Line from "@/composables/graph/line";
import Vector from "@/composables/graph/vector";
import { describe, it, expect, beforeEach } from "vitest";

describe("Line", () => {
    let line: Line;

    beforeEach(() => {
        const start = new Vector([0, 0]);
        const end = new Vector([3, 4]);
        line = new Line(start, end);
    });

    it("should calculate the length correctly", () => {
        expect(line.getLength()).toBe(5);
    });

    it("should calculate the angle correctly", () => {
        expect(line.getAngle()).toBeCloseTo(0.927295218, 6);
    });

    it("should calculate the midpoint correctly", () => {
        const midpoint = line.getMidPoint();
        expect(midpoint.x).toBe(1.5);
        expect(midpoint.y).toBe(2);
    });
});
