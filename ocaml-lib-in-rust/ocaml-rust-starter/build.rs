pub fn main() -> std::io::Result<()> {
    ocaml_build::Sigs::new("src/ocaml_lib_in_rust.ml").generate()
}
