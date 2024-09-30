use chrono::Local;

fn main() {
    let build_time = Local::now().to_rfc2822();
    println!("cargo:rustc-env=BUILD_DATE={}", build_time);
    let used_actions = std::env::var("GITHUB_ACTIONS").unwrap_or_default() == "true";
    println!("cargo:rustc-env=USED_ACTIONS={}", used_actions);
}