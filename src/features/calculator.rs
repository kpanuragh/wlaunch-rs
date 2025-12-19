use crate::core::{Item, ItemType};

pub struct Calculator;

impl Calculator {
    pub fn new() -> Self {
        Self
    }

    pub fn get_items(&self, query: &str) -> Vec<Item> {
        if query.is_empty() {
            return Vec::new();
        }

        match self.evaluate(query) {
            Some(result) => {
                let result_str = if result.fract() == 0.0 {
                    format!("{}", result as i64)
                } else {
                    format!("{:.6}", result).trim_end_matches('0').trim_end_matches('.').to_string()
                };

                let mut item = Item::new(
                    format!("calc:{}", result_str),
                    format!("{} = {}", query, result_str),
                    ItemType::Calculator,
                )
                .with_description("Press Enter to copy result")
                .with_icon("accessories-calculator");

                item.metadata.content = Some(result_str);
                vec![item]
            }
            None => Vec::new(),
        }
    }

    fn evaluate(&self, expr: &str) -> Option<f64> {
        // Simple expression parser
        // Supports: +, -, *, /, ^, %, ()

        let expr = expr
            .replace(" ", "")
            .replace("x", "*")
            .replace("×", "*")
            .replace("÷", "/")
            .replace("^", "**");

        Self::parse_expression(&expr)
    }

    fn parse_expression(expr: &str) -> Option<f64> {
        Self::parse_additive(expr).map(|(result, _)| result)
    }

    fn parse_additive(expr: &str) -> Option<(f64, &str)> {
        let (mut left, mut rest) = Self::parse_multiplicative(expr)?;

        while let Some(c) = rest.chars().next() {
            match c {
                '+' => {
                    let (right, new_rest) = Self::parse_multiplicative(&rest[1..])?;
                    left += right;
                    rest = new_rest;
                }
                '-' => {
                    let (right, new_rest) = Self::parse_multiplicative(&rest[1..])?;
                    left -= right;
                    rest = new_rest;
                }
                _ => break,
            }
        }

        Some((left, rest))
    }

    fn parse_multiplicative(expr: &str) -> Option<(f64, &str)> {
        let (mut left, mut rest) = Self::parse_power(expr)?;

        while let Some(c) = rest.chars().next() {
            match c {
                '*' => {
                    if rest.starts_with("**") {
                        break; // Power operator, handle elsewhere
                    }
                    let (right, new_rest) = Self::parse_power(&rest[1..])?;
                    left *= right;
                    rest = new_rest;
                }
                '/' => {
                    let (right, new_rest) = Self::parse_power(&rest[1..])?;
                    if right == 0.0 {
                        return None; // Division by zero
                    }
                    left /= right;
                    rest = new_rest;
                }
                '%' => {
                    let (right, new_rest) = Self::parse_power(&rest[1..])?;
                    left %= right;
                    rest = new_rest;
                }
                _ => break,
            }
        }

        Some((left, rest))
    }

    fn parse_power(expr: &str) -> Option<(f64, &str)> {
        let (base, rest) = Self::parse_unary(expr)?;

        if rest.starts_with("**") {
            let (exponent, new_rest) = Self::parse_power(&rest[2..])?;
            Some((base.powf(exponent), new_rest))
        } else {
            Some((base, rest))
        }
    }

    fn parse_unary(expr: &str) -> Option<(f64, &str)> {
        let expr = expr.trim_start();

        if expr.starts_with('-') {
            let (value, rest) = Self::parse_primary(&expr[1..])?;
            Some((-value, rest))
        } else if expr.starts_with('+') {
            Self::parse_primary(&expr[1..])
        } else {
            Self::parse_primary(expr)
        }
    }

    fn parse_primary(expr: &str) -> Option<(f64, &str)> {
        let expr = expr.trim_start();

        // Parentheses
        if expr.starts_with('(') {
            let (value, rest) = Self::parse_additive(&expr[1..])?;
            let rest = rest.trim_start();
            if rest.starts_with(')') {
                return Some((value, &rest[1..]));
            }
            return None;
        }

        // Functions
        for (func_name, func) in [
            ("sqrt", f64::sqrt as fn(f64) -> f64),
            ("sin", f64::sin),
            ("cos", f64::cos),
            ("tan", f64::tan),
            ("log", f64::log10),
            ("ln", f64::ln),
            ("abs", f64::abs),
            ("floor", f64::floor),
            ("ceil", f64::ceil),
            ("round", f64::round),
        ] {
            if expr.to_lowercase().starts_with(func_name) {
                let rest = &expr[func_name.len()..].trim_start();
                if rest.starts_with('(') {
                    let (arg, rest) = Self::parse_additive(&rest[1..])?;
                    let rest = rest.trim_start();
                    if rest.starts_with(')') {
                        return Some((func(arg), &rest[1..]));
                    }
                }
            }
        }

        // Constants
        if expr.to_lowercase().starts_with("pi") {
            return Some((std::f64::consts::PI, &expr[2..]));
        }
        if expr.to_lowercase().starts_with("e") && !expr[1..].starts_with(|c: char| c.is_alphabetic()) {
            return Some((std::f64::consts::E, &expr[1..]));
        }

        // Number
        Self::parse_number(expr)
    }

    fn parse_number(expr: &str) -> Option<(f64, &str)> {
        let expr = expr.trim_start();
        let mut end = 0;
        let mut has_dot = false;

        for c in expr.chars() {
            if c.is_ascii_digit() {
                end += 1;
            } else if c == '.' && !has_dot {
                has_dot = true;
                end += 1;
            } else {
                break;
            }
        }

        if end == 0 {
            return None;
        }

        let num_str = &expr[..end];
        let num: f64 = num_str.parse().ok()?;
        Some((num, &expr[end..]))
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let calc = Calculator::new();
        assert_eq!(calc.evaluate("2 + 2"), Some(4.0));
        assert_eq!(calc.evaluate("10 - 3"), Some(7.0));
        assert_eq!(calc.evaluate("4 * 5"), Some(20.0));
        assert_eq!(calc.evaluate("20 / 4"), Some(5.0));
    }

    #[test]
    fn test_order_of_operations() {
        let calc = Calculator::new();
        assert_eq!(calc.evaluate("2 + 3 * 4"), Some(14.0));
        assert_eq!(calc.evaluate("(2 + 3) * 4"), Some(20.0));
    }

    #[test]
    fn test_power() {
        let calc = Calculator::new();
        assert_eq!(calc.evaluate("2 ** 3"), Some(8.0));
        assert_eq!(calc.evaluate("2^3"), Some(8.0));
    }
}
