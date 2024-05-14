use once_cell::sync::Lazy;
use std::process;
use tera::Tera;

pub const URL: &str = "https://pastebin.com";
pub static TEMPLATES: Lazy<Tera> = Lazy::new(|| {
    let mut tera = match Tera::new("templates/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            process::exit(1);
        }
    };
    tera
});
