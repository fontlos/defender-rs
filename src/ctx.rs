use std::fs::File;
use std::io::{Read, Write};

#[repr(C, packed)]
pub struct Ctx {
    pub state: u8,
    pub verbose: u8,
    pub name: [u8; 129],
}

impl Ctx {
    pub fn default_with_name(name: &str) -> Self {
        // name_buf[128] = 0; // nullterm
        let mut name_buf = [0u8; 129];
        let bytes = name.as_bytes();
        let len = bytes.len().min(128);
        name_buf[..len].copy_from_slice(&bytes[..len]);
        Ctx {
            state: 1, // ON
            verbose: 0,
            name: name_buf,
        }
    }
    pub fn serialize(&self, path: &str) {
        let mut f = File::create(path).unwrap();
        let bytes = unsafe {
            std::slice::from_raw_parts(
                (self as *const Ctx) as *const u8,
                std::mem::size_of::<Ctx>(),
            )
        };
        f.write_all(bytes).unwrap();
    }
    pub fn deserialize(path: &str) -> Option<Self> {
        let mut f = File::open(path).ok()?;
        let mut buf = [0u8; std::mem::size_of::<Ctx>()];
        f.read_exact(&mut buf).ok()?;
        let ctx: Ctx = unsafe { std::ptr::read(buf.as_ptr() as *const _) };
        Some(ctx)
    }
    pub fn name_str(&self) -> String {
        let nul = self
            .name
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(self.name.len());
        String::from_utf8_lossy(&self.name[..nul]).to_string()
    }
}
