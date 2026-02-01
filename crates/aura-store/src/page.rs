pub const PAGE_SIZE: usize = 4096;
pub const DATA_SIZE: usize = 3997; // Leave space for metadata

/// The physical representation of a block on disk.
/// This entire struct is what gets encrypted.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Page {
    pub id: u32,
    pub page_type: u8, // 1 = Data, 2 = Index
    pub used_space: u16,
    pub next_page: u32, // For linked lists of pages
    pub reserved: [u8; 88], // Padding to align headers
    pub data: [u8; DATA_SIZE], // The actual payload
}

impl Page {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            page_type: 1,
            used_space: 0,
            next_page: 0,
            reserved: [0; 88],
            data: [0; DATA_SIZE],
        }
    }
}