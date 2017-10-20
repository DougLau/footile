// build.rs

extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/imgbuf.c")
        .flag("-march=native")
        .opt_level(3)
        .compile("imgbuf");
}
