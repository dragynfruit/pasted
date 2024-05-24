use byte_unit::Byte;
use chrono::DateTime;
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    process,
};
use tera::{Error, Result, Tera, Value};

pub static TEMPLATES: Lazy<Tera> = Lazy::new(|| {
    let mut tera = match Tera::new("templates/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            process::exit(1);
        }
    };
    tera.register_filter("format_date", format_date);
    tera.register_filter("format_date_user", format_date_user);
    tera.register_filter("format_bytes", format_bytes);
    tera
});

fn format_date(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    if let Some(num) = value.as_i64() {
        Ok(tera::to_value(DateTime::from_timestamp(num, 0).unwrap().to_rfc3339()).unwrap())
    } else {
        Err(Error::msg(
            "Filter `format_date` was used on a value that isn't a valid number.",
        ))
    }
}

fn format_date_user(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    if let Some(num) = value.as_i64() {
        Ok(tera::to_value(DateTime::from_timestamp(num, 0).unwrap().format("%D %r %Z").to_string()).unwrap())
    } else {
        Err(Error::msg(
            "Filter `format_date` was used on a value that isn't a valid number.",
        ))
    }
}

fn format_bytes(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    if let Some(num) = value.as_u64() {
        Ok(tera::to_value(format!("{:#}", Byte::from_u64(num))).unwrap())
    } else {
        Err(Error::msg(
            "Filter `format_bytes` was used on a value that isn't a valid number.",
        ))
    }
}