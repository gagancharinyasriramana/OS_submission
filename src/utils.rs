// Pure logic utilities for RustOS shell commands
// Compatible with both no_std and host unit tests
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, PartialEq, Eq)]
pub enum BcError {
    InvalidNumber,
    InvalidOperator,
    DivisionByZero,
    WrongArgumentCount,
}

pub fn eval_bc(args: &[&str]) -> Result<i64, BcError> {
    if args.len() != 3 {
        return Err(BcError::WrongArgumentCount);
    }

    let left = args[0].parse::<i64>().map_err(|_| BcError::InvalidNumber)?;
    let op = args[1];
    let right = args[2].parse::<i64>().map_err(|_| BcError::InvalidNumber)?;

    match op {
        "+" => Ok(left + right),
        "-" => Ok(left - right),
        "*" => Ok(left * right),
        "/" => {
            if right == 0 {
                Err(BcError::DivisionByZero)
            } else {
                Ok(left / right)
            }
        }
        "%" => {
            if right == 0 {
                Err(BcError::DivisionByZero)
            } else {
                Ok(left % right)
            }
        }
        _ => Err(BcError::InvalidOperator),
    }
}

pub fn cut_text(text: &str, start_col: usize, end_col: usize) -> String {
    let mut result = String::new();
    if start_col == 0 || start_col > end_col {
        return result;
    }

    let start_idx = start_col - 1;
    let end_idx = end_col;

    for line in text.lines() {
        let chars: Vec<char> = line.chars().collect();
        if start_idx < chars.len() {
            let actual_end = end_idx.min(chars.len());
            for &c in &chars[start_idx..actual_end] {
                result.push(c);
            }
        }
        result.push('\n');
    }
    result
}

pub fn head_text(text: &str, n: usize) -> String {
    let mut result = String::new();
    for (i, line) in text.lines().enumerate() {
        if i >= n {
            break;
        }
        result.push_str(line);
        result.push('\n');
    }
    result
}

pub fn tail_text(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut result = String::new();
    let skip_count = if lines.len() > n { lines.len() - n } else { 0 };
    for line in lines.iter().skip(skip_count) {
        result.push_str(line);
        result.push('\n');
    }
    result
}

#[derive(Debug, PartialEq, Eq)]
pub struct WcCount {
    pub lines: usize,
    pub words: usize,
    pub bytes: usize,
}

pub fn count_wc(text: &str) -> WcCount {
    let bytes = text.len();
    let lines = text.lines().count();
    let words = text.split_whitespace().count();

    WcCount { lines, words, bytes }
}

pub fn grab_lines(text: &str, query: &str) -> String {
    let mut result = String::new();
    for line in text.lines() {
        if line.contains(query) {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

// Days in months helper (handles leap years)
pub fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

// Tomohiko Sakamoto's algorithm to calculate day of week for y/m/d (0 = Sunday .. 6 = Saturday)
pub fn day_of_week(year: u16, month: u8, day: u8) -> u8 {
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut y = year;
    if month < 3 {
        y -= 1;
    }
    let m_idx = (month - 1) as usize;
    let dow = (y + y / 4 - y / 100 + y / 400 + t[m_idx] + day as u16) % 7;
    dow as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bc_arithmetic() {
        assert_eq!(eval_bc(&["10", "+", "20"]), Ok(30));
        assert_eq!(eval_bc(&["-15", "+", "5"]), Ok(-10));
        assert_eq!(eval_bc(&["20", "-", "7"]), Ok(13));
        assert_eq!(eval_bc(&["6", "*", "7"]), Ok(42));
        assert_eq!(eval_bc(&["100", "/", "4"]), Ok(25));
        assert_eq!(eval_bc(&["17", "%", "5"]), Ok(2));
        assert_eq!(eval_bc(&["10", "/", "0"]), Err(BcError::DivisionByZero));
        assert_eq!(eval_bc(&["10", "%", "0"]), Err(BcError::DivisionByZero));
        assert_eq!(eval_bc(&["10", "^", "2"]), Err(BcError::InvalidOperator));
    }

    #[test]
    fn test_cut() {
        let text = "abcdef\n123456\n";
        assert_eq!(cut_text(text, 2, 4), "bcd\n234\n");
    }

    #[test]
    fn test_head_and_tail() {
        let text = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n";
        assert_eq!(head_text(text, 3), "1\n2\n3\n");
        assert_eq!(tail_text(text, 2), "11\n12\n");
    }

    #[test]
    fn test_wc() {
        let text = "hello world\nthis is rustos\n";
        let wc = count_wc(text);
        assert_eq!(wc.lines, 2);
        assert_eq!(wc.words, 5);
        assert_eq!(wc.bytes, 27);
    }

    #[test]
    fn test_grab() {
        let text = "apple pie\nbanana bread\ncherry pie\n";
        assert_eq!(grab_lines(text, "pie"), "apple pie\ncherry pie\n");
    }
}
