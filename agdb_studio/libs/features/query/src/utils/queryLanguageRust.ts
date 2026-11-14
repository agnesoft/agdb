import type { ChainCall, Literal, QueryLanguage } from "./queryChain";

// --- format ---------------------------------------------------------------

/** `where` is a Rust keyword, so the Rust builder calls it `where_`. */
const toRustMethod = (method: string): string =>
  method === "where" ? "where_" : method;

const STRING_ESCAPES: Record<string, string> = {
  "\\": "\\\\",
  '"': '\\"',
  "\n": "\\n",
  "\r": "\\r",
  "\t": "\\t",
};

const escapeString = (value: string): string => {
  const escaped = [...value].map((char) => {
    const known = STRING_ESCAPES[char];
    if (known) return known;
    const code = char.charCodeAt(0);
    return code < 0x20 || code === 0x7f
      ? `\\u{${code.toString(16).padStart(2, "0")}}`
      : char;
  });
  return `"${escaped.join("")}"`;
};

const formatNumber = (
  literal: Extract<Literal, { kind: "number" }>,
): string => {
  if (literal.type === "unsigned") return `${literal.value}_u64`;
  if (literal.type === "float" && !/[.eE]|inf|NaN/i.test(literal.value)) {
    return `${literal.value}.0`;
  }
  return literal.value;
};

const formatLiteral = (literal: Literal): string => {
  switch (literal.kind) {
    case "string":
      return escapeString(literal.value);
    case "number":
      return formatNumber(literal);
    case "boolean":
      return String(literal.value);
    case "list":
      return `[${formatItems(literal.items, literal.of)}]`;
    case "array":
      return `vec![${formatItems(literal.items)}]`;
    case "pair":
      return `(${formatLiteral(literal.key)}, ${formatLiteral(literal.value)}).into()`;
    case "order":
      return `DbKeyOrder::${literal.direction === "asc" ? "Asc" : "Desc"}(${formatLiteral(literal.key)}.into())`;
    /* v8 ignore next 2 -- @preserve: steps never produce comparisons */
    default:
      return `Comparison::Equal(${formatLiteral(literal.value)}.into())`;
  }
};

const itemType = (literal: Literal): string =>
  literal.kind === "number" ? `number:${literal.type}` : literal.kind;

/** Rust arrays need one item type, so mixed items are converted explicitly. */
const formatItems = (items: Literal[], of?: "ids" | "values"): string => {
  const mixed = !!of && new Set(items.map(itemType)).size > 1;
  const wrapper = of === "ids" ? "QueryId" : "DbValue";
  return items
    .map((item) =>
      mixed ? `${wrapper}::from(${formatLiteral(item)})` : formatLiteral(item),
    )
    .join(", ");
};

/** Formats the chain as a Rust expression, e.g. `QueryBuilder::select().ids(1).query()`. */
const format = (calls: ChainCall[]): string => {
  const chain = calls
    .map(
      (call) =>
        `${toRustMethod(call.method)}(${call.args.map(formatLiteral).join(", ")})`,
    )
    .join(".");
  const terminated = calls.at(-1)?.method === "query";
  return `QueryBuilder::${chain}${terminated ? "" : ".query()"}`;
};

// --- parse ----------------------------------------------------------------

type Token =
  | { kind: "ident"; value: string }
  | { kind: "string"; value: string }
  | { kind: "number"; value: string }
  | { kind: "punct"; value: string };

class ParseError extends Error {}

const fail = (message: string): never => {
  throw new ParseError(message);
};

const PUNCT = ["::", "(", ")", "[", "]", ",", ".", "&", "!", ";", "?", "-"];

