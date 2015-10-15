extern crate typenum;

use std::env;
use std::fs::File;
use std::io::{Write, BufWriter};
use std::path::Path;
use std::fmt;
use std::process::Command;

use typenum::__private::build::{gen_int, gen_uint};

fn sign(i: i64) -> char {
    if i > 0 { 'P' } else if i < 0 { 'N' } else { '_' }
}

struct UIntTest {
    a: u64,
    op: &'static str,
    b: Option<u64>,
    r: u64,
}

impl fmt::Display for UIntTest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.b {
            Some(b) => write!(f, "
{{
    type A = {gen_a};
    type B = {gen_b};
    type U{r} = {result};

    type U{a}{op}U{b} = <<A as {op}<B>>::Output as Same<U{r}>>::Output;
    assert_eq!(<U{a}{op}U{b} as Unsigned>::to_u64(), <U{r} as Unsigned>::to_u64());
}}
",
                              gen_a = gen_uint(self.a),
                              gen_b = gen_uint(b),
                              r = self.r,
                              result = gen_uint(self.r),
                              a = self.a,
                              b = b,
                              op = self.op),
            None => write!(f, "
{{
    type A = {gen_a};
    type U{r} = {result};

    type {op}U{a} = <<A as {op}>::Output as Same<U{r}>>::Output;
    assert_eq!(<{op}U{a} as Unsigned>::to_u64(), <U{r} as Unsigned>::to_u64());
}}
",
                           gen_a = gen_uint(self.a),
                           r = self.r,
                           result = gen_uint(self.r),
                           a = self.a,
                           op = self.op)
        }
    }
}

fn uint_binary_test(a: u64, op: &'static str, b: u64, result: u64) -> UIntTest {
    UIntTest { a: a, op: op, b: Option::Some(b), r: result }
}

fn uint_unary_test(op: &'static str, a: u64, result: u64) -> UIntTest {
    UIntTest { a: a, op: op, b: Option::None, r: result }
}

struct IntBinaryTest {
    a: i64,
    op: &'static str,
    b: i64,
    r: i64,
}

impl fmt::Display for IntBinaryTest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "
{{
    type A = {gen_a};
    type B = {gen_b};
    type {sr}{r} = {result};

    type {sa}{a}{op}{sb}{b} = <<A as {op}<B>>::Output as Same<{sr}{r}>>::Output;
    assert_eq!(<{sa}{a}{op}{sb}{b} as Integer>::to_i64(), <{sr}{r} as Integer>::to_i64());
}}
",
               gen_a = gen_int(self.a),
               gen_b = gen_int(self.b),
               r = self.r.abs(),
               sr = sign(self.r),
               result = gen_int(self.r),
               a = self.a.abs(),
               b = self.b.abs(),
               sa = sign(self.a),
               sb = sign(self.b),
               op = self.op)
    }
}

fn int_binary_test(a: i64, op: &'static str, b: i64, result: i64) -> IntBinaryTest {
    IntBinaryTest { a: a, op: op, b: b, r: result }
}

struct IntUnaryTest {
    op: &'static str,
    a: i64,
    r: i64,
}

impl fmt::Display for IntUnaryTest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "
{{
    type A = {gen_a};
    type {sr}{r} = {result};

    type {op}{sa}{a} = <<A as {op}>::Output as Same<{sr}{r}>>::Output;
    assert_eq!(<{op}{sa}{a} as Integer>::to_i64(), <{sr}{r} as Integer>::to_i64());
}}
",
               gen_a = gen_int(self.a),
               r = self.r.abs(),
               sr = sign(self.r),
               result = gen_int(self.r),
               a = self.a.abs(),
               sa = sign(self.a),
               op = self.op)
    }
}

fn int_unary_test(op: &'static str, a: i64, result: i64) -> IntUnaryTest {
    IntUnaryTest { op: op, a: a, r: result }
}

fn uint_cmp_test(a: u64, b: u64) -> String {
    format!("
{{
    type A = {a};
    type B = {b};
    type Result = {result:?};

    type {computed_name} = <A as Cmp<B>>::Output;
    assert_eq!(<{computed_name} as Ord>::to_ordering(), Ordering::{result:?});
}}
",
            a = gen_uint(a),
            b = gen_uint(b),
            result = a.cmp(&b),
            computed_name = format!("U{}_Cmp_U{}", a, b))
}

