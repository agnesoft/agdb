//! [`Emitter::client`]: the PHP client of an API crate.

use super::Emitter;
use super::Php;
use crate::codegen::lowered::*;

impl Emitter<'_> {
    /// The [client](crate::SourceCrate::lower_client) module: the consts,
    /// the traits as the types the target implements (an abstract class
    /// when they provide methods), the methods of the data types as
    /// functions and the builders as exported classes.
    pub fn client(mut self) -> Php {
        let m = self.m;
        self.bounded = true;

        // Constants
        for (name, value) in &m.consts {
            self.current = name.clone();
            let value = self.expr(value);
            self.line(0, &format!("const {name} = {value};\n"));
        }

        // Traits as abstract classes or interfaces
        for (name, sigs) in &m.traits {
            self.trait_type(name, sigs);
        }

        // Data type methods as free functions
        self.data_methods();

        // Builder/client classes
        for b in &m.builders {
            self.class(&m.classes[b], true);
        }

        Php {
            code: self.php_header() + &self.out + "\n",
            notes: self.notes,
            unsupported: self.unsupported,
            values: Vec::new(),
        }
    }

    fn trait_type(&mut self, name: &str, sigs: &[Func]) {
        let supers = self.m.supertraits.get(name).cloned().unwrap_or_default();

        if sigs.is_empty() {
            // Empty trait — just a marker, skip in PHP.
            return;
        }

        if sigs.iter().all(|f| f.body.is_empty()) {
            // All abstract methods → interface
            let extends = if supers.is_empty() {
                String::new()
            } else {
                format!(" extends {}", supers.join(", "))
            };
            self.line(0, &format!("interface {name}{extends} {{"));

            for f in sigs {
                let sig = self.abstract_signature(f);
                self.line(1, &format!("{sig};"));
            }

            self.line(0, "}\n");
            return;
        }

        // Has provided methods → abstract class
        if !supers.is_empty() {
            self.current = name.to_owned();
            self.unsupported("a trait with provided methods and supertraits");
        }

        self.line(0, &format!("abstract class {name} {{"));

        for f in sigs {
            if f.body.is_empty() {
                let sig = self.abstract_signature(f);
                self.line(1, &format!("{sig};"));
            } else {
                self.func(f, 1, "public ");
            }
        }

        self.line(0, "}\n");
    }

    /// The signature of an abstract method.
    fn abstract_signature(&mut self, f: &Func) -> String {
        self.current = f.name.clone();
        let params: Vec<String> = f
            .params
            .iter()
            .map(|(n, t)| format!("{} ${n}", self.ty(t)))
            .collect();
        let ret = self.return_ty(&f.ret);
        format!(
            "abstract public function {}({}): {ret}",
            f.name,
            params.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::php::PhpConfig;
    use crate::codegen::test_utilities::CLIENT;
    use crate::codegen::test_utilities::client_passed;

    fn emitted(api: &str) -> Php {
        let m = client_passed(api);
        let config = PhpConfig {
            header: "// header".into(),
            namespace: "Test".into(),
            types_require: "types".into(),
        };
        Emitter::new(&m, &config).client()
    }

    mod emitter {
        use super::*;

        mod client {
            use super::*;

            #[test]
            fn client_should_emit_the_consts_traits_data_methods_and_builders() {
                let php = emitted(CLIENT);

                assert_eq!(php.unsupported, Vec::<String>::new());
                assert_eq!(
                    php.code,
                    "<?php\n// header\n\nnamespace Test;\n\nrequire_once __DIR__ . '/types.php';\n\nconst PREFIX = '/api';\n\nabstract class Http {\n    abstract public function get(string $uri): array;\n    public function status(string $uri): int {\n        return $this->get($uri)[0];\n    }\n}\n\nfunction Kind_to_string(Kind $self): string {\n    if (is_array($self) && array_key_exists('File', $self)) {\n        return 'file';\n    } else {\n        return 'memory';\n    }\n}\n\nclass Api {\n    public function __construct(public $client, public string $base, public string|null $token) {}\n\n    public static function new($client, string $address): Api {\n        $base = (str_starts_with($address, 'http') ? $address : 'http://' . $address . '/api');\n        return new Api($client, $base, null);\n    }\n\n    public function base(): string {\n        return $this->base;\n    }\n\n    public function kind(Kind $kind, int $n): int {\n        return (function($tuple1) {\n            $status = $tuple1[0];\n            return $status;\n        })($this->client->get($this->base . '/' . Kind_to_string($kind) . '/' . $n));\n    }\n\n    public function login(): int {\n        $tuple1 = $this->client->get($this->base . '/login');\n        $status = $tuple1[0];\n        $token = $tuple1[1];\n        $this->token = $token;\n        return $status;\n    }\n\n    public function ping(): int {\n        return $this->client->status($this->base . '/ping');\n    }\n\n    public function fail(): int {\n        throw new ApiError(1, 'fail');\n    }\n}\n\nclass ApiError extends \\Exception {\n    public function __construct(public int $status, public string $description) { parent::__construct($this->to_string()); }\n\n    public function to_string(): string {\n        return $this->status . ': {' . $this->description . '}';\n    }\n}\n\n\n"
                );
            }

            #[test]
            fn client_should_report_a_trait_with_provided_methods_and_supertraits() {
                let php = emitted(
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
                    php.unsupported,
                    ["Provided: `a trait with provided methods and supertraits`"]
                );
            }

            #[test]
            fn client_should_emit_traits_without_provided_methods_as_interfaces() {
                let php = emitted(
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

                assert_eq!(php.unsupported, Vec::<String>::new());
                assert_eq!(
                    php.code,
                    "<?php\n// header\n\nnamespace Test;\n\nrequire_once __DIR__ . '/types.php';\n\ninterface Plain extends Base {\n    abstract public function f(): int;\n}\n\nclass B {\n    public function __construct(public $t) {}\n\n    public function g(): int {\n        return $this->t->f();\n    }\n}\n\n\n"
                );
            }
        }
    }
}
