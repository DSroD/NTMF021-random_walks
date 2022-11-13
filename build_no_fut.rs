fn main() {
    let path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let path_h = std::path::PathBuf::from("");
    let package = futhark_bindgen::Package {
        c_file: path_h.join("walk.c"),
        h_file: path_h.join("walk.h"),
        manifest: futhark_bindgen::manifest::Manifest::parse_file("walk.json").expect("erererer"),
        src: path_h.join("futhark/walk.fut")
    };

    let out = path.join("walk.rs");
    let mut config = futhark_bindgen::Config::new(out).expect("Aaaaa");
    let mut gen = config.detect().expect("aaaaaaaaaaaa");
    gen.generate(&package, &mut config).expect("reeeeeeeeee");
    package.link();
}