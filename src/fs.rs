// In-memory File System for RustOS
// Max 32 files, each up to 2048 bytes, with permissions (6 = rw-, 4 = r--)
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

pub const MAX_FILES: usize = 32;
pub const MAX_FILE_SIZE: usize = 2048;
pub const MAX_NAME_LEN: usize = 24;

#[derive(Clone, Copy)]
pub struct FileEntry {
    pub name: [u8; MAX_NAME_LEN],
    pub name_len: usize,
    pub permission: u8, // 6 = rw-, 4 = r--
    pub len: usize,
    pub data: [u8; MAX_FILE_SIZE],
    pub in_use: bool,
}

impl FileEntry {
    pub const fn empty() -> Self {
        FileEntry {
            name: [0; MAX_NAME_LEN],
            name_len: 0,
            permission: 6,
            len: 0,
            data: [0; MAX_FILE_SIZE],
            in_use: false,
        }
    }

    pub fn get_name(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("?")
    }

    pub fn get_content(&self) -> &str {
        core::str::from_utf8(&self.data[..self.len]).unwrap_or("")
    }
}

pub struct FileSystem {
    pub files: [FileEntry; MAX_FILES],
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub permission: String, // "rw-" or "r--"
    pub size: usize,
}

impl FileSystem {
    pub const fn new() -> Self {
        FileSystem {
            files: [FileEntry::empty(); MAX_FILES],
        }
    }

    pub fn init_defaults(&mut self) {
        let readme = "Welcome to RustOS 0.1.0!\n\
                      A 32-bit x86 educational operating system written from scratch in Rust.\n\
                      Features:\n\
                      - Interactive command shell with 25+ standard commands\n\
                      - Playable desktop games: Snake, Snakes and Ladders, Car Dodge\n\
                      - Workload-Aware CPU Scheduling Research Lab\n\
                      - Deadlock Detection and Resource Allocation Lab\n\
                      Type 'help' for shell commands or 'games' for the desktop menu.\n";

        let notes = "RustOS Architecture & Design Notes:\n\
                     - Target: 32-bit x86 protected mode (i686-pc-none)\n\
                     - Bootloader: GNU GRUB 2 via Multiboot specification\n\
                     - Hardware Drivers: VGA 80x25, COM1 UART, 8254 PIT, 8259A PIC, PS/2 Keyboard, CMOS RTC\n\
                     - Memory: 4 MiB linked-list heap with coalescing\n\
                     - Storage: In-memory virtual file table\n";

        let todo = "RustOS Future Roadmap / To-Do List:\n\
                    [ ] PS/2 Mouse cursor integration\n\
                    [ ] FAT16/32 Hard drive persistence\n\
                    [ ] Preemptive kernel multitasking with TSS context switching\n\
                    [ ] TCP/IP networking stack with RTL8139 / E1000 NIC driver\n\
                    [ ] Sound Blaster 16 audio driver\n";

        let about = "RustOS Educational Operating System\n\
                     Version: 0.1.0 (2026 Edition)\n\
                     Kernel: 32-bit x86 Protected Mode (no_std Rust)\n\
                     Author: Senior Systems Engineering Team\n\
                     Research: Workload-Aware CPU Scheduling & Deadlock Analysis\n";

        let _ = self.create_file("readme.txt", readme.as_bytes(), 4); // Read-only
        let _ = self.create_file("notes.txt", notes.as_bytes(), 6);
        let _ = self.create_file("todo.txt", todo.as_bytes(), 6);
        let _ = self.create_file("about.txt", about.as_bytes(), 4);  // Read-only
    }

    pub fn find_file_idx(&self, name: &str) -> Option<usize> {
        for (i, file) in self.files.iter().enumerate() {
            if file.in_use && file.get_name() == name {
                return Some(i);
            }
        }
        None
    }

    pub fn create_file(&mut self, name: &str, content: &[u8], perm: u8) -> Result<(), &'static str> {
        if name.is_empty() || name.len() > MAX_NAME_LEN {
            return Err("File name must be 1-24 characters");
        }
        if content.len() > MAX_FILE_SIZE {
            return Err("File content exceeds 2048 bytes limit");
        }
        if self.find_file_idx(name).is_some() {
            return Err("File already exists");
        }

