use chrono::Local;

fn main() {
    let build_time = Local::now().to_rfc2822();
    println!("cargo:rustc-env=BUILD_DATE={}", build_time);
    let action_name = std::env::var("GITHUB_ACTIONS").unwrap_or("Actions not detected".to_string());
    println!("cargo:rustc-env=ACTION_NAME={}", action_name);
}