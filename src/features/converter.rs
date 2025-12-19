use crate::core::{Item, ItemType};
use regex::Regex;
use std::collections::HashMap;

pub struct Converter {
    length_units: HashMap<&'static str, f64>,
    weight_units: HashMap<&'static str, f64>,
    temperature_units: Vec<&'static str>,
    time_units: HashMap<&'static str, f64>,
    data_units: HashMap<&'static str, f64>,
}

impl Converter {
    pub fn new() -> Self {
        let mut length_units = HashMap::new();
        // Base unit: meters
        length_units.insert("m", 1.0);
        length_units.insert("meter", 1.0);
        length_units.insert("meters", 1.0);
        length_units.insert("km", 1000.0);
        length_units.insert("kilometer", 1000.0);
        length_units.insert("kilometers", 1000.0);
        length_units.insert("cm", 0.01);
        length_units.insert("centimeter", 0.01);
        length_units.insert("centimeters", 0.01);
        length_units.insert("mm", 0.001);
        length_units.insert("millimeter", 0.001);
        length_units.insert("millimeters", 0.001);
        length_units.insert("mi", 1609.344);
        length_units.insert("mile", 1609.344);
        length_units.insert("miles", 1609.344);
        length_units.insert("ft", 0.3048);
        length_units.insert("foot", 0.3048);
        length_units.insert("feet", 0.3048);
        length_units.insert("in", 0.0254);
        length_units.insert("inch", 0.0254);
        length_units.insert("inches", 0.0254);
        length_units.insert("yd", 0.9144);
        length_units.insert("yard", 0.9144);
        length_units.insert("yards", 0.9144);

        let mut weight_units = HashMap::new();
        // Base unit: grams
        weight_units.insert("g", 1.0);
        weight_units.insert("gram", 1.0);
        weight_units.insert("grams", 1.0);
        weight_units.insert("kg", 1000.0);
        weight_units.insert("kilogram", 1000.0);
        weight_units.insert("kilograms", 1000.0);
        weight_units.insert("mg", 0.001);
        weight_units.insert("milligram", 0.001);
        weight_units.insert("milligrams", 0.001);
        weight_units.insert("lb", 453.592);
        weight_units.insert("lbs", 453.592);
        weight_units.insert("pound", 453.592);
        weight_units.insert("pounds", 453.592);
        weight_units.insert("oz", 28.3495);
        weight_units.insert("ounce", 28.3495);
        weight_units.insert("ounces", 28.3495);
        weight_units.insert("t", 1_000_000.0);
        weight_units.insert("ton", 1_000_000.0);
        weight_units.insert("tons", 1_000_000.0);

        let temperature_units = vec!["c", "celsius", "f", "fahrenheit", "k", "kelvin"];

        let mut time_units = HashMap::new();
        // Base unit: seconds
        time_units.insert("s", 1.0);
        time_units.insert("sec", 1.0);
        time_units.insert("second", 1.0);
        time_units.insert("seconds", 1.0);
        time_units.insert("ms", 0.001);
        time_units.insert("millisecond", 0.001);
        time_units.insert("milliseconds", 0.001);
        time_units.insert("min", 60.0);
        time_units.insert("minute", 60.0);
        time_units.insert("minutes", 60.0);
        time_units.insert("h", 3600.0);
        time_units.insert("hr", 3600.0);
        time_units.insert("hour", 3600.0);
        time_units.insert("hours", 3600.0);
        time_units.insert("d", 86400.0);
        time_units.insert("day", 86400.0);
        time_units.insert("days", 86400.0);
        time_units.insert("w", 604800.0);
        time_units.insert("week", 604800.0);
        time_units.insert("weeks", 604800.0);
        time_units.insert("y", 31536000.0);
        time_units.insert("year", 31536000.0);
        time_units.insert("years", 31536000.0);

        let mut data_units = HashMap::new();
        // Base unit: bytes
        data_units.insert("b", 1.0);
        data_units.insert("byte", 1.0);
        data_units.insert("bytes", 1.0);
        data_units.insert("kb", 1024.0);
        data_units.insert("kilobyte", 1024.0);
        data_units.insert("kilobytes", 1024.0);
        data_units.insert("mb", 1024.0 * 1024.0);
        data_units.insert("megabyte", 1024.0 * 1024.0);
        data_units.insert("megabytes", 1024.0 * 1024.0);
        data_units.insert("gb", 1024.0 * 1024.0 * 1024.0);
        data_units.insert("gigabyte", 1024.0 * 1024.0 * 1024.0);
        data_units.insert("gigabytes", 1024.0 * 1024.0 * 1024.0);
        data_units.insert("tb", 1024.0 * 1024.0 * 1024.0 * 1024.0);
        data_units.insert("terabyte", 1024.0 * 1024.0 * 1024.0 * 1024.0);
        data_units.insert("terabytes", 1024.0 * 1024.0 * 1024.0 * 1024.0);

        Self {
            length_units,
            weight_units,
            temperature_units,
            time_units,
            data_units,
        }
    }

