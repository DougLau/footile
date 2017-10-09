// build.rs

extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/imgbuf.c")
        .opt_level(3)
        .compile("imgbuf");
}