        for file in self.files.iter_mut() {
            if !file.in_use {
                file.in_use = true;
                file.name_len = name.len();
                file.name[..name.len()].copy_from_slice(name.as_bytes());
                file.permission = perm;
                file.len = content.len();
                file.data[..content.len()].copy_from_slice(content);
                return Ok(());
            }
        }
        Err("File table is full (max 32 files)")
    }

    pub fn write_file(&mut self, name: &str, content: &[u8]) -> Result<(), &'static str> {
        if content.len() > MAX_FILE_SIZE {
            return Err("File content exceeds 2048 bytes limit");
        }

        if let Some(idx) = self.find_file_idx(name) {
            let file = &mut self.files[idx];
            if file.permission == 4 {
                return Err("Permission denied: file is read-only");
            }
            file.len = content.len();
            file.data[..content.len()].copy_from_slice(content);
            Ok(())
        } else {
            self.create_file(name, content, 6)
        }
    }

    pub fn read_file(&self, name: &str) -> Result<&str, &'static str> {
        if let Some(idx) = self.find_file_idx(name) {
            Ok(self.files[idx].get_content())
        } else {
            Err("No such file or directory")
        }
    }

    pub fn delete_file(&mut self, name: &str) -> Result<(), &'static str> {
        if let Some(idx) = self.find_file_idx(name) {
            let file = &mut self.files[idx];
            if file.permission == 4 {
                return Err("Permission denied: file is read-only");
            }
            file.in_use = false;
            file.name_len = 0;
            file.len = 0;
            Ok(())
        } else {
            Err("No such file or directory")
        }
    }

    pub fn rename_file(&mut self, src: &str, dst: &str) -> Result<(), &'static str> {
        if dst.is_empty() || dst.len() > MAX_NAME_LEN {
            return Err("Destination file name must be 1-24 characters");
        }
        if self.find_file_idx(dst).is_some() {
            return Err("Destination file already exists");
        }
        if let Some(src_idx) = self.find_file_idx(src) {
            let file = &mut self.files[src_idx];
            if file.permission == 4 {
                return Err("Permission denied: source file is read-only");
            }
            file.name_len = dst.len();
            file.name[..dst.len()].copy_from_slice(dst.as_bytes());
            Ok(())
        } else {
            Err("Source file not found")
        }
    }

    pub fn copy_file(&mut self, src: &str, dst: &str) -> Result<(), &'static str> {
        if dst.is_empty() || dst.len() > MAX_NAME_LEN {
            return Err("Destination file name must be 1-24 characters");
        }
        if self.find_file_idx(dst).is_some() {
            return Err("Destination file already exists");
        }
        if let Some(src_idx) = self.find_file_idx(src) {
            let src_file = self.files[src_idx];
            let len = src_file.len;
            let perm = src_file.permission;
            let mut data = [0u8; MAX_FILE_SIZE];
            data[..len].copy_from_slice(&src_file.data[..len]);

            for file in self.files.iter_mut() {
                if !file.in_use {
                    file.in_use = true;
                    file.name_len = dst.len();
                    file.name[..dst.len()].copy_from_slice(dst.as_bytes());
                    file.permission = perm;
                    file.len = len;
                    file.data[..len].copy_from_slice(&data[..len]);
                    return Ok(());
                }
            }
            Err("File table is full")
        } else {
            Err("Source file not found")
        }
    }

    pub fn chmod(&mut self, name: &str, perm: u8) -> Result<(), &'static str> {
        if perm != 4 && perm != 6 {
            return Err("Invalid mode: use 6 (rw-) or 4 (r--)");
        }
        if let Some(idx) = self.find_file_idx(name) {
            self.files[idx].permission = perm;
            Ok(())
        } else {
            Err("No such file or directory")
        }
    }

    pub fn list_files(&self) -> Vec<FileInfo> {
        let mut list = Vec::new();
        for file in self.files.iter() {
            if file.in_use {
                let perm_str = if file.permission == 6 {
                    String::from("rw-")
                } else {
                    String::from("r--")
                };
                list.push(FileInfo {
                    name: String::from(file.get_name()),
                    permission: perm_str,
                    size: file.len,
                });
            }
        }
        list
    }
}

pub static mut FS: FileSystem = FileSystem::new();