    pub fn get_items(&self, query: &str) -> Vec<Item> {
        if let Some((value, from, to)) = self.parse_conversion(query) {
            if let Some(result) = self.convert(value, &from, &to) {
                let result_str = if result.fract() == 0.0 {
                    format!("{}", result as i64)
                } else {
                    format!("{:.4}", result)
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_string()
                };

                let mut item = Item::new(
                    format!("convert:{}", result_str),
                    format!("{} {} = {} {}", value, from, result_str, to),
                    ItemType::Converter,
                )
                .with_description("Press Enter to copy result")
                .with_icon("accessories-calculator");

                item.metadata.content = Some(result_str);
                return vec![item];
            }
        }

        Vec::new()
    }

    fn parse_conversion(&self, query: &str) -> Option<(f64, String, String)> {
        // Patterns:
        // "100 km to mi"
        // "100km to mi"
        // "100 km in mi"
        // "100km in miles"

        let query = query.to_lowercase();
        let re = Regex::new(r"(\d+\.?\d*)\s*([a-z]+)\s+(?:to|in)\s+([a-z]+)").ok()?;

        if let Some(caps) = re.captures(&query) {
            let value: f64 = caps.get(1)?.as_str().parse().ok()?;
            let from = caps.get(2)?.as_str().to_string();
            let to = caps.get(3)?.as_str().to_string();
            return Some((value, from, to));
        }

        None
    }

    fn convert(&self, value: f64, from: &str, to: &str) -> Option<f64> {
        // Try length
        if let (Some(&from_factor), Some(&to_factor)) = (
            self.length_units.get(from),
            self.length_units.get(to),
        ) {
            let meters = value * from_factor;
            return Some(meters / to_factor);
        }

        // Try weight
        if let (Some(&from_factor), Some(&to_factor)) = (
            self.weight_units.get(from),
            self.weight_units.get(to),
        ) {
            let grams = value * from_factor;
            return Some(grams / to_factor);
        }

        // Try temperature
        if self.temperature_units.contains(&from) && self.temperature_units.contains(&to) {
            return self.convert_temperature(value, from, to);
        }

        // Try time
        if let (Some(&from_factor), Some(&to_factor)) = (
            self.time_units.get(from),
            self.time_units.get(to),
        ) {
            let seconds = value * from_factor;
            return Some(seconds / to_factor);
        }

        // Try data
        if let (Some(&from_factor), Some(&to_factor)) = (
            self.data_units.get(from),
            self.data_units.get(to),
        ) {
            let bytes = value * from_factor;
            return Some(bytes / to_factor);
        }

        None
    }

    fn convert_temperature(&self, value: f64, from: &str, to: &str) -> Option<f64> {
        // Convert to Celsius first
        let celsius = match from {
            "c" | "celsius" => value,
            "f" | "fahrenheit" => (value - 32.0) * 5.0 / 9.0,
            "k" | "kelvin" => value - 273.15,
            _ => return None,
        };

        // Convert from Celsius to target
        let result = match to {
            "c" | "celsius" => celsius,
            "f" | "fahrenheit" => celsius * 9.0 / 5.0 + 32.0,
            "k" | "kelvin" => celsius + 273.15,
            _ => return None,
        };

        Some(result)
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_conversion() {
        let conv = Converter::new();
        let result = conv.convert(1.0, "km", "m");
        assert_eq!(result, Some(1000.0));
    }

    #[test]
    fn test_temperature_conversion() {
        let conv = Converter::new();
        let result = conv.convert(100.0, "c", "f");
        assert!((result.unwrap() - 212.0).abs() < 0.001);
    }
}