#[test]
fn test_all() {
    //let uints = (0..9).map(|a| );
    let uints = vec![(0, 0), (0, 1), (1, 0), (1, 1), (1, 2), (3, 4)];
    let ints = vec![(0, 0), (0, 1), (0, -1), (1, 0), (-1, 0), (1, 1), (-1, 1), (1, -1), (-1, -1), (1, 2), (3, 4)];


    // int operators: Neg, Add, Sub, Mul, Div, Pow, Cmp
    // let int_tests = ints.iter().map(|&(a, _)| int_unary_test("Neg", a, -a))
    //     .chain(ints.iter().map(|&(a, b)| int_binary_test(a, "Add", b, a + b)))
    //     .chain(ints.iter().map(|&(a, b)| int_binary_test(a, "Sub", b, a - b)))
    //     .chain(ints.iter().map(|&(a, b)| int_binary_test(a, "Mul", b, a * b)))
    //     .chain(ints.iter().filter(|&&(_, b)| b != 0).map(|&(a, b)| int_binary_test(a, "Div", b, a / b)))
    //     ;
    // run_tests(int_tests.collect());

    let out_dir = env::var("OUT_DIR").unwrap();
    let test_dir = Path::new(&out_dir).join("test/");
    let cargo = Path::new(&out_dir).join("test/Cargo.toml");
    let main = Path::new(&out_dir).join("test/src/main.rs");

    Command::new("cargo").arg("new").arg("--bin").arg(&test_dir).output().unwrap();

    // Write cargo file
    let mut cargof = File::create(&cargo).unwrap();
    write!(cargof, "
[package]
name = \"test\"
version = \"0.0.1\"

[dependencies.typenum]
# typenum = \"0.1.0\"
git = \"file:{}\"
", env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();

    // Write main.rs
    let mainf = File::create(&main).unwrap();
    let mut writer = BufWriter::new(&mainf);
    writer.write(b"
#![allow(unused_imports)]
#![allow(non_camel_case_types)]
extern crate typenum;

use std::ops::{BitAnd, BitOr, BitXor, Shl, Shr, Neg, Add, Sub, Mul, Div, Rem};
use std::cmp::Ordering;
use typenum::{NonZero, Same, Pow, Ord, Cmp, SizeOf, Greater, Less, Equal};
use typenum::bit::{Bit, B0, B1};
use typenum::uint::{Unsigned, UInt, UTerm};
use typenum::int::{Integer, NInt, PInt, Z0};

fn main() {
").unwrap();
    // uint operators: BitAnd, BitOr, BitXor, Shl, Shr, Add, Sub, Mul, Div, Pow, Cmp, SizeOf
    for (a, b) in uints {
        write!(writer, "{}", uint_binary_test(a, "BitAnd", b, a & b)).unwrap();
        write!(writer, "{}", uint_binary_test(a, "BitOr", b, a | b)).unwrap();
        write!(writer, "{}", uint_binary_test(a, "BitXor", b, a ^ b)).unwrap();
        write!(writer, "{}", uint_binary_test(a, "Shl", b, a << b)).unwrap();
        write!(writer, "{}", uint_binary_test(a, "Shr", b, a >> b)).unwrap();
        write!(writer, "{}", uint_binary_test(a, "Add", b, a + b)).unwrap();
        if a >= b {
            write!(writer, "{}", uint_binary_test(a, "Sub", b, a - b)).unwrap();
        }
        write!(writer, "{}", uint_binary_test(a, "Mul", b, a * b)).unwrap();
        if b != 0 {
            write!(writer, "{}", uint_binary_test(a, "Div", b, a / b)).unwrap();
        }
        write!(writer, "{}", uint_binary_test(a, "Pow", b, a.pow(b as u32))).unwrap();
    }
    for (a, b) in ints {
        write!(writer, "{}", int_unary_test("Neg", a, -a)).unwrap();
        write!(writer, "{}", int_binary_test(a, "Add", b, a + b)).unwrap();
        write!(writer, "{}", int_binary_test(a, "Sub", b, a - b)).unwrap();
        write!(writer, "{}", int_binary_test(a, "Mul", b, a * b)).unwrap();
        if b != 0 {
            write!(writer, "{}", int_binary_test(a, "Div", b, a / b)).unwrap();
        }
    }
    writer.write(b"}").unwrap();
    writer.flush();

    Command::new("cargo").arg("update").current_dir(&test_dir).output().unwrap();
    let test_out = Command::new("cargo").arg("run").current_dir(&test_dir).output().unwrap();
    if !test_out.status.success() {
        let stdout = ::std::str::from_utf8(&test_out.stdout).unwrap();
        let stderr = ::std::str::from_utf8(&test_out.stderr).unwrap();
        panic!("Exit status: {}.\nStdout: {}\nStderr: {}\n", test_out.status, stdout, stderr);
    }
}
