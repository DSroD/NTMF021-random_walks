use std::env;

fn main() {
    let env_var = env::var("BACKEND");
    let target_backend = match env_var.as_ref().map(|x| &**x) {
        Ok("cuda") => futhark_bindgen::Backend::CUDA,
        Ok("c") | Ok("") | Err(env::VarError::NotPresent) => futhark_bindgen::Backend::C,
        Ok("multicore") => futhark_bindgen::Backend::Multicore,
        Ok("ispc") => futhark_bindgen::Backend::ISPC,
        other => panic!("Unknown backend {:?}!", other)
    };

    println!("Using backend {:?}", env_var.as_ref().map(|x| &**x));

    futhark_bindgen::build(target_backend, "futhark/walk.fut", "walk.rs")
}