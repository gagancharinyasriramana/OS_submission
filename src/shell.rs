// RustOS Interactive Command Shell and Line Editor
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use crate::fs::FS;
use crate::keyboard::{get_key, Key};
use crate::ports::outb;
use crate::rtc;
use crate::timer;
use crate::utils::{count_wc, cut_text, day_of_week, days_in_month, eval_bc, grab_lines, head_text, tail_text, BcError};
use crate::vga::{Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH, WRITER};
use crate::{print, println};

const HISTORY_CAPACITY: usize = 32;

pub struct Shell {
    pub history: Vec<String>,
    pub running: bool,
}

impl Shell {
    pub const fn new() -> Self {
        Shell {
            history: Vec::new(),
            running: true,
        }
    }

    pub fn draw_status_bar(&self, app_name: &str) {
        let dt = rtc::read_datetime();
        let time_str = alloc::format!(
            " RustOS 0.1.0 | App: {:<14} | {:02}:{:02}:{:02} {} {:04}-{:02}-{:02} ",
            app_name,
            dt.hour,
            dt.minute,
            dt.second,
            rtc::weekday_name(dt.weekday),
            dt.year,
            dt.month,
            dt.day
        );

        let color = ColorCode::new(Color::Black, Color::LightGray);
        unsafe {
            for col in 0..BUFFER_WIDTH {
                let ch = if col < time_str.len() {
                    time_str.as_bytes()[col]
                } else {
                    b' '
                };
                WRITER.write_char_at(col, BUFFER_HEIGHT - 1, ch, color);
            }
        }
    }

    pub fn read_line(&mut self, prompt: &str) -> String {
        self.draw_status_bar("Shell");
        print!("{}", prompt);

        let mut line = String::new();
        let mut history_idx = self.history.len();

        loop {
            match get_key() {
                Key::Char(c) => {
                    line.push(c);
                    print!("{}", c);
                }
                Key::Backspace => {
                    if !line.is_empty() {
                        line.pop();
                        unsafe {
                            WRITER.write_byte(0x08);
                        }
                    }
                }
                Key::Enter => {
                    println!();
                    break;
                }
                Key::Up => {
                    // History recall earlier
                    if !self.history.is_empty() && history_idx > 0 {
                        history_idx -= 1;
                        // Erase current line on screen
                        while !line.is_empty() {
                            line.pop();
                            unsafe {
                                WRITER.write_byte(0x08);
                            }
                        }
                        line = self.history[history_idx].clone();
                        print!("{}", line);
                    }
                }
                Key::Down => {
                    // History recall later
                    if history_idx + 1 < self.history.len() {
                        history_idx += 1;
                        while !line.is_empty() {
                            line.pop();
                            unsafe {
                                WRITER.write_byte(0x08);
                            }
                        }
                        line = self.history[history_idx].clone();
                        print!("{}", line);
                    } else if history_idx + 1 == self.history.len() {
                        history_idx += 1;
                        while !line.is_empty() {
                            line.pop();
                            unsafe {
                                WRITER.write_byte(0x08);
                            }
                        }
                    }
                }
                Key::Tab => {
                    // Tab auto-complete files or commands
                    self.tab_complete(&mut line);
                }
                Key::Esc => {
                    // Return from shell to desktop menu
                    return String::from("games");
                }
                _ => {}
            }
        }

        let trimmed = line.trim();
        if !trimmed.is_empty() {
            if self.history.is_empty() || self.history.last().map(|s| s.as_str()) != Some(trimmed) {
                if self.history.len() >= HISTORY_CAPACITY {
                    self.history.remove(0);
                }
                self.history.push(String::from(trimmed));
            }
        }

        String::from(trimmed)
    }

    fn tab_complete(&self, line: &mut String) {
        let commands = [
            "help", "clear", "reboot", "shutdown", "date", "cal", "echo", "ls", "lp",
            "whoami", "uname", "hostname", "uptime", "bc", "cat", "grab", "rm", "touch",
            "cp", "mv", "cut", "head", "tail", "chmod", "wc", "vi", "games", "research", "deadlock"
        ];

        let trimmed = line.trim();
        for cmd in commands.iter() {
            if cmd.starts_with(trimmed) && cmd.len() > trimmed.len() {
                let rest = &cmd[trimmed.len()..];
                line.push_str(rest);
                print!("{}", rest);
                break;
            }
        }
    }

    pub fn print_error(&self, msg: &str) {
        unsafe {
            WRITER.set_color(Color::LightRed, Color::Black);
        }
        println!("Error: {}", msg);
        unsafe {
            WRITER.set_color(Color::White, Color::Black);
        }
    }

