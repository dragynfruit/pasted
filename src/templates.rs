use byte_unit::{Byte, UnitType};
use chrono::DateTime;
use once_cell::sync::Lazy;
use std::{collections::HashMap, env, process};
use tera::{Error, Result, Tera, Value};

pub static BANNER: Lazy<String> = Lazy::new(|| env::var("BANNER").unwrap_or_default());

#[cfg(feature = "include_templates")]
use include_dir::include_dir;

pub static TEMPLATES: Lazy<Tera> = Lazy::new(|| {
    #[cfg(feature = "include_templates")]
    let mut tera = {
        let dir = include_dir!("templates");
        let mut tera = Tera::default();

        let templates = dir
            .files()
            .filter_map(|file| {
                let name = file.path().to_str()?;
                let content = std::str::from_utf8(file.contents()).ok()?;
                Some((name, content))
            })
            .collect::<Vec<_>>();

        match tera.add_raw_templates(templates) {
            Ok(_) => tera,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                process::exit(1);
            }
        }
    };

    #[cfg(not(feature = "include_templates"))]
    let mut tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            process::exit(1);
        }
    };

    tera.register_filter("format_date", format_date);
    tera.register_filter("format_date_user", format_date_user);
    tera.register_filter("format_bytes", format_bytes);
    tera.register_function("get_banner", get_banner);
    tera
});

fn format_date(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    if let Some(num) = value.as_i64() {
        match DateTime::from_timestamp(num, 0) {
            Some(dt) => match tera::to_value(dt.to_rfc3339()) {
                Ok(v) => Ok(v),
                Err(e) => Err(Error::msg(format!("Failed to serialize date: {}", e))),
            },
            None => Err(Error::msg("Invalid timestamp value")),
        }
    } else {
        Err(Error::msg(
            "Filter `format_date` was used on a value that isn't a valid number.",
        ))
    }
}

fn format_date_user(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    if let Some(num) = value.as_i64() {
        match DateTime::from_timestamp(num, 0) {
            Some(dt) => {
                let formatted = dt.format("%D %r %Z").to_string();
                match tera::to_value(formatted) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(Error::msg(format!(
                        "Failed to serialize formatted date: {}",
                        e
                    ))),
                }
            }
            None => Err(Error::msg("Invalid timestamp value")),
        }
    } else {
        Err(Error::msg(
            "Filter `format_date` was used on a value that isn't a valid number.",
        ))
    }
}

fn format_bytes(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    if let Some(num) = value.as_u64() {
        let formatted = format!(
            "{:#}",
            Byte::from_u64(num).get_appropriate_unit(UnitType::Decimal)
        );
        match tera::to_value(formatted) {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::msg(format!("Failed to serialize byte value: {}", e))),
        }
    } else {
        Err(Error::msg(
            "Filter `format_bytes` was used on a value that isn't a valid number.",
        ))
    }
}

fn get_banner(_: &HashMap<String, Value>) -> Result<Value> {
    match tera::to_value(BANNER.clone()) {
        Ok(v) => Ok(v),
        Err(e) => Err(Error::msg(format!("Failed to serialize banner: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_date() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let value = tera::to_value(1618033988)?;
        let result = format_date(&value, &HashMap::new())?;
        assert_eq!(result.as_str().unwrap_or(""), "2021-04-10T05:53:08+00:00");
        Ok(())
    }

    #[test]
    fn test_format_date_user() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let value = tera::to_value(1618033988)?;
        let result = format_date_user(&value, &HashMap::new())?;
        assert_eq!(result.as_str().unwrap_or(""), "04/10/21 05:53:08 AM UTC");
        Ok(())
    }

    #[test]
    fn test_format_bytes() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let value = tera::to_value(147483647)?;
        let result = format_bytes(&value, &HashMap::new())?;
        assert_eq!(result.as_str().unwrap_or(""), "147.483647 MB");
        Ok(())
    }
}
