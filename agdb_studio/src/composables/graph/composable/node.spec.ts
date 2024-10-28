import useNode, { type Node } from "@/composables/graph/composable/node";
import { describe, it, expect, beforeEach } from "vitest";

describe("useNode", () => {
    let node: Node;

    beforeEach(() => {
        node = useNode({
            id: 1,
            coordinates: { x: 0, y: 0, z: 0 },
            values: { foo: "bar" },
        });
    });

    it("should return the correct ID", () => {
        const id = 1;
        expect(node.getId()).toBe(id);
    });

    it("should return the correct coordinates", () => {
        const coordinates = { x: 0, y: 0, z: 0 };
        expect(node.getCoordinates()).toEqual(coordinates);
    });

    it("should set the coordinates correctly", () => {
        const x = 1;
        const y = 2;
        const z = 3;
        const node = useNode({
            id: 1,
            coordinates: { x: 0, y: 0, z: 0 },
            values: {},
        });
        node.setCoordinates(x, y, z);
        expect(node.getCoordinates()).toEqual({ x, y, z });
    });

    it("should return the correct values", () => {
        expect(node.getValues().get("foo")).toBe("bar");
    });

    it("should return the correct values object", () => {
        expect(node.getValuesObject()).toEqual({ foo: "bar" });
    });

    it("should return the correct values entries", () => {
        expect(node.getValuesEntries().next().value).toEqual(["foo", "bar"]);
    });

    it("should return the correct velocity", () => {
        const vx = 1;
        const vy = 2;
        const vz = 3;
        node.setVelocity(vx, vy, vz);
        expect(node.getVelocity()).toEqual({ x: vx, y: vy, z: vz });
    });

    it("should return the correct velocity length", () => {
        const vx = 1;
        const vy = 2;
        const vz = 3;
        node.setVelocity(vx, vy, vz);
        const velocityLength = Math.sqrt(vx ** 2 + vy ** 2 + vz ** 2);
        expect(node.getVelocityLength()).toBe(velocityLength);
    });

    it("should move the node correctly", () => {
        const damper = 0.5;
        const initialCoordinates = { x: 0, y: 0, z: 0 };
        const vx = 1;
        const vy = 2;
        const vz = 3;
        node.setVelocity(vx, vy, vz);
        node.move(damper);
        const finalCoordinates = {
            x: initialCoordinates.x + vx * damper,
            y: initialCoordinates.y + vy * damper,
            z: initialCoordinates.z + vz * damper,
        };
        expect(node.getCoordinates()).toEqual(finalCoordinates);
        expect(node.getVelocity()).toEqual({ x: 0, y: 0, z: 0 });
    });

    it("should reset the velocity correctly", () => {
        const vx = 1;
        const vy = 2;
        const vz = 3;
        node.setVelocity(vx, vy, vz);
        node.resetVelocity();
        expect(node.getVelocity()).toEqual({ x: 0, y: 0, z: 0 });
    });

    it("should add velocity correctly", () => {
        const initialVelocity = { x: 1, y: 2, z: 3 };
        const vx = 1;
        const vy = 2;
        const vz = 3;
        node.setVelocity(
            initialVelocity.x,
            initialVelocity.y,
            initialVelocity.z,
        );
        node.addVelocity(vx, vy, vz);
        node.addVelocity(vx, vy, vz);
        const finalVelocity = {
            x: initialVelocity.x + vx + vx,
            y: initialVelocity.y + vy + vy,
            z: initialVelocity.z + vz + vz,
        };
        expect(node.getVelocity()).toEqual(finalVelocity);
    });

    it("should return the correct X coordinate", () => {
        const x = 1;
        node.setCoordinates(x, 0, 0);
        expect(node.getX()).toBe(x);
    });

    it("should return the correct Y coordinate", () => {
        const y = 1;
        node.setCoordinates(0, y, 0);
        expect(node.getY()).toBe(y);
    });

    it("should return the correct Z coordinate", () => {
        const z = 1;
        node.setCoordinates(0, 0, z);
        expect(node.getZ()).toBe(z);
    });

    it("should calculate the correct distance between two nodes", () => {
        const node1 = useNode({
            id: 2,
            coordinates: { x: 1, y: 2, z: 3 },
            values: {},
        });
        const node2 = useNode({
            id: 3,
            coordinates: { x: 4, y: 5, z: 6 },
            values: {},
        });
        const distance = Math.sqrt((4 - 1) ** 2 + (5 - 2) ** 2 + (6 - 3) ** 2);
        expect(node1.dist(node2)).toBe(distance);
    });
});
