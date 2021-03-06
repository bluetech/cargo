extern crate cargo;
extern crate cargotest;
extern crate hamcrest;

use cargo::util::paths::dylib_path_envvar;
use cargotest::support::{project, execs, path2url};
use hamcrest::{assert_that, existing_file};

#[test]
fn simple() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#)
        .file("src/main.rs", r#"
            fn main() { println!("hello"); }
        "#);

    assert_that(p.cargo_process("run"),
                execs().with_status(0)
                       .with_stderr(&format!("\
[COMPILING] foo v0.0.1 ({dir})
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `target[/]debug[/]foo[EXE]`", dir = path2url(p.root())))
                       .with_stdout("\
hello
"));
    assert_that(&p.bin("foo"), existing_file());
}

#[test]
fn simple_quiet() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#)
        .file("src/main.rs", r#"
            fn main() { println!("hello"); }
        "#);

    assert_that(p.cargo_process("run").arg("-q"),
                execs().with_status(0).with_stdout("\
hello
")
    );
}

#[test]
fn simple_quiet_and_verbose() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#)
        .file("src/main.rs", r#"
            fn main() { println!("hello"); }
        "#);

    assert_that(p.cargo_process("run").arg("-q").arg("-v"),
                execs().with_status(101).with_stderr("\
[ERROR] cannot set both --verbose and --quiet
"));
}

#[test]
fn quiet_and_verbose_config() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#)
        .file(".cargo/config", r#"
            [term]
            verbose = true
        "#)
        .file("src/main.rs", r#"
            fn main() { println!("hello"); }
        "#);

    assert_that(p.cargo_process("run").arg("-q"),
                execs().with_status(0));
}

#[test]
fn simple_with_args() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#)
        .file("src/main.rs", r#"
            fn main() {
                assert_eq!(std::env::args().nth(1).unwrap(), "hello");
                assert_eq!(std::env::args().nth(2).unwrap(), "world");
            }
        "#);

    assert_that(p.cargo_process("run").arg("hello").arg("world"),
                execs().with_status(0));
}

#[test]
fn exit_code() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#)
        .file("src/main.rs", r#"
            fn main() { std::process::exit(2); }
        "#);

    let mut output = String::from("\
[COMPILING] foo v0.0.1 (file[..])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `target[..]`
");
    if !cfg!(unix) {
        output.push_str("\
[ERROR] process didn't exit successfully: `target[..]foo[..]` (exit code: 2)
");
    }
    assert_that(p.cargo_process("run"),
                execs().with_status(2).with_stderr(output));
}

#[test]
fn exit_code_verbose() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#)
        .file("src/main.rs", r#"
            fn main() { std::process::exit(2); }
        "#);

    let mut output = String::from("\
[COMPILING] foo v0.0.1 (file[..])
[RUNNING] `rustc [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `target[..]`
");
    if !cfg!(unix) {
        output.push_str("\
[ERROR] process didn't exit successfully: `target[..]foo[..]` (exit code: 2)
");
    }

    assert_that(p.cargo_process("run").arg("-v"),
                execs().with_status(2).with_stderr(output));
}

#[test]
fn no_main_file() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#)
        .file("src/lib.rs", "");

    assert_that(p.cargo_process("run"),
                execs().with_status(101)
                       .with_stderr("[ERROR] a bin target must be available \
                                     for `cargo run`\n"));
}

#[test]
fn too_many_bins() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#)
        .file("src/lib.rs", "")
        .file("src/bin/a.rs", "")
        .file("src/bin/b.rs", "");

    assert_that(p.cargo_process("run"),
                execs().with_status(101)
                       .with_stderr("[ERROR] `cargo run` requires that a project only \
                                     have one executable; use the `--bin` option \
                                     to specify which one to run\n"));
}

#[test]
fn specify_name() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#)
        .file("src/lib.rs", "")
        .file("src/bin/a.rs", r#"
            extern crate foo;
            fn main() { println!("hello a.rs"); }
        "#)
        .file("src/bin/b.rs", r#"
            extern crate foo;
            fn main() { println!("hello b.rs"); }
        "#);

    assert_that(p.cargo_process("run").arg("--bin").arg("a").arg("-v"),
                execs().with_status(0)
                       .with_stderr(&format!("\
[COMPILING] foo v0.0.1 ({dir})
[RUNNING] `rustc [..] src[/]lib.rs [..]`
[RUNNING] `rustc [..] src[/]bin[/]a.rs [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `target[/]debug[/]a[EXE]`", dir = path2url(p.root())))
                       .with_stdout("\
hello a.rs
"));

    assert_that(p.cargo("run").arg("--bin").arg("b").arg("-v"),
                execs().with_status(0)
                       .with_stderr("\
[COMPILING] foo v0.0.1 ([..])
[RUNNING] `rustc [..] src[/]bin[/]b.rs [..]`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `target[/]debug[/]b[EXE]`")
                       .with_stdout("\
hello b.rs
"));
}

#[test]
fn run_example() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#)
        .file("src/lib.rs", "")
        .file("examples/a.rs", r#"
            fn main() { println!("example"); }
        "#)
        .file("src/bin/a.rs", r#"
            fn main() { println!("bin"); }
        "#);

    assert_that(p.cargo_process("run").arg("--example").arg("a"),
                execs().with_status(0)
                       .with_stderr(&format!("\
[COMPILING] foo v0.0.1 ({dir})
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `target[/]debug[/]examples[/]a[EXE]`", dir = path2url(p.root())))
                       .with_stdout("\
example
"));
}

#[test]
fn run_with_filename() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#)
        .file("src/lib.rs", "")
        .file("src/bin/a.rs", r#"
            extern crate foo;
            fn main() { println!("hello a.rs"); }
        "#)
        .file("examples/a.rs", r#"
            fn main() { println!("example"); }
        "#);

    assert_that(p.cargo_process("run").arg("--bin").arg("bin.rs"),
                execs().with_status(101).with_stderr("\
[ERROR] no bin target named `bin.rs`"));

    assert_that(p.cargo_process("run").arg("--bin").arg("a.rs"),
                execs().with_status(101).with_stderr("\
[ERROR] no bin target named `a.rs`

Did you mean `a`?"));

    assert_that(p.cargo_process("run").arg("--example").arg("example.rs"),
                execs().with_status(101).with_stderr("\
[ERROR] no example target named `example.rs`"));

    assert_that(p.cargo_process("run").arg("--example").arg("a.rs"),
                execs().with_status(101).with_stderr("\
[ERROR] no example target named `a.rs`

Did you mean `a`?"));
}

#[test]
fn either_name_or_example() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#)
        .file("src/bin/a.rs", r#"
            fn main() { println!("hello a.rs"); }
        "#)
        .file("examples/b.rs", r#"
            fn main() { println!("hello b.rs"); }
        "#);

    assert_that(p.cargo_process("run").arg("--bin").arg("a").arg("--example").arg("b"),
                execs().with_status(101)
                       .with_stderr("[ERROR] `cargo run` can run at most one \
                                     executable, but multiple were \
                                     specified"));
}

#[test]
fn one_bin_multiple_examples() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#)
        .file("src/lib.rs", "")
        .file("src/bin/main.rs", r#"
            fn main() { println!("hello main.rs"); }
        "#)
        .file("examples/a.rs", r#"
            fn main() { println!("hello a.rs"); }
        "#)
        .file("examples/b.rs", r#"
            fn main() { println!("hello b.rs"); }
        "#);

    assert_that(p.cargo_process("run"),
                execs().with_status(0)
                       .with_stderr(&format!("\
[COMPILING] foo v0.0.1 ({dir})
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `target[/]debug[/]main[EXE]`", dir = path2url(p.root())))
                       .with_stdout("\
hello main.rs
"));
}

#[test]
fn example_with_release_flag() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies.bar]
            version = "*"
            path = "bar"
        "#)
        .file("examples/a.rs", r#"
            extern crate bar;

            fn main() {
                if cfg!(debug_assertions) {
                    println!("slow1")
                } else {
                    println!("fast1")
                }
                bar::baz();
            }
        "#)
        .file("bar/Cargo.toml", r#"
            [project]
            name = "bar"
            version = "0.0.1"
            authors = []

            [lib]
            name = "bar"
        "#)
        .file("bar/src/bar.rs", r#"
            pub fn baz() {
                if cfg!(debug_assertions) {
                    println!("slow2")
                } else {
                    println!("fast2")
                }
            }
        "#);

    assert_that(p.cargo_process("run").arg("-v").arg("--release").arg("--example").arg("a"),
                execs().with_status(0)
                       .with_stderr(&format!("\
[COMPILING] bar v0.0.1 ({url}/bar)
[RUNNING] `rustc --crate-name bar bar[/]src[/]bar.rs --crate-type lib \
        --emit=dep-info,link \
        -C opt-level=3 \
        -C metadata=[..] \
        --out-dir {dir}[/]target[/]release[/]deps \
        -L dependency={dir}[/]target[/]release[/]deps`
[COMPILING] foo v0.0.1 ({url})
[RUNNING] `rustc --crate-name a examples[/]a.rs --crate-type bin \
        --emit=dep-info,link \
        -C opt-level=3 \
        -C metadata=[..] \
        --out-dir {dir}[/]target[/]release[/]examples \
        -L dependency={dir}[/]target[/]release[/]deps \
         --extern bar={dir}[/]target[/]release[/]deps[/]libbar-[..].rlib`
[FINISHED] release [optimized] target(s) in [..]
[RUNNING] `target[/]release[/]examples[/]a[EXE]`
",
        dir = p.root().display(),
        url = path2url(p.root()),
        ))
                       .with_stdout("\
fast1
fast2"));

    assert_that(p.cargo("run").arg("-v").arg("--example").arg("a"),
                execs().with_status(0)
                       .with_stderr(&format!("\
[COMPILING] bar v0.0.1 ({url}/bar)
[RUNNING] `rustc --crate-name bar bar[/]src[/]bar.rs --crate-type lib \
        --emit=dep-info,link \
        -C debuginfo=2 \
        -C metadata=[..] \
        --out-dir {dir}[/]target[/]debug[/]deps \
        -L dependency={dir}[/]target[/]debug[/]deps`
[COMPILING] foo v0.0.1 ({url})
[RUNNING] `rustc --crate-name a examples[/]a.rs --crate-type bin \
        --emit=dep-info,link \
        -C debuginfo=2 \
        -C metadata=[..] \
        --out-dir {dir}[/]target[/]debug[/]examples \
        -L dependency={dir}[/]target[/]debug[/]deps \
         --extern bar={dir}[/]target[/]debug[/]deps[/]libbar-[..].rlib`
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
[RUNNING] `target[/]debug[/]examples[/]a[EXE]`
",
        dir = p.root().display(),
        url = path2url(p.root()),
        ))
                       .with_stdout("\
slow1
slow2"));
}

#[test]
fn run_dylib_dep() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies.bar]
            path = "bar"
        "#)
        .file("src/main.rs", r#"
            extern crate bar;
            fn main() { bar::bar(); }
        "#)
        .file("bar/Cargo.toml", r#"
            [package]
            name = "bar"
            version = "0.0.1"
            authors = []

            [lib]
            name = "bar"
            crate-type = ["dylib"]
        "#)
        .file("bar/src/lib.rs", "pub fn bar() {}");

    assert_that(p.cargo_process("run").arg("hello").arg("world"),
                execs().with_status(0));
}

#[test]
fn release_works() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#)
        .file("src/main.rs", r#"
            fn main() { if cfg!(debug_assertions) { panic!() } }
        "#);

    assert_that(p.cargo_process("run").arg("--release"),
                execs().with_status(0).with_stderr(&format!("\
[COMPILING] foo v0.0.1 ({dir})
[FINISHED] release [optimized] target(s) in [..]
[RUNNING] `target[/]release[/]foo[EXE]`
",
        dir = path2url(p.root()),
        )));
    assert_that(&p.release_bin("foo"), existing_file());
}

#[test]
fn run_bin_different_name() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [[bin]]
            name = "bar"
        "#)
        .file("src/bar.rs", r#"
            fn main() { }
        "#);

    assert_that(p.cargo_process("run"), execs().with_status(0));
}

#[test]
fn dashes_are_forwarded() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [[bin]]
            name = "bar"
        "#)
        .file("src/main.rs", r#"
            fn main() {
                let s: Vec<String> = std::env::args().collect();
                assert_eq!(s[1], "a");
                assert_eq!(s[2], "--");
                assert_eq!(s[3], "b");
            }
        "#);

    assert_that(p.cargo_process("run").arg("--").arg("a").arg("--").arg("b"),
                execs().with_status(0));
}

#[test]
fn run_from_executable_folder() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#)
        .file("src/main.rs", r#"
            fn main() { println!("hello"); }
        "#);

    let cwd = p.root().join("target").join("debug");
    p.cargo_process("build").exec_with_output().unwrap();

    assert_that(p.cargo("run").cwd(cwd),
                execs().with_status(0)
                       .with_stderr("\
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]\n\
[RUNNING] `.[/]foo[EXE]`")
                       .with_stdout("\
hello
"));
}

#[test]
fn run_with_library_paths() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
            build = "build.rs"
        "#)
        .file("build.rs", r#"
            fn main() {
                println!("cargo:rustc-link-search=native=foo");
                println!("cargo:rustc-link-search=bar");
                println!("cargo:rustc-link-search=/path=containing=equal=signs");
            }
        "#)
        .file("src/main.rs", &format!(r#"
            fn main() {{
                let search_path = std::env::var_os("{}").unwrap();
                let paths = std::env::split_paths(&search_path).collect::<Vec<_>>();
                assert!(paths.contains(&"foo".into()));
                assert!(paths.contains(&"bar".into()));
                assert!(paths.contains(&"/path=containing=equal=signs".into()));
            }}
        "#, dylib_path_envvar()));

    assert_that(p.cargo_process("run"), execs().with_status(0));
}

#[test]
fn fail_no_extra_verbose() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#)
        .file("src/main.rs", r#"
            fn main() {
                std::process::exit(1);
            }
        "#);

    assert_that(p.cargo_process("run").arg("-q"),
                execs().with_status(1)
                       .with_stdout("")
                       .with_stderr(""));
}
