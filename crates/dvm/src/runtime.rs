// dustrun/crates/dvm/src/runtime.rs
//
// DPL v0.2 Runtime Support
//
// This module provides runtime support functions for compiled Dust programs,
// including memory management, string operations, and error handling.

use std::alloc::{GlobalAlloc, Layout, System};
use std::ptr;

// ─────────────────────────────────────────────────────────────────────────────
// Heap Allocator
// ─────────────────────────────────────────────────────────────────────────────

/// Global heap allocator using the system allocator
struct HeapAllocator;

unsafe impl GlobalAlloc for HeapAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        System.alloc(layout)
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if new_size > layout.size() {
            // Allocate new block and copy data
            let new_ptr = System.alloc(Layout::from_size_align_unchecked(new_size, layout.align()));
            if !new_ptr.is_null() {
                // Copy existing data
                ptr::copy_nonoverlapping(ptr, new_ptr, layout.size());
                System.dealloc(ptr, layout);
            }
            new_ptr
        } else {
            // Can use existing block
            ptr
        }
    }
}

#[global_allocator]
static HEAP: HeapAllocator = HeapAllocator;

// ─────────────────────────────────────────────────────────────────────────────
// Memory Operations
// ─────────────────────────────────────────────────────────────────────────────

/// Allocate heap memory
#[no_mangle]
pub extern "C" fn heap_alloc(size: usize) -> *mut u8 {
    if size == 0 {
        return ptr::null_mut();
    }

    let layout = Layout::from_size_align(size, 8).expect("Invalid layout");
    unsafe {
        let ptr = System.alloc(layout);
        if ptr.is_null() {
            panic!("Out of memory: cannot allocate {} bytes", size);
        }
        ptr
    }
}

/// Free heap memory
#[no_mangle]
pub extern "C" fn heap_free(ptr: *mut u8, size: usize) {
    if ptr.is_null() || size == 0 {
        return;
    }

    let layout = Layout::from_size_align(size, 8).expect("Invalid layout");
    unsafe {
        System.dealloc(ptr, layout);
    }
}