    pub fn execute(&mut self, input: &str) -> ShellAction {
        if input.is_empty() {
            return ShellAction::Continue;
        }

        // Handle redirection: command ... > file
        if let Some(pos) = input.find('>') {
            let left_part = input[..pos].trim();
            let right_part = input[pos + 1..].trim();

            if right_part.is_empty() {
                self.print_error("Missing destination file for '>' redirection");
                return ShellAction::Continue;
            }

            if left_part.starts_with("echo ") {
                let text = &left_part[5..];
                let content = alloc::format!("{}\n", text);
                unsafe {
                    if let Err(e) = FS.write_file(right_part, content.as_bytes()) {
                        self.print_error(e);
                    }
                }
                return ShellAction::Continue;
            } else {
                self.print_error("Redirection '>' is currently supported for 'echo <text> > <file>'");
                return ShellAction::Continue;
            }
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts[0];
        let args = &parts[1..];

        match cmd {
            "help" => self.cmd_help(),
            "clear" => self.cmd_clear(),
            "reboot" => self.cmd_reboot(),
            "shutdown" => self.cmd_shutdown(),
            "date" => self.cmd_date(),
            "cal" => self.cmd_cal(),
            "echo" => self.cmd_echo(args),
            "ls" => self.cmd_ls(),
            "lp" => self.cmd_lp(args),
            "whoami" => println!("root"),
            "uname" => println!("RustOS 0.1.0 i686"),
            "hostname" => println!("rustos"),
            "uptime" => self.cmd_uptime(),
            "bc" => self.cmd_bc(args),
            "cat" => self.cmd_cat(args),
            "grab" => self.cmd_grab(args),
            "rm" => self.cmd_rm(args),
            "touch" => self.cmd_touch(args),
            "cp" => self.cmd_cp(args),
            "mv" => self.cmd_mv(args),
            "cut" => self.cmd_cut(args),
            "head" => self.cmd_head(args),
            "tail" => self.cmd_tail(args),
            "chmod" => self.cmd_chmod(args),
            "wc" => self.cmd_wc(args),
            "vi" => self.cmd_vi(args),
            "games" => return ShellAction::OpenMenu,
            "research" | "deadlock" => return ShellAction::OpenResearch,
            _ => {
                self.print_error(&alloc::format!("unknown command '{}'. Type 'help' for a list of commands.", cmd));
            }
        }

        ShellAction::Continue
    }

    fn cmd_help(&self) {
        println!("Available RustOS Commands:");
        println!("  help                 - Display this command list with descriptions");
        println!("  clear                - Clear the screen");
        println!("  reboot               - Reboot the machine via keyboard controller");
        println!("  shutdown             - Halt the operating system cleanly");
        println!("  date                 - Display current date and time from RTC");
        println!("  cal                  - Display calendar for current month");
        println!("  echo <text>          - Print text to screen or file (with > file)");
        println!("  ls                   - List files with permission, size, and name");
        println!("  lp <file>            - Line printer simulation (prints to screen)");
        println!("  whoami               - Print current user identity (root)");
        println!("  uname                - Print operating system name and architecture");
        println!("  hostname             - Print system host name (rustos)");
        println!("  uptime               - Print system uptime from PIT tick counter");
        println!("  bc <n> <op> <n>      - Integer calculator (+, -, *, /, %)");
        println!("  cat <file>           - Display contents of a file");
        println!("  grab <word> <file>   - Search and print lines matching word");
        println!("  rm <file>            - Remove/delete a file");
        println!("  touch <file>         - Create an empty file");
        println!("  cp <src> <dst>       - Copy a file");
        println!("  mv <src> <dst>       - Rename a file");
        println!("  cut <start> <end> <f>- Cut character columns from each line");
        println!("  head <file> [n]      - Display first n lines of file (default 10)");
        println!("  tail <file> [n]      - Display last n lines of file (default 10)");
        println!("  chmod <6|4> <file>   - Change file permission (6: rw-, 4: r--)");
        println!("  wc <file>            - Count lines, words, and bytes in file");
        println!("  vi <file>            - Simple line editor (:wq saves and exits)");
        println!("  games                - Open desktop games and application menu");
        println!("  research / deadlock  - Open Workload-Aware CPU Scheduling & Deadlock Lab");
    }

    fn cmd_clear(&self) {
        unsafe {
            WRITER.clear_screen();
        }
    }

    fn cmd_reboot(&self) {
        println!("Rebooting RustOS...");
        timer::sleep_ms(200);
        unsafe {
            // Pulse the reset line using the keyboard controller (port 0x64 with 0xFE)
            outb(0x64, 0xFE);
            loop {
                core::arch::asm!("hlt");
            }
        }
    }

    fn cmd_shutdown(&self) {
        println!("\nSystem halted. You can now power off the virtual machine.");
        loop {
            unsafe {
                core::arch::asm!("cli; hlt");
            }
        }
    }

    fn cmd_date(&self) {
        let dt = rtc::read_datetime();
        println!(
            "{}, {} {:02}, {:04} {:02}:{:02}:{:02} UTC",
            rtc::weekday_name(dt.weekday),
            rtc::month_name(dt.month),
            dt.day,
            dt.year,
            dt.hour,
            dt.minute,
            dt.second
        );
    }

    fn cmd_cal(&self) {
        let dt = rtc::read_datetime();
        let month_name = rtc::month_name(dt.month);
        println!("     {} {:04}", month_name, dt.year);
        println!("Su Mo Tu We Th Fr Sa");

        let first_dow = day_of_week(dt.year, dt.month, 1);
        let total_days = days_in_month(dt.year, dt.month);

        let mut col = 0;
        for _ in 0..first_dow {
            print!("   ");
            col += 1;
        }

        for day in 1..=total_days {
            if day == dt.day {
                unsafe {
                    WRITER.set_color(Color::LightGreen, Color::Black);
                }
                print!("{:2} ", day);
                unsafe {
                    WRITER.set_color(Color::White, Color::Black);
                }
            } else {
                print!("{:2} ", day);
            }
            col += 1;
            if col == 7 {
                println!();
                col = 0;
            }
        }
        if col != 0 {
            println!();
        }
    }

    fn cmd_echo(&self, args: &[&str]) {
        let text = args.join(" ");
        println!("{}", text);
    }

    fn cmd_ls(&self) {
        unsafe {
            let files = FS.list_files();
            println!("{:<5} {:>6}  {}", "PERM", "SIZE", "NAME");
            println!("-----------------------------");
            for f in files {
                println!("{:<5} {:>6}  {}", f.permission, f.size, f.name);
            }
        }
    }

    fn cmd_lp(&self, args: &[&str]) {
        if args.is_empty() {
            self.print_error("Usage: lp <file>");
            return;
        }
        let filename = args[0];
        unsafe {
            match FS.read_file(filename) {
                Ok(content) => {
                    println!("[lp] no printer attached; printing to screen:\n");
                    print!("{}", content);
                }
                Err(e) => self.print_error(e),
            }
        }
    }

    fn cmd_uptime(&self) {
        let ticks = timer::get_ticks();
        let total_secs = ticks / 100;
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        let secs = total_secs % 60;
        println!("uptime: {:02}:{:02}:{:02} ({} ticks)", hours, mins, secs, ticks);
    }

    fn cmd_bc(&self, args: &[&str]) {
        match eval_bc(args) {
            Ok(val) => println!("{}", val),
            Err(BcError::WrongArgumentCount) => self.print_error("Usage: bc <number> <operator> <number>\nSupported operators: + - * / %"),
            Err(BcError::DivisionByZero) => self.print_error("Division by zero"),
            Err(BcError::InvalidNumber) => self.print_error("Invalid integer operand"),
            Err(BcError::InvalidOperator) => self.print_error("Invalid operator: use +, -, *, /, or %"),
        }
    }

    fn cmd_cat(&self, args: &[&str]) {
        if args.is_empty() {
            self.print_error("Usage: cat <file>");
            return;
        }
        unsafe {
            match FS.read_file(args[0]) {
                Ok(content) => print!("{}", content),
                Err(e) => self.print_error(e),
            }
        }
    }

    fn cmd_grab(&self, args: &[&str]) {
        if args.len() < 2 {
            self.print_error("Usage: grab <word> <file>");
            return;
        }
        let word = args[0];
        let file = args[1];
        unsafe {
            match FS.read_file(file) {
                Ok(content) => {
                    let matched = grab_lines(content, word);
                    print!("{}", matched);
                }
                Err(e) => self.print_error(e),
            }
        }
    }

    fn cmd_rm(&self, args: &[&str]) {
        if args.is_empty() {
            self.print_error("Usage: rm <file>");
            return;
        }
        unsafe {
            if let Err(e) = FS.delete_file(args[0]) {
                self.print_error(e);
            }
        }
    }

    fn cmd_touch(&self, args: &[&str]) {
        if args.is_empty() {
            self.print_error("Usage: touch <file>");
            return;
        }
        unsafe {
            if FS.find_file_idx(args[0]).is_none() {
                if let Err(e) = FS.create_file(args[0], b"", 6) {
                    self.print_error(e);
                }
            }
        }
    }

    fn cmd_cp(&self, args: &[&str]) {
        if args.len() < 2 {
            self.print_error("Usage: cp <source> <destination>");
            return;
        }
        unsafe {
            if let Err(e) = FS.copy_file(args[0], args[1]) {
                self.print_error(e);
            }
        }
    }

    fn cmd_mv(&self, args: &[&str]) {
        if args.len() < 2 {
            self.print_error("Usage: mv <source> <destination>");
            return;
        }
        unsafe {
            if let Err(e) = FS.rename_file(args[0], args[1]) {
                self.print_error(e);
            }
        }
    }

    fn cmd_cut(&self, args: &[&str]) {
        if args.len() < 3 {
            self.print_error("Usage: cut <start_col> <end_col> <file>");
            return;
        }
        let start = match args[0].parse::<usize>() {
            Ok(n) if n > 0 => n,
            _ => {
                self.print_error("Start column must be a positive integer (>= 1)");
                return;
            }
        };
        let end = match args[1].parse::<usize>() {
            Ok(n) if n >= start => n,
            _ => {
                self.print_error("End column must be >= start column");
                return;
            }
        };
        let file = args[2];
        unsafe {
            match FS.read_file(file) {
                Ok(content) => {
                    let res = cut_text(content, start, end);
                    print!("{}", res);
                }
                Err(e) => self.print_error(e),
            }
        }
    }

    fn cmd_head(&self, args: &[&str]) {
        if args.is_empty() {
            self.print_error("Usage: head <file> [n]");
            return;
        }
        let file = args[0];
        let n = if args.len() >= 2 {
            args[1].parse::<usize>().unwrap_or(10)
        } else {
            10
        };
        unsafe {
            match FS.read_file(file) {
                Ok(content) => print!("{}", head_text(content, n)),
                Err(e) => self.print_error(e),
            }
        }
    }

    fn cmd_tail(&self, args: &[&str]) {
        if args.is_empty() {
            self.print_error("Usage: tail <file> [n]");
            return;
        }
        let file = args[0];
        let n = if args.len() >= 2 {
            args[1].parse::<usize>().unwrap_or(10)
        } else {
            10
        };
        unsafe {
            match FS.read_file(file) {
                Ok(content) => print!("{}", tail_text(content, n)),
                Err(e) => self.print_error(e),
            }
        }
    }

    fn cmd_chmod(&self, args: &[&str]) {
        if args.len() < 2 {
            self.print_error("Usage: chmod <6|4> <file>\n  6 = rw- (writable), 4 = r-- (read-only)");
            return;
        }
        let perm = match args[0] {
            "6" => 6,
            "4" => 4,
            _ => {
                self.print_error("Invalid permission mode: use 6 or 4");
                return;
            }
        };
        unsafe {
            if let Err(e) = FS.chmod(args[1], perm) {
                self.print_error(e);
            }
        }
    }

    fn cmd_wc(&self, args: &[&str]) {
        if args.is_empty() {
            self.print_error("Usage: wc <file>");
            return;
        }
        unsafe {
            match FS.read_file(args[0]) {
                Ok(content) => {
                    let wc = count_wc(content);
                    println!("{:>6} {:>6} {:>6} {}", wc.lines, wc.words, wc.bytes, args[0]);
                }
                Err(e) => self.print_error(e),
            }
        }
    }

    fn cmd_vi(&mut self, args: &[&str]) {
        if args.is_empty() {
            self.print_error("Usage: vi <file>");
            return;
        }
        let filename = args[0];
        let mut buffer = String::new();

        unsafe {
            if let Ok(existing) = FS.read_file(filename) {
                buffer.push_str(existing);
            }
        }

        unsafe {
            WRITER.clear_screen();
            WRITER.set_color(Color::White, Color::Black);
        }
        println!("=== RustOS vi editor: {} ===", filename);
        println!("Type lines of text to append. Type ':wq' on an empty line to save and exit.\n");
        if !buffer.is_empty() {
            print!("{}", buffer);
        }

        loop {
            self.draw_status_bar(&alloc::format!("vi: {}", filename));
            let mut line = String::new();
            loop {
                match get_key() {
                    Key::Char(c) => {
                        line.push(c);
                        print!("{}", c);
                    }
                    Key::Backspace => {
                        if !line.is_empty() {
                            line.pop();
                            unsafe {
                                WRITER.write_byte(0x08);
                            }
                        }
                    }
                    Key::Enter => {
                        println!();
                        break;
                    }
                    Key::Esc => {
                        // Exit without save
                        return;
                    }
                    _ => {}
                }
            }

            if line.trim() == ":wq" {
                unsafe {
                    if let Err(e) = FS.write_file(filename, buffer.as_bytes()) {
                        self.print_error(e);
                    } else {
                        println!("\"{}\" saved.", filename);
                    }
                }
                break;
            } else {
                buffer.push_str(&line);
                buffer.push('\n');
            }
        }
    }
}

pub enum ShellAction {
    Continue,
    OpenMenu,
    OpenResearch,
}
