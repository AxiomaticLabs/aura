use crate::page::{Page, PAGE_SIZE};
use crate::StoreError;
use aura_security::symmetric::{self, KEY_SIZE};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

// Encrypted page size = PAGE_SIZE + NONCE_SIZE + TAG_SIZE
pub const ENCRYPTED_PAGE_SIZE: usize = PAGE_SIZE + symmetric::NONCE_SIZE + symmetric::TAG_SIZE;

pub struct Pager {
    file: File,
    total_pages: u32,
    master_key: [u8; KEY_SIZE],
}

impl Pager {
    pub fn open(path: impl AsRef<Path>, master_key: [u8; KEY_SIZE]) -> Result<Self, StoreError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;

        let len = file.metadata()?.len();
        let total_pages = (len / ENCRYPTED_PAGE_SIZE as u64) as u32;

        Ok(Self {
            file,
            total_pages,
            master_key,
        })
    }

    /// Writes a page to disk with transparent encryption
    pub fn write_page(&mut self, page: &Page) -> Result<(), StoreError> {
        let offset = page.id as u64 * ENCRYPTED_PAGE_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;

        // Convert struct to raw bytes safely
        let plaintext =
            unsafe { std::slice::from_raw_parts(page as *const Page as *const u8, PAGE_SIZE) };

        // Encrypt the page data
        let encrypted_data = symmetric::encrypt(plaintext, &self.master_key)
            .map_err(|_| StoreError::Tampered(page.id))?;

        // Write encrypted data to disk
        self.file.write_all(&encrypted_data)?;

        // Update total_pages if we wrote beyond the current end
        if page.id >= self.total_pages {
            self.total_pages = page.id + 1;
        }

        Ok(())
    }

    /// Reads a page from disk with transparent decryption
    pub fn read_page(&mut self, id: u32) -> Result<Page, StoreError> {
        if id >= self.total_pages {
            return Err(StoreError::PageNotFound(id));
        }

        let offset = id as u64 * ENCRYPTED_PAGE_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;

        // Read encrypted data from disk
        let mut encrypted_data = vec![0u8; ENCRYPTED_PAGE_SIZE];
        self.file.read_exact(&mut encrypted_data)?;

        // Decrypt the data
        let plaintext = symmetric::decrypt(&encrypted_data, &self.master_key)
            .map_err(|_| StoreError::Tampered(id))?;

        // Ensure decrypted data is exactly PAGE_SIZE
        if plaintext.len() != PAGE_SIZE {
            return Err(StoreError::Tampered(id));
        }

        // Convert back to Page struct safely
        let mut page: Page = unsafe { std::mem::zeroed() };
        unsafe {
            std::ptr::copy_nonoverlapping(
                plaintext.as_ptr(),
                &mut page as *mut Page as *mut u8,
                PAGE_SIZE,
            );
        }

        Ok(page)
    }

    /// Allocates a new empty page
    pub fn allocate_page(&mut self) -> u32 {
        let id = self.total_pages;
        self.total_pages += 1;
        id
    }
}
