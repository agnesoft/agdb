type VectorOptions =
    | [number, number]
    | {
          x: number;
          y: number;
      };

export default class Vector {
    public x: number;
    public y: number;

    constructor(options: VectorOptions | undefined = undefined) {
        let values: [number, number] = [0, 0];
        if (Array.isArray(options)) {
            values = options;
        } else if (options !== undefined) {
            values = [options.x, options.y];
        }
        this.x = values[0];
        this.y = values[1];
    }

    public copy(): Vector {
        return new Vector({ x: this.x, y: this.y });
    }

    public set(x: number, y: number): Vector {
        this.x = x;
        this.y = y;
        return this;
    }

    public add(v: Vector): Vector {
        this.x += v.x;
        this.y += v.y;
        return this;
    }

    public sub(v: Vector): Vector {
        this.x -= v.x;
        this.y -= v.y;
        return this;
    }

    public mult(n: number): Vector {
        this.x *= n;
        this.y *= n;
        return this;
    }

    public div(n: number): Vector {
        this.x /= n;
        this.y /= n;
        return this;
    }

    public dist(v: Vector): number {
        return Math.sqrt(Math.pow(this.x - v.x, 2) + Math.pow(this.y - v.y, 2));
    }

    public static add(v1: Vector, v2: Vector): Vector {
        return new Vector({ x: v1.x + v2.x, y: v1.y + v2.y });
    }

    public static sub(v1: Vector, v2: Vector): Vector {
        return new Vector({ x: v1.x - v2.x, y: v1.y - v2.y });
    }

    public static mult(v: Vector, n: number): Vector {
        return new Vector({ x: v.x * n, y: v.y * n });
    }

    public static div(v: Vector, n: number): Vector {
        return new Vector({ x: v.x / n, y: v.y / n });
    }
}
