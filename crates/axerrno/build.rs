use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Result, Write};
use std::path::Path;

macro_rules! template {
    () => {
        "\
// Generated by build.rs, DO NOT edit

/// Linux specific error codes defined in `errno.h`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinuxError {{
{0}\
}}

impl LinuxError {{
    /// Returns the error description.
    pub const fn as_str(&self) -> &'static str {{
        use self::LinuxError::*;
        match self {{
{1}        }}
    }}

    /// Returns the error code value in `i32`.
    pub const fn code(self) -> i32 {{
        self as i32
    }}
}}
"
    };
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    gen_linux_errno(&Path::new(&out_dir).join("linux_errno.rs")).unwrap();
}

fn gen_linux_errno(dest_path: &Path) -> Result<()> {
    let mut enum_define = Vec::new();
    let mut detail_info = Vec::new();

    let file = File::open("src/errno.h")?;
    for line in BufReader::new(file).lines().map_while(Result::ok) {
        if line.starts_with("#define") {
            let mut iter = line.split_whitespace();
            if let Some(name) = iter.nth(1) {
                if let Some(num) = iter.next() {
                    let description = if let Some(pos) = line.find("/* ") {
                        String::from(line[pos + 3..].trim_end_matches(" */"))
                    } else {
                        format!("Error number {num}")
                    };
                    writeln!(enum_define, "    /// {description}\n    {name} = {num},")?;
                    writeln!(detail_info, "            {name} => \"{description}\",")?;
                }
            }
        }
    }

    fs::write(
        dest_path,
        format!(
            template!(),
            String::from_utf8_lossy(&enum_define),
            String::from_utf8_lossy(&detail_info)
        ),
    )?;

    Ok(())
}