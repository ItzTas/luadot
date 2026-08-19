mod hooks;
mod lpeg;
mod lua;
mod vendor;

fn main() {
    println!("cargo::rerun-if-changed=build");
    println!("cargo::rerun-if-changed=.githooks");

    hooks::install();

    let headers = lua::headers();
    if let Err(err) = lpeg::compile(&headers) {
        panic!("failed to build lpeg: {err}");
    }
}
