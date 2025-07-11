import Node from "@/composables/graph/class/node";
import { describe, it, expect, beforeEach } from "vitest";

describe("Node", () => {
  let node: Node;

  beforeEach(() => {
    node = new Node({
      id: 1,
      coordinates: { x: 0, y: 0, z: 0 },
      values: { foo: "bar" },
    });
  });

  it("should set the velocity correctly", () => {
    const vx = 1;
    const vy = 2;
    const vz = 3;
    node.setVelocity(vx, vy, vz);
    expect(node.getVelocity()).toEqual({ x: vx, y: vy, z: vz });
  });

  it("should get the correct ID", () => {
    const id = 1;
    expect(node.getId()).toBe(id);
  });

  it("should get the correct coordinates", () => {
    const coordinates = { x: 0, y: 0, z: 0 };
    expect(node.getCoordinates()).toEqual(coordinates);
  });

  it("should set the coordinates correctly", () => {
    const x = 1;
    const y = 2;
    const z = 3;
    node.setCoordinates(x, y, z);
    expect(node.getCoordinates()).toEqual({ x, y, z });
  });

  it("should get the correct values", () => {
    expect(node.getValues().get("foo")).toBe("bar");
  });

  it("should get the correct values object", () => {
    expect(node.getValuesObject()).toEqual({ foo: "bar" });
  });

  it("should get the correct values entries", () => {
    expect(node.getValuesEntries().next().value).toEqual(["foo", "bar"]);
  });

  it("should get the correct velocity length", () => {
    const vx = 1;
    const vy = 2;
    const vz = 3;
    node.setVelocity(vx, vy, vz);
    const velocityLength = Math.sqrt(vx ** 2 + vy ** 2 + vz ** 2);
    expect(node.getVelocityLength()).toBe(velocityLength);
  });

  it("should move the node correctly", () => {
    const damper = 0.5;
    const initialCoordinates = node.getCoordinates();
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
    const initialVelocity = node.getVelocity();
    const vx = 1;
    const vy = 2;
    const vz = 3;
    node.addVelocity(vx, vy, vz);
    node.addVelocity(vx, vy, vz);
    const finalVelocity = {
      x: initialVelocity.x + vx + vx,
      y: initialVelocity.y + vy + vy,
      z: initialVelocity.z + vz + vz,
    };
    expect(node.getVelocity()).toEqual(finalVelocity);
  });

  it("should get the correct X coordinate", () => {
    const x = 1;
    node.setCoordinates(x, 0, 0);
    expect(node.getX()).toBe(x);
  });

  it("should get the correct Y coordinate", () => {
    const y = 1;
    node.setCoordinates(0, y, 0);
    expect(node.getY()).toBe(y);
  });

  it("should get the correct Z coordinate", () => {
    const z = 1;
    node.setCoordinates(0, 0, z);
    expect(node.getZ()).toBe(z);
  });

  it("should calculate the correct distance between two nodes", () => {
    const node1 = new Node({
      id: 2,
      coordinates: { x: 1, y: 2, z: 3 },
      values: {},
    });
    const node2 = new Node({
      id: 3,
      coordinates: { x: 4, y: 5, z: 6 },
      values: {},
    });
    const distance = Math.sqrt((4 - 1) ** 2 + (5 - 2) ** 2 + (6 - 3) ** 2);
    expect(node1.dist(node2)).toBe(distance);
  });
});
