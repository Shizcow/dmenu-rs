fn main() {
    println!("cargo:rustc-link-lib=X11");
    //println!("cargo:rustc-link-lib=Xrender");
    println!("cargo:rustc-link-lib=Xft");
}
