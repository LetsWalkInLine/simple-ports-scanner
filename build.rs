fn main() {
    println!("cargo:rustc-link-search=native=C:/ABCDEFG/WpdPack/Lib/x64");
    println!("cargo:rustc-link-lib=static=Packet");
}
