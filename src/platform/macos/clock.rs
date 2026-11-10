//! Local wall-clock time on macOS, from the C library, so no date crate is needed. Used by util.rs and learning/annoyance.rs.

/// (year, month, day, hour, minute) in local time.
pub fn local_time() -> (i32, u32, u32, u32, u32) {
    #[repr(C)]
    struct Tm {
        sec: i32,
        min: i32,
        hour: i32,
        mday: i32,
        mon: i32,
        year: i32,
        wday: i32,
        yday: i32,
        isdst: i32,
        gmtoff: std::ffi::c_long,
        zone: *const std::ffi::c_char,
    }
    extern "C" {
        fn time(t: *mut i64) -> i64;
        fn localtime_r(t: *const i64, out: *mut Tm) -> *mut Tm;
    }
    unsafe {
        let now = time(std::ptr::null_mut());
        let mut tm: Tm = std::mem::zeroed();
        if localtime_r(&now, &mut tm).is_null() {
            return (1970, 1, 1, 0, 0);
        }
        (tm.year + 1900, (tm.mon + 1) as u32, tm.mday as u32, tm.hour as u32, tm.min as u32)
    }
}

/// Minutes since local midnight.
pub fn local_minutes() -> u32 {
    let (_, _, _, h, m) = local_time();
    h * 60 + m
}

/// "YYYY-MM-DD HH:MM" in local time.
pub fn stamp() -> String {
    let (y, mo, d, h, mi) = local_time();
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}")
}
