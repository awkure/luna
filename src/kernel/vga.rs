// -*- mode: rust; -*-

extern crate spin;

use spin::Mutex;
use core::{ fmt, ptr::Unique };

#[cfg(feature = "use_spin")]
pub static WRITER : Mutex<Writer> = 
    Mutex::new(Writer::new(VGAConfig::new()));

#[derive(Debug, Default)]
pub struct VGAConfig {
    color : VGAColor,
    tab_width  : u8,
}

impl VGAConfig {
    const fn new() -> Self {
        VGAConfig {
            color : VGAColor::new(Color::LightGray, Color::Black),
            tab_width  : 4,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Color {
      Black
    , Blue
    , Green
    , Cyan
    , Red
    , Magenta
    , Brown
    , LightGray
    , DarkGray
    , LightBlue
    , LightGreen
    , LightCyan
    , LightRed
    , Pink
    , Yellow
    , White
}


#[derive(Debug, Clone, Copy, Default)]
struct VGAColor(u8);

impl VGAColor {
    const fn new(fg : Color, bg : Color) -> VGAColor {
        VGAColor((bg as u8) << 4 | (fg as u8))
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct ScreenChar {
    char  : u8,
    color : VGAColor,
}

const BUF_HEIGHT : usize = 25;
const BUF_WIDTH  : usize = 80;

struct Buffer {
    chars: [[ScreenChar ; BUF_WIDTH] ; BUF_HEIGHT],
}

pub struct Writer {
    cur_cmn : usize,
    config  : VGAConfig,
    buffer  : Unique<Buffer>,
}

impl Writer {
    const fn new(config : VGAConfig) -> Writer {
        Writer {
            buffer  : unsafe { Unique::new_unchecked(0xb8000 as *mut _) },
            cur_cmn : 0,
            config,
        }
    }

    pub fn write_byte(&mut self, byte : u8) {
        match byte {
            b'\n' => self.newline(),
            b'\t' => self.tabulate(),
            byte  => {
                if self.cur_cmn >= BUF_WIDTH {
                    self.newline();
                }

                self.buffer().chars[BUF_HEIGHT-1][self.cur_cmn] =
                    ScreenChar {
                        char  : byte,
                        color : self.config.color,
                    };

                self.cur_cmn += 1;
            }
        }
    }

    pub fn write_str(&mut self, s : &str) {
        s.bytes().for_each(|b| self.write_byte(b));
    }

    fn buffer(&mut self) -> &mut Buffer {
        unsafe{ self.buffer.as_mut() }
    }

    fn tabulate(&mut self) {
        let tab_width = self.config.tab_width as usize;

        let col = 
            if self.cur_cmn < tab_width + 1
            || self.cur_cmn % (tab_width + 1) == 0 { 
                tab_width * 2 + 1
            } else { 
                self.cur_cmn 
            };
        
        for _ in 0..col % (tab_width + 1) {
            self.buffer().chars[BUF_HEIGHT-1][col] =  
                ScreenChar { 
                    char  : b' ', 
                    color : self.config.color,
                };
        }

        self.cur_cmn += col % (tab_width + 1);
    }

    fn newline(&mut self) {
        ( 1..BUF_HEIGHT ).for_each(|row| 
        ( 0..BUF_WIDTH  ).for_each(|col| {
            self.buffer().chars[row-1][col] = 
                self.buffer().chars[row][col];
        }));

        self.clear_row(BUF_HEIGHT-1);
        self.cur_cmn = 0;
    }

    fn clear_row(&mut self, row: usize) {
        (0..BUF_WIDTH).for_each(|col| self.buffer().chars[row][col] = 
            ScreenChar {
                char  : b' ',
                color : self.config.color,
            });
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        s.bytes().for_each(|b| self.write_byte(b));
        Ok(())
    }
}

pub fn clear_screen() {
    (0..BUF_HEIGHT).for_each(|_| println!(""));
}

pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
