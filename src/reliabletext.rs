use std::{borrow::Cow, str::Utf8Error, string::FromUtf16Error};

#[derive(Debug)]
pub enum Error {
    Utf8Error(Utf8Error),
    Utf16Error(FromUtf16Error),
    Utf32Error(usize),
    BadByteOrderMark,
    BadLength,
}

pub fn decode<'a>(bytes: &'a [u8]) -> Result<Cow<'a, str>, Error> {
    if let Some(body) = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]) {
        decode_utf8(body)
    } else if let Some(body) = bytes.strip_prefix(&[0xfe, 0xff]) {
        decode_utf16be(body)
    } else if let Some(body) = bytes.strip_prefix(&[0xff, 0xfe]) {
        decode_utf16le(body)
    } else if let Some(body) = bytes.strip_prefix(&[0x0, 0x0, 0xfe, 0xff]) {
        decode_utf32be(body)
    } else {
        Err(Error::BadByteOrderMark)
    }
}

fn decode_utf32be(bytes: &[u8]) -> Result<Cow<'_, str>, Error> {
    if bytes.len() % 4 != 0 {
        return Err(Error::BadLength);
    }
    let mut out = String::new();
    out.reserve_exact(bytes.len() / 4);

    for i in 0..bytes.len() / 4 {
        let val = ((bytes[4 * i] as u32) << 24)
            | ((bytes[4 * i + 1] as u32) << 16)
            | ((bytes[4 * i + 2] as u32) << 8)
            | (bytes[4 * i + 3] as u32);
        out.push(char::from_u32(val).ok_or(Error::Utf32Error(i*4))?);
    }
    Ok(Cow::Owned(out))
}

fn decode_utf16le(bytes: &[u8]) -> Result<Cow<'_, str>, Error> {
    if bytes.len() % 2 != 0 {
        return Err(Error::BadLength);
    }
    let mut u16s = Vec::new();
    u16s.reserve_exact(bytes.len() / 2);
    for i in 0..bytes.len() / 2 {
        u16s.push((bytes[2 * i] as u16) | ((bytes[2 * i + 1] as u16) << 8));
    }
    match std::string::String::from_utf16(&u16s) {
        Ok(raw) => Ok(Cow::Owned(raw)),
        Err(e) => Err(Error::Utf16Error(e)),
    }
}

fn decode_utf16be(bytes: &[u8]) -> Result<Cow<'_, str>, Error> {
    if bytes.len() % 2 != 0 {
        return Err(Error::BadLength);
    }
    let mut u16s = Vec::new();
    u16s.reserve_exact(bytes.len() / 2);
    for i in 0..bytes.len() / 2 {
        u16s.push(((bytes[2 * i] as u16) << 8) | (bytes[2 * i + 1] as u16));
    }
    match std::string::String::from_utf16(&u16s) {
        Ok(raw) => Ok(Cow::Owned(raw)),
        Err(e) => Err(Error::Utf16Error(e)),
    }
}

fn decode_utf8(bytes: &[u8]) -> Result<Cow<'_, str>, Error> {
    match std::str::from_utf8(bytes) {
        Ok(raw) => Ok(Cow::Borrowed(raw)),
        Err(e) => Err(Error::Utf8Error(e)),
    }
}

/// Reliable text line iterator.
///
/// Reliable text lines are special in that they use a line-separator
/// instead of a line terminator. Every reliable text file has at
/// least one empty line.
pub fn lines<'a>(txt: &'a str) -> impl Iterator<Item = &'a str>
{
    txt.split('\n')
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test0()
    {
        let txt = decode(&[0xef, 0xbb, 0xbf]).unwrap();
        let lines: Vec<&str> = lines(&txt).collect();
        assert_eq!(lines, vec![""])
    }

    #[test]
    fn test1()
    {
        let txt = decode(&[0xef, 0xbb, 0xbf, 0x61]).unwrap();
        let lines: Vec<&str> = lines(&txt).collect();
        assert_eq!(lines, vec!["a"])
    }

    #[test]
    fn test2()
    {
        let txt = decode(&[0xef, 0xbb, 0xbf, 0x61, 0x0a, 0x62, 0x63]).unwrap();
        let lines: Vec<&str> = lines(&txt).collect();
        assert_eq!(lines, vec!["a", "bc"])
    }
}
