use std::os::raw::c_int;

pub fn run_cli<I, S>(args: I) -> std::io::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    use std::ffi::CString;
    use std::os::raw::c_char;

    /* Convert Rust args -> CStrings */
    let cstrings: Vec<CString> = args
        .into_iter()
        .map(|s| CString::new(s.as_ref()))
        .collect::<Result<_, _>>()
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "argument contains interior NUL byte",
            )
        })?;

    /* Build argv */
    let mut argv: Vec<*mut c_char> = cstrings.iter().map(|s| s.as_ptr() as *mut c_char).collect();

    let argc = argv.len() as c_int;

    /* Call the raw function */
    let exit_code: i32 = unsafe { snaphu_sys::run_main(argc, argv.as_mut_ptr()) };
    if exit_code == 0 {
        Ok(())
    } else {
        Err(std::io::Error::from_raw_os_error(exit_code))
    }
}

extern crate libc;
// Keep this nodule private for now. It is a c2rust translation of the original SNAPHU code.
mod snaphu_full;
