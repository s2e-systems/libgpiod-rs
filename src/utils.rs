use std::{io, mem::size_of_val, str};

#[inline(always)]
pub fn is_set<T>(flags: T, flag: T) -> bool
where
    T: core::ops::BitAnd<Output = T> + Eq + Copy,
{
    flags & flag == flag
}

#[inline(always)]
pub fn invalid_input() -> io::Error {
    io::Error::from(io::ErrorKind::InvalidInput)
}

#[inline(always)]
pub fn invalid_data() -> io::Error {
    io::Error::from(io::ErrorKind::InvalidData)
}

#[inline(always)]
pub fn check_len<V, T: ?Sized>(slice: &[V], val: &T) -> io::Result<()> {
    if slice.len() < size_of_val(val) {
        Ok(())
    } else {
        Err(invalid_input())
    }
}

#[inline(always)]
pub fn check_len_str<T: ?Sized>(slice: &str, val: &T) -> io::Result<()> {
    if slice.as_bytes().len() + 1 /* \0 */ < size_of_val(val) {
        Ok(())
    } else {
        Err(invalid_input())
    }
}

#[inline(always)]
pub fn safe_set_str<const N: usize>(dst: &mut [u8; N], src: &str) -> io::Result<()> {
    check_len_str(src, &dst)?;

    let src = src.as_bytes();
    dst.copy_from_slice(src);
    dst[src.len()] = 0;

    Ok(())
}

#[inline(always)]
pub fn safe_get_str(src: &[u8]) -> io::Result<&str> {
    Ok(str::from_utf8(src)
        .map_err(|_| invalid_data())?
        .trim_end_matches('\0'))
}