// Sticky (`y`) patterns match at a position, so the text is never copied.
const SPACE = /\s+|\/\/[^\n]*|\/\*[\s\S]*?\*\//y;
const RAW_STRING = /r(#*)"([\s\S]*?)"\1/y;
const IDENT = /[A-Za-z_][A-Za-z0-9_]*/y;
// Hex digits never include the `i`/`u` of a suffix, e.g. `0xFF_u64`.
const HEX = /0x[0-9a-fA-F_]+([iu](8|16|32|64|128|size))?/y;
const NUMBER =
  /\d[\d_]*(\.\d[\d_]*)?([eE][+-]?\d+)?([iuf](8|16|32|64|128|size))?/y;
const UNICODE_ESCAPE = /u\{([0-9a-fA-F]{1,6})\}/y;

/** Reads tokens on demand, so code after the query never has to be valid. */
class Lexer {
  private pos = 0;
  private peeked: Token | null | undefined;

  constructor(private readonly text: string) {}

  peek(): Token | null {
    if (this.peeked === undefined) this.peeked = this.read();
    return this.peeked;
  }

  next(): Token | null {
    const token = this.peek();
    this.peeked = undefined;
    return token;
  }

  /** Matches a sticky regex at the current position without copying the text. */
  private match(pattern: RegExp, at = this.pos): RegExpExecArray | null {
    pattern.lastIndex = at;
    return pattern.exec(this.text);
  }

  private skipSpace(): void {
    for (;;) {
      const space = this.match(SPACE);
      if (!space) return;
      this.pos += space[0].length;
    }
  }

  private read(): Token | null {
    this.skipSpace();
    if (this.pos >= this.text.length) return null;
    const raw = this.match(RAW_STRING);
    if (raw) {
      this.pos += raw[0].length;
      return { kind: "string", value: raw[2] ?? "" };
    }
    const token = this.match(IDENT) ?? this.match(HEX) ?? this.match(NUMBER);
    if (token) {
      this.pos += token[0].length;
      return {
        kind: /^\d/.test(token[0]) ? "number" : "ident",
        value: token[0],
      };
    }
    if (this.text.startsWith('"', this.pos)) return this.readString();
    const punct = PUNCT.find((p) => this.text.startsWith(p, this.pos));
    if (punct) {
      this.pos += punct.length;
      return { kind: "punct", value: punct };
    }
    return fail(`Unexpected "${this.text.charAt(this.pos)}"`);
  }

  private readString(): Token {
    const escapes: Record<string, string> = {
      n: "\n",
      r: "\r",
      t: "\t",
      "\\": "\\",
      '"': '"',
      "'": "'",
      "0": "\0",
    };
    let value = "";
    let i = this.pos + 1;
    while (i < this.text.length) {
      const char = this.text.charAt(i);
      if (char === '"') {
        this.pos = i + 1;
        return { kind: "string", value };
      }
      if (char === "\\") {
        const next = this.text.charAt(i + 1);
        const unicode = this.match(UNICODE_ESCAPE, i + 1);
        if (unicode) {
          const code = parseInt(unicode[1] ?? "0", 16);
          if (code > 0x10ffff || (code >= 0xd800 && code <= 0xdfff)) {
            fail(`"\\${unicode[0]}" is not a valid character`);
          }
          value += String.fromCodePoint(code);
          i += 1 + unicode[0].length;
          continue;
        }
        const escaped = escapes[next];
        if (escaped !== undefined) {
          value += escaped;
          i += 2;
          continue;
        }
        if (next === "\n") {
          // A line continuation skips the newline and leading whitespace.
          i += 2;
          while (/\s/.test(this.text.charAt(i))) i++;
          continue;
        }
        return fail(`Unknown escape "\\${next}" in a string`);
      }
      value += char;
      i++;
    }
    return fail("Unterminated string");
  }
}

class Parser {
  constructor(private readonly lexer: Lexer) {}

  private isPunct(value: string): boolean {
    const token = this.lexer.peek();
    return token?.kind === "punct" && token.value === value;
  }

  private expect(value: string): void {
    if (!this.isPunct(value)) {
      const token = this.lexer.peek();
      fail(
        token
          ? `Expected "${value}", got "${token.value}"`
          : `Expected "${value}"`,
      );
    }
    this.lexer.next();
  }

  private ident(): string {
    const token = this.lexer.next();
    if (token?.kind !== "ident") {
      return fail(token ? `Unexpected "${token.value}"` : "Expected a method");
    }
    return token.value;
  }

  /** `select().ids(1)` or the short form `select.ids(1)`. */
  chain(): ChainCall[] {
    const calls: ChainCall[] = [];
    if (this.isPunct(".")) this.lexer.next();
    do {
      const name = this.ident();
      const method = name === "where_" ? "where" : name;
      const args = this.isPunct("(") ? this.callArgs() : [];
      calls.push({ method, args });
    } while (this.isPunct(".") && this.lexer.next());
    return calls;
  }

  done(): boolean {
    if (this.isPunct(";")) this.lexer.next();
    return this.lexer.peek() === null;
  }

  private callArgs(close = ")"): Literal[] {
    this.expect(close === ")" ? "(" : "[");
    const args: Literal[] = [];
    while (!this.isPunct(close)) {
      args.push(this.expression());
      if (!this.isPunct(",")) break;
      this.lexer.next();
    }
    this.expect(close);
    return args;
  }

  private expression(): Literal {
    const literal = this.primary();
    // Conversions do not change the value.
    while (this.isPunct(".")) {
      this.lexer.next();
      const method = this.ident();
      if (!["into", "to_string", "to_owned"].includes(method)) {
        fail(`${method}() is not supported in arguments`);
      }
      this.expect("(");
      this.expect(")");
    }
    return literal;
  }

  private primary(): Literal {
    if (this.isPunct("&")) {
      this.lexer.next();
      return this.primary();
    }
    if (this.isPunct("-")) {
      this.lexer.next();
      if (this.lexer.peek()?.kind !== "number") {
        return fail('"-" must be followed by a number');
      }
      const number = this.primary();
      /* v8 ignore next -- @preserve: a number token is always a number */
      if (number.kind !== "number") return number;
      if (number.type === "unsigned")
        fail("Unsigned numbers cannot be negative");
      return { ...number, value: `-${number.value}` };
    }
    if (this.isPunct("[")) {
      return { kind: "list", items: this.callArgs("]") };
    }
    if (this.isPunct("(")) {
      const items = this.callArgs();
      if (items.length === 1 && items[0]) return items[0];
      const [key, value] = items;
      if (items.length !== 2 || !key || !value) {
        return fail("Only (key, value) pairs are supported");
      }
      return { kind: "pair", key, value };
    }
    const token = this.lexer.next();
    if (!token) return fail("Unexpected end of the query");
    if (token.kind === "string") return { kind: "string", value: token.value };
    if (token.kind === "number") return parseNumber(token.value);
    if (token.kind === "punct") return fail(`Unexpected "${token.value}"`);
    if (token.value === "true" || token.value === "false") {
      return { kind: "boolean", value: token.value === "true" };
    }
    if (token.value === "vec" && this.isPunct("!")) {
      this.lexer.next();
      return {
        kind: "array",
        items: this.callArgs(this.isPunct("(") ? ")" : "]"),
      };
    }
    return this.pathCall(token.value);
  }

  /** e.g. `DbKeyOrder::Asc(k)`, `CountComparison::LessThan(3)`, `QueryId::from(1)`. */
  private pathCall(first: string): Literal {
    const path = [first];
    while (this.isPunct("::")) {
      this.lexer.next();
      path.push(this.ident());
    }
    const name = path.at(-1) ?? first;
    if (name === "QueryBuilder" || path.includes("QueryBuilder")) {
      fail("Nested queries are not supported");
    }
    if (!this.isPunct("("))
      return fail(`Unsupported value "${path.join("::")}"`);
    const args = this.callArgs();
    const [arg] = args;
    if (args.length !== 1 || !arg) {
      return fail(`${path.join("::")}() must have one argument`);
    }
    if (name === "Asc" || name === "Desc") {
      return {
        kind: "order",
        direction: name === "Asc" ? "asc" : "desc",
        key: arg,
      };
    }
    if (name === "from" || name === "new") return arg;
    if (/^[A-Z][A-Za-z]*$/.test(name) && path.at(-2) !== "DbValue") {
      // Comparison variants, e.g. LessThanOrEqual -> less_than_or_equal.
      const op = name.replace(
        /[A-Z]/g,
        (c, i: number) => `${i ? "_" : ""}${c.toLowerCase()}`,
      );
      return { kind: "comparison", op, value: arg };
    }
    return fail(`Unsupported value "${path.join("::")}(…)"`);
  }
}

const parseNumber = (raw: string): Literal => {
  const hex = /^0x([0-9a-fA-F_]+?)(?:_?([iu])(?:8|16|32|64|128|size))?$/.exec(
    raw,
  );
  if (hex) {
    const digits = (hex[1] ?? "").replace(/_/g, "");
    if (!digits) return fail(`"${raw}" is not a number`);
    const value = BigInt(`0x${digits}`).toString();
    return {
      kind: "number",
      value,
      type: hex[2] === "u" ? "unsigned" : hex[2] === "i" ? "signed" : "int",
    };
  }
  const [, digits = "", suffix] =
    /^([\d_.eE+-]+?)(?:_?([iuf])(?:8|16|32|64|128|size))?$/.exec(raw) ?? [];
  const value = digits.replace(/_/g, "");
  if (suffix === "u") return { kind: "number", value, type: "unsigned" };
  if (suffix === "i") return { kind: "number", value, type: "signed" };
  if (suffix === "f" || /[.eE]/.test(value)) {
    return { kind: "number", value, type: "float" };
  }
  return { kind: "number", value, type: "int" };
};

/**
 * Parses a Rust query, e.g. `QueryBuilder::select().ids(1).query()`.
 * Code around a `QueryBuilder::` chain is ignored, so a whole line such as
 * `db.exec(&QueryBuilder::select().ids(1).query())?;` can be pasted.
 */
const parse = (text: string): { calls: ChainCall[] } | { error: string } => {
  const start = text.indexOf("QueryBuilder::");
  const code = start >= 0 ? text.slice(start + "QueryBuilder::".length) : text;
  try {
    const parser = new Parser(new Lexer(code));
    const calls = parser.chain();
    // e.g. `.query().into()` for a batch of queries.
    while (
      calls.length > 1 &&
      calls.at(-2)?.method === "query" &&
      calls.at(-1)?.method === "into"
    ) {
      calls.pop();
    }
    if (start < 0 && !parser.done())
      return { error: "Unexpected text after the query" };
    return { calls };
  } catch (error) {
    // Anything unexpected is reported too, so a paste never crashes.
    /* v8 ignore next 4 -- @preserve: all known failures are ParseErrors */
    return {
      error:
        error instanceof ParseError
          ? error.message
          : "Could not read the query",
    };
  }
};

export const rustLanguage: QueryLanguage = { name: "Rust", format, parse };
