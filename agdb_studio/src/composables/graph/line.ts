import Vector from "./vector";

export default class Line {
    public start: Vector;
    public end: Vector;

    constructor(start: Vector, end: Vector) {
        this.start = start;
        this.end = end;
    }

    public getLength(): number {
        return this.start.dist(this.end);
    }

    public getAngle(): number {
        return Math.atan2(this.end.y - this.start.y, this.end.x - this.start.x);
    }

    public getMidPoint(): Vector {
        return new Vector([(this.start.x + this.end.x) / 2, (this.start.y + this.end.y) / 2]);
    }
}