/// Reallocate heap memory
#[no_mangle]
pub extern "C" fn heap_realloc(ptr: *mut u8, old_size: usize, new_size: usize) -> *mut u8 {
    if new_size == 0 {
        if !ptr.is_null() && old_size > 0 {
            heap_free(ptr, old_size);
        }
        return ptr::null_mut();
    }

    if ptr.is_null() {
        return heap_alloc(new_size);
    }

    let old_layout = Layout::from_size_align(old_size, 8).expect("Invalid old layout");
    let new_layout = Layout::from_size_align(new_size, 8).expect("Invalid new layout");

    unsafe {
        let new_ptr = System.realloc(ptr, old_layout, new_size);
        if new_ptr.is_null() {
            panic!("Out of memory: cannot reallocate to {} bytes", new_size);
        }
        new_ptr
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// String Operations
// ─────────────────────────────────────────────────────────────────────────────

/// Represents a Dust string
#[repr(C)]
pub struct DustString {
    ptr: *const u8,
    len: usize,
    capacity: usize,
}

impl DustString {
    /// Create a new empty string
    pub fn new() -> Self {
        Self {
            ptr: ptr::null(),
            len: 0,
            capacity: 0,
        }
    }

    /// Create from a Rust string slice
    pub fn from_str(s: &str) -> Self {
        let len = s.len();
        let capacity = len + 1; // +1 for null terminator

        let ptr = heap_alloc(capacity);
        if ptr.is_null() {
            panic!("Out of memory");
        }

        unsafe {
            ptr::copy_nonoverlapping(s.as_bytes(), ptr, len);
            ptr.add(len).write(0); // null terminator
        }

        Self { ptr, len, capacity }
    }

    /// Get string length
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get string as &str
    pub fn as_str(&self) -> &str {
        unsafe {
            std::slice::from_raw_parts(self.ptr, self.len)
                .utf8_chunks()
                .as_str()
        }
    }
}

/// Allocate a new string
#[no_mangle]
pub extern "C" fn dust_string_alloc(s: &str) -> Box<DustString> {
    Box::new(DustString::from_str(s))
}

/// Free a string
#[no_mangle]
pub extern "C" fn dust_string_free(s: Box<DustString>) {
    if s.capacity > 0 {
        heap_free(s.ptr as *mut u8, s.capacity);
    }
}

/// String concatenation
#[no_mangle]
pub extern "C" fn dust_string_concat(a: &DustString, b: &DustString) -> Box<DustString> {
    let new_len = a.len + b.len;
    let new_capacity = new_len + 1;

    let new_ptr = heap_alloc(new_capacity);
    if new_ptr.is_null() {
        panic!("Out of memory");
    }

    unsafe {
        if a.len > 0 {
            ptr::copy_nonoverlapping(a.ptr, new_ptr, a.len);
        }
        if b.len > 0 {
            ptr::copy_nonoverlapping(b.ptr, new_ptr.add(a.len), b.len);
        }
        new_ptr.add(new_len).write(0);
    }

    Box::new(DustString {
        ptr: new_ptr,
        len: new_len,
        capacity: new_capacity,
    })
}

/// String length
#[no_mangle]
pub extern "C" fn dust_string_len(s: &DustString) -> usize {
    s.len
}

// ─────────────────────────────────────────────────────────────────────────────
// Error Handling
// ─────────────────────────────────────────────────────────────────────────────

/// Panic with message
#[no_mangle]
pub extern "C" fn dust_panic(msg: &str) -> ! {
    eprintln!("Dust panic: {}", msg);
    std::process::exit(1);
}

/// Assert condition
#[no_mangle]
pub extern "C" fn dust_assert(cond: bool, msg: &str) {
    if !cond {
        dust_panic(msg);
    }
}

/// Unreachable code panic
#[no_mangle]
pub extern "C" fn dust_unreachable() -> ! {
    dust_panic("unreachable code executed");
}

// ─────────────────────────────────────────────────────────────────────────────
// Type Conversions
// ─────────────────────────────────────────────────────────────────────────────

/// Convert i64 to f64
#[no_mangle]
pub extern "C" fn dust_int_to_float(i: i64) -> f64 {
    i as f64
}

/// Convert f64 to i64
#[no_mangle]
pub extern "C" fn dust_float_to_int(f: f64) -> i64 {
    f as i64
}

/// Convert char to i32
#[no_mangle]
pub extern "C" fn dust_char_to_int(c: char) -> i32 {
    c as i32
}

/// Convert i32 to char
#[no_mangle]
pub extern "C" fn dust_int_to_char(i: i32) -> char {
    char::from_u32(i as u32).unwrap_or('\0')
}

// ─────────────────────────────────────────────────────────────────────────────
// Array Operations
// ─────────────────────────────────────────────────────────────────────────────

/// Allocate array
#[no_mangle]
pub extern "C" fn dust_array_alloc<T>(size: usize) -> *mut T {
    let layout = Layout::array::<T>(size).expect("Invalid array layout");
    unsafe {
        let ptr = System.alloc(layout);
        if ptr.is_null() {
            panic!("Out of memory");
        }
        ptr
    }
}

/// Free array
#[no_mangle]
pub extern "C" fn dust_array_free<T>(ptr: *mut T, size: usize) {
    if ptr.is_null() {
        return;
    }
    let layout = Layout::array::<T>(size).expect("Invalid array layout");
    unsafe {
        System.dealloc(ptr, layout);
    }
}

/// Get array element
#[no_mangle]
pub extern "C" fn dust_array_get<T>(ptr: *mut T, index: usize) -> T {
    unsafe { *ptr.add(index) }
}

/// Set array element
#[no_mangle]
pub extern "C" fn dust_array_set<T>(ptr: *mut T, index: usize, value: T) {
    unsafe {
        *ptr.add(index) = value;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Process Entry Point
// ─────────────────────────────────────────────────────────────────────────────

/// Main entry point called by the runtime
#[no_mangle]
pub extern "C" fn dust_main() {
    // This will be replaced by the actual main function
    dust_panic("main function not defined");
}
