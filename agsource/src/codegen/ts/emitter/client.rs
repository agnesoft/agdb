//! [`Emitter::client`]: the TS client of an API crate.

use super::Emitter;
use super::Ts;
use crate::codegen::lowered::*;
use std::fmt::Write;

impl Emitter<'_> {
    /// The [client](crate::SourceCrate::lower_client) module: the consts,
    /// the traits as the types the target implements (an abstract class
    /// when they provide methods, the intersection of the supertraits or
    /// an empty record when they have none), the methods of the data types as
    /// functions and the builders as exported classes whose generics the
    /// traits bound.
    pub fn client(mut self) -> Ts {
        let m = self.m;
        self.bounded = true;

        for (name, value) in &m.consts {
            self.current = name.clone();
            let ty = self.ty(&value.ty);
            let value = self.expr(value);
            self.line(0, &format!("export const {name}: {ty} = {value};\n"));
        }

        for (name, sigs) in &m.traits {
            self.trait_type(name, sigs);
        }

        self.data_methods();

        for b in &m.builders {
            self.class(&m.classes[b], true);
        }

        let mut head = String::new();

        for name in self.schemas.borrow().iter() {
            let _ = writeln!(head, "type {name} = {}.{name};", self.config.schemas);
        }

        head.extend(self.helpers.iter().copied());

        if !head.is_empty() {
            head += "\n";
        }

        Ts {
            code: format!("{}\n\n{}\n\n", self.config.header, self.config.import)
                + &head
                + &self.out,
            notes: self.notes,
            unsupported: self.unsupported,
            values: Vec::new(),
        }
    }

    fn trait_type(&mut self, name: &str, sigs: &[Func]) {
        let supers = self.m.supertraits.get(name).cloned().unwrap_or_default();

        if sigs.is_empty() {
            let ty = if supers.is_empty() {
                "Record<never, never>".to_owned()
            } else {
                supers.join(" & ")
            };
            self.line(0, &format!("export type {name} = {ty};\n"));
            return;
        }

        if sigs.iter().all(|f| f.body.is_empty()) {
            let supertraits = if supers.is_empty() {
                String::new()
            } else {
                format!(" extends {}", supers.join(", "))
            };
            self.line(0, &format!("export interface {name}{supertraits} {{"));

            for f in sigs {
                let signature = self.signature(f, "", false);
                self.line(1, &format!("{signature};"));
            }
        } else {
            if !supers.is_empty() {
                self.current = name.to_owned();
                // A class extends one class, and implementing an interface
                // would declare its methods again.
                self.unsupported("a trait with provided methods and supertraits");
            }

            self.line(0, &format!("export abstract class {name} {{"));

            for f in sigs {
                if f.body.is_empty() {
                    let signature = self.signature(f, "abstract ", false);
                    self.line(1, &format!("{signature};"));
                } else {
                    self.func(f, 1, "");
                }
            }
        }

        self.line(0, "}\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::test_utilities::CLIENT;
    use crate::codegen::test_utilities::client_passed;
    use crate::codegen::ts::TsConfig;

    fn emitted(api: &str) -> Ts {
        let m = client_passed(api);
        let config = TsConfig {
            header: "// header".into(),
            import: "import type * as types from \"./types.ts\";".into(),
            schemas: "types".into(),
        };
        Emitter::new(&m, &config).client()
    }

    mod emitter {
        use super::*;

        mod client {
            use super::*;

            #[test]
            fn client_should_emit_the_consts_traits_data_methods_and_builders() {
                let ts = emitted(CLIENT);

                assert_eq!(ts.unsupported, Vec::<String>::new());
                assert_eq!(
                    ts.code,
                    "// header\n\nimport type * as types from \"./types.ts\";\n\ntype Kind = types.Kind;\n\nexport const PREFIX: string = \"/api\";\n\nexport type Client = Http;\n\nexport abstract class Http {\n    abstract get<T>(uri: string): Promise<[number, T]>;\n    async status(uri: string): Promise<number> {\n        return (await this.get<void>(uri))[0];\n    }\n}\n\nfunction Kind_to_string(self: Kind): string {\n    if (self === \"File\") {\n        return \"file\";\n    } else {\n        return \"memory\";\n    }\n}\n\nexport class Api<T extends Client> {\n    constructor(public client: T, public _base: string, public token: string | undefined) {}\n\n    static new<T extends Client>(client: T, address: string): Api<T> {\n        const base = (address.startsWith(\"http\") ? address : `http://${address}/api`);\n        return new Api(client, base, undefined);\n    }\n\n    base(): string {\n        return this._base;\n    }\n\n    async kind({ kind, n }: { kind: Kind; n: number }): Promise<number> {\n        return ((tuple1) => {\n            const status = tuple1[0];\n            return status;\n        })((await this.client.get<void>(`${this._base}/${Kind_to_string(kind)}/${n}`)));\n    }\n\n    async login(): Promise<number> {\n        const tuple1 = (await this.client.get(`${this._base}/login`));\n        const status = tuple1[0];\n        const token = tuple1[1];\n        this.token = token;\n        return status;\n    }\n\n    async ping(): Promise<number> {\n        return (await this.client.status(`${this._base}/ping`));\n    }\n\n    fail(): number {\n        throw new ApiError(1, \"fail\");\n    }\n}\n\nexport class ApiError {\n    constructor(public status: number, public description: string) {}\n\n    to_string(): string {\n        return `${this.status}: {${this.description}}`;\n    }\n}\n\n"
                );
            }

            #[test]
            fn client_should_report_a_trait_with_provided_methods_and_supertraits() {
                let ts = emitted(
                    r#"pub trait Base {
    fn b(&self) -> u8;
}
pub trait Provided: Base {
    fn p(&self) -> u8 {
        self.b()
    }
}
pub struct B<T: Provided> {
    t: T,
}
impl<T: Provided> B<T> {
    pub fn g(&self) -> u8 {
        self.t.p()
    }
}
"#,
                );

                assert_eq!(
                    ts.unsupported,
                    ["Provided: `a trait with provided methods and supertraits`"]
                );
            }

            #[test]
            fn client_should_emit_traits_without_provided_methods_as_interfaces() {
                let ts = emitted(
                    r#"pub mod util {}
pub use std::fmt;
pub type Num<'a, T> = &'a T;
pub trait Base {
    fn b(&self) -> u8;
}
pub trait Plain: Base + Send {
    fn f(&self) -> u8;
}
pub trait Marker {}
impl Marker for (u8, u8) {}
impl Base for u8 {
    fn b(&self) -> u8 {
        1
    }
}
pub struct B<T: Plain + Marker> {
    t: T,
}
impl<T: Plain + Marker> B<T> {
    pub fn g(&self) -> Num<'static, u8> {
        &self.t.f()
    }
}
"#,
                );

                assert_eq!(ts.unsupported, Vec::<String>::new());
                assert_eq!(
                    ts.code,
                    "// header\n\nimport type * as types from \"./types.ts\";\n\nexport type Base = Record<never, never>;\n\nexport type Marker = Record<never, never>;\n\nexport interface Plain extends Base {\n    f(): number;\n}\n\nexport class B<T extends Plain & Marker> {\n    constructor(public t: T) {}\n\n    g(): number {\n        return this.t.f();\n    }\n}\n\n"
                );
            }
        }
    }
}
