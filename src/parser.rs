pub fn trim_whitespaces(buf: &[u8]) -> &[u8] {
    let mut start = 0;
    
    while let Some(c) = buf.get(start) {
        if *c == b' ' {
            start += 1;
        } else {
            break;
        }
    }
    
    if start >= buf.len() {
        return &[];
    }
    
    let mut end = buf.len() - 1;
    
    while let Some(c) = buf.get(end) {
        if *c == b' ' {
            end -= 1;
        } else {
            break;
        }
    }
    
    end += 1;
    
    &buf[start..end]
}

pub fn split_once(buf: &[u8], delim: u8) -> (&[u8], &[u8]) {
    let mut cursor = 0;
    
    while let Some(c) = buf.get(cursor) {
        if *c == delim {
            let start = cursor + 1;
            
            if start >= buf.len() {
                return (&buf[..cursor], &[]);
            } else {
                return (&buf[..cursor], &buf[start..]);
            }
            
        } else {
            cursor += 1;
        }
    }
    
    (buf, &[])
}

#[inline]
pub fn convert_number(buf: &[u8]) -> usize {
    let mut result = 0;
    for c in buf.iter() {
        result *= 10;
        result += (*c - b'0') as usize;
    }
    result
}

#[inline]
pub fn is_numerical(buf: &[u8]) -> bool {
    for c in buf {
        if !c.is_ascii_digit() {
            return false;
        }
    }
    
    true
}