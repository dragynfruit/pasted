use chrono::Local;

fn main() {
    let build_time = Local::now().to_rfc2822();
    println!("cargo:rustc-env=BUILD_DATE={}", build_time);
}