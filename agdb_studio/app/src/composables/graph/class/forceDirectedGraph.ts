import Node, { type Coordinates } from "./node";
import Edge from "./edge";

export type ForceDirectedGraphOptions = {
  is2d: boolean;
};

const ITERATION_COUNT = 500;
const ATTRACTION_CONSTANT = 0.1;
const REPULSION_CONSTANT = 100000.0;
const SPRING_LENGTH = 100.0;
const GRAVITY = 0.1;
const DAMPER = 0.5;

export default class ForceDirectedGraph {
  private nodes: Node[] = [];
  private edges: Edge[] = [];
  private is2d: boolean;

  private angle1 = 0.1;
  private angle2 = 0.1;

  private startTimestamp = 0;
  private endTimestamp = 0;

  private iterations = 0;

  constructor(options: ForceDirectedGraphOptions) {
    this.is2d = options.is2d;
  }

  public loadGraph(graph: Graph): void {
    this.nodes = [];
    this.edges = [];
    this.angle1 = 0.1;
    this.angle2 = 0.1;
    for (const element of graph.elements) {
      if (element.id < 0) {
        // element is an edge
        const edge = element as GraphEdge;
        const from = this.findNode(edge.from);
        const to = this.findNode(edge.to);

        this.edges.push(
          new Edge({
            id: edge.id,
            from: from,
            to: to,
            values: edge.values,
          }),
        );
      } else {
        // element is a node
        const node = element as GraphNode;
        this.nodes.push(
          new Node({
            id: node.id,
            values: node.values,
            coordinates: this.nextPos(),
          }),
        );
      }
    }
  }

  public simulate(): void {
    this.startTimestamp = Date.now();
    this.iterations = 0;
    while (this.step() && this.iterations < ITERATION_COUNT) {
      this.iterations++;
    }
    this.normalizeNodes();
    this.endTimestamp = Date.now();
  }

  public getPerformance(): number {
    return this.endTimestamp - this.startTimestamp;
  }

  public getIterations(): number {
    return this.iterations;
  }

  public getNodes(): Node[] {
    return this.nodes;
  }

  public getEdges(): Edge[] {
    return this.edges;
  }

  private normalizeNodes(): void {
    let minX = Infinity;
    let minY = Infinity;
    let minZ = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;
    let maxZ = -Infinity;

    for (const node of this.nodes) {
      const coordinates = node.getCoordinates();
      minX = Math.min(minX, coordinates.x);
      minY = Math.min(minY, coordinates.y);
      minZ = Math.min(minZ, coordinates.z);
      maxX = Math.max(maxX, coordinates.x);
      maxY = Math.max(maxY, coordinates.y);
      maxZ = Math.max(maxZ, coordinates.z);
    }

    const centerX = (minX + maxX) / 2.0;
    const centerY = (minY + maxY) / 2.0;
    const centerZ = (minZ + maxZ) / 2.0;

    const scaleX = 1.0 / (maxX - minX);
    const scaleY = 1.0 / (maxY - minY);
    const scaleZ = maxZ !== minZ ? 1.0 / (maxZ - minZ) : 0.0;

    for (const node of this.nodes) {
      const coordinates = node.getCoordinates();
      node.setCoordinates(
        (coordinates.x - centerX) * scaleX,
        (coordinates.y - centerY) * scaleY,
        (coordinates.z - centerZ) * scaleZ,
      );
    }
  }

  private nextPos(): Coordinates {
    this.angle1 += 0.1;
    this.angle2 += 0.1;
    const distance = 10.0 * this.angle1;
    if (this.is2d) {
      return {
        x: Math.cos(this.angle1) * distance,
        y: Math.sin(this.angle1) * distance,
        z: 0,
      };
    }

    return {
      x: Math.cos(this.angle1) * distance,
      y: Math.sin(this.angle1) * distance + Math.cos(this.angle2) * distance,
      z: Math.sin(this.angle2) * distance,
    };
  }

  public findNode(id: number): Node | undefined {
    return this.nodes.find((node) => node.getId() === id);
  }

  private step(): boolean {
    this.applyForces();
    return this.moveNodes();
  }

  private applyForces(): void {
    this.applyAttractionForces();
    this.applyRepulsionForces();
    this.applyGravity();
  }

  private moveNodes(): boolean {
    let totalMovement = 0.0;
    for (const node of this.nodes) {
      totalMovement += node.getVelocityLength();
      node.move(DAMPER);
    }
    return totalMovement >= 10.0;
  }

  private applyAttractionForces(): void {
    for (const edge of this.edges) {
      const from = edge.getFrom();
      const to = edge.getTo();
      if (from === undefined || to === undefined) {
        continue;
      }
      const dx = edge.getDx();
      const dy = edge.getDy();
      const dz = edge.getDz();
      const distance = edge.getLength();
      const force =
        ATTRACTION_CONSTANT * Math.max(distance - SPRING_LENGTH, 0.0);

      // Apply force to both from and to nodes
      from.addVelocity(
        (force * dx) / distance,
        (force * dy) / distance,
        this.is2d ? 0 : (force * dz) / distance,
      );
      to.addVelocity(
        -(force * dx) / distance,
        -(force * dy) / distance,
        this.is2d ? 0 : -(force * dz) / distance,
      );
    }
  }

  private applyRepulsionForces(): void {
    for (let i = 0; i < this.nodes.length; i++) {
      const nodeA = this.nodes[i];
      for (let j = i + 1; j < this.nodes.length; j++) {
        const nodeB = this.nodes[j];
        if (!nodeA || !nodeB) continue;
        const dx = nodeB.getX() - nodeA.getX();
        const dy = nodeB.getY() - nodeA.getY();
        const dz = nodeB.getZ() - nodeA.getZ();
        const distance = nodeA.dist(nodeB);
        const force = REPULSION_CONSTANT / (distance * distance);

        // Apply force to both nodes
        nodeA.addVelocity(
          -(force * dx) / distance,
          -(force * dy) / distance,
          this.is2d ? 0 : -(force * dz) / distance,
        );
        nodeB.addVelocity(
          (force * dx) / distance,
          (force * dy) / distance,
          this.is2d ? 0 : (force * dz) / distance,
        );
      }
    }
  }

  private applyGravity(): void {
    for (const node of this.nodes) {
      node.addVelocity(
        -node.getX() * GRAVITY,
        -node.getY() * GRAVITY,
        -node.getZ() * GRAVITY,
      );
    }
  }
}
