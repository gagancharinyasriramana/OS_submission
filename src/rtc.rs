// CMOS Real-Time Clock (RTC) driver
use crate::ports::{inb, outb};

const CMOS_ADDRESS: u16 = 0x70;
const CMOS_DATA: u16 = 0x71;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub weekday: u8, // 1 = Sun, 7 = Sat
}

unsafe fn get_update_in_progress_flag() -> bool {
    outb(CMOS_ADDRESS, 0x0A);
    (inb(CMOS_DATA) & 0x80) != 0
}

unsafe fn read_register(reg: u8) -> u8 {
    outb(CMOS_ADDRESS, reg);
    inb(CMOS_DATA)
}

fn bcd_to_bin(val: u8) -> u8 {
    (val & 0x0F) + ((val / 16) * 10)
}

pub fn read_datetime() -> DateTime {
    unsafe {
        // Wait until RTC is not updating
        while get_update_in_progress_flag() {}

        let mut second = read_register(0x00);
        let mut minute = read_register(0x02);
        let mut hour = read_register(0x04);
        let weekday = read_register(0x06);
        let mut day = read_register(0x07);
        let mut month = read_register(0x08);
        let mut year = read_register(0x09) as u16;

        // Double check read to avoid inconsistent read during RTC update transition
        loop {
            while get_update_in_progress_flag() {}
            let s2 = read_register(0x00);
            let m2 = read_register(0x02);
            let h2 = read_register(0x04);
            let d2 = read_register(0x07);
            let mo2 = read_register(0x08);
            let y2 = read_register(0x09) as u16;

            if second == s2 && minute == m2 && hour == h2 && day == d2 && month == mo2 && year == y2 {
                break;
            }
            second = s2;
            minute = m2;
            hour = h2;
            day = d2;
            month = mo2;
            year = y2;
        }

        let register_b = read_register(0x0B);

        // Convert BCD to binary values if necessary
        let is_bcd = (register_b & 0x04) == 0;
        if is_bcd {
            second = bcd_to_bin(second);
            minute = bcd_to_bin(minute);
            hour = bcd_to_bin(hour & 0x7F) | (hour & 0x80);
            day = bcd_to_bin(day);
            month = bcd_to_bin(month);
            year = bcd_to_bin(year as u8) as u16;
        }

        // Convert 12-hour clock to 24-hour clock if necessary
        let is_24hr = (register_b & 0x02) != 0;
        if !is_24hr && (hour & 0x80) != 0 {
            hour = ((hour & 0x7F) + 12) % 24;
        }

        // Calculate 4-digit year (standard century assumption 2000+)
        year += 2000;

        DateTime {
            year,
            month,
            day,
            hour,
            minute,
            second,
            weekday,
        }
    }
}

pub fn weekday_name(weekday: u8) -> &'static str {
    match weekday {
        1 => "Sunday",
        2 => "Monday",
        3 => "Tuesday",
        4 => "Wednesday",
        5 => "Thursday",
        6 => "Friday",
        7 => "Saturday",
        _ => "Unknown",
    }
}

pub fn month_name(month: u8) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}
