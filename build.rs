fn main() {
    println!("cargo:rustc-link-lib=X11");
    println!("cargo:rustc-link-search=native=/usr/X11R6/lib");
}
