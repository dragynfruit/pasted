use byte_unit::{Byte, UnitType};
use chrono::DateTime;
use once_cell::sync::Lazy;
use std::{collections::HashMap, process};
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
        Ok(tera::to_value(
            DateTime::from_timestamp(num, 0)
                .unwrap()
                .format("%D %r %Z")
                .to_string(),
        )
        .unwrap())
    } else {
        Err(Error::msg(
            "Filter `format_date` was used on a value that isn't a valid number.",
        ))
    }
}

fn format_bytes(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    if let Some(num) = value.as_u64() {
        Ok(tera::to_value(format!(
            "{:#}",
            Byte::from_u64(num).get_appropriate_unit(UnitType::Decimal)
        ))
        .unwrap())
    } else {
        Err(Error::msg(
            "Filter `format_bytes` was used on a value that isn't a valid number.",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_date() {
        let value = tera::to_value(1618033988).unwrap();
        let result = format_date(&value, &HashMap::new()).unwrap();
        assert_eq!(result.as_str().unwrap(), "2021-04-10T05:53:08+00:00");
    }

    #[test]
    fn test_format_date_user() {
        let value = tera::to_value(1618033988).unwrap();
        let result = format_date_user(&value, &HashMap::new()).unwrap();
        assert_eq!(result.as_str().unwrap(), "04/10/21 05:53:08 AM UTC");
    }

    #[test]
    fn test_format_bytes() {
        let value = tera::to_value(147483647).unwrap();
        let result = format_bytes(&value, &HashMap::new()).unwrap();
        assert_eq!(result.as_str().unwrap(), "147.483647 MB");
    }
}
