//! Extract executable payloads from the pinned backend's sectionless ELF format.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub struct Segment {
    pub address: u64,
    pub bytes: Vec<u8>,
    end: u64,
}
struct Header {
    entry: u64,
    count: usize,
    end: usize,
}
fn bytes<const N: usize>(data: &[u8], at: usize) -> Result<[u8; N]> {
    let end = at.checked_add(N).ok_or("ELF field offset overflow")?;
    Ok(data.get(at..end).ok_or("truncated ELF field")?.try_into()?)
}
fn header(data: &[u8]) -> Result<Header> {
    if data.get(..7) != Some(b"\x7fELF\x02\x01\x01")
        || u16::from_le_bytes(bytes(data, 16)?) != 2
        || u16::from_le_bytes(bytes(data, 18)?) != 62
        || u32::from_le_bytes(bytes(data, 20)?) != 1
        || u16::from_le_bytes(bytes(data, 52)?) != 64
        || u64::from_le_bytes(bytes(data, 32)?) != 64
        || u16::from_le_bytes(bytes(data, 54)?) != 56
        || u64::from_le_bytes(bytes(data, 40)?) != 0
        || u16::from_le_bytes(bytes(data, 60)?) != 0
    {
        return Err("capability executable differs from the pinned ELF64 LE x86-64 layout".into());
    }
    let count = usize::from(u16::from_le_bytes(bytes(data, 56)?));
    if count == 0 || count > 128 {
        return Err("invalid ELF program header count".into());
    }
    let end = count
        .checked_mul(56)
        .and_then(|size| size.checked_add(64))
        .ok_or("ELF program table overflow")?;
    if end > data.len() {
        return Err("truncated ELF program table".into());
    }
    Ok(Header {
        entry: u64::from_le_bytes(bytes(data, 24)?),
        count,
        end,
    })
}
fn segment(data: &[u8], at: usize, header_end: usize) -> Result<Option<Segment>> {
    let kind = u32::from_le_bytes(bytes(data, at)?);
    let flags = u32::from_le_bytes(bytes(data, at + 4)?);
    if kind != 1 || flags & 1 == 0 {
        return Ok(None);
    }
    let start = usize::try_from(u64::from_le_bytes(bytes(data, at + 8)?))?;
    let address = u64::from_le_bytes(bytes(data, at + 16)?);
    let file_size = u64::from_le_bytes(bytes(data, at + 32)?);
    let memory_size = u64::from_le_bytes(bytes(data, at + 40)?);
    let align = u64::from_le_bytes(bytes(data, at + 48)?);
    if file_size > memory_size
        || align > 1
            && (!align.is_power_of_two() || u64::try_from(start)? % align != address % align)
    {
        return Err("invalid executable ELF segment geometry".into());
    }
    let end = start
        .checked_add(usize::try_from(file_size)?)
        .ok_or("ELF executable file range overflow")?;
    let virtual_end = address
        .checked_add(file_size)
        .ok_or("ELF executable address range overflow")?;
    address
        .checked_add(memory_size)
        .ok_or("ELF executable memory range overflow")?;
    if end > data.len() {
        return Err("truncated executable ELF segment".into());
    }
    // This pinned format has its ELF header and complete program table first.
    // Their executable mapping is not instruction evidence: exclude those bytes.
    let payload = start.max(header_end);
    if payload >= end {
        return Err("executable ELF segment has no code payload".into());
    }
    Ok(Some(Segment {
        address: address
            .checked_add(u64::try_from(payload - start)?)
            .ok_or("ELF code address overflow")?,
        bytes: data
            .get(payload..end)
            .ok_or("invalid ELF code bounds")?
            .to_vec(),
        end: virtual_end,
    }))
}
pub fn extract(data: &[u8]) -> Result<Vec<Segment>> {
    let header = header(data)?;
    let mut segments: Vec<Segment> = Vec::new();
    for index in 0..header.count {
        if let Some(segment) = segment(data, 64 + index * 56, header.end)? {
            if segments
                .iter()
                .any(|prior| prior.address < segment.end && segment.address < prior.end)
            {
                return Err("overlapping executable ELF payloads".into());
            }
            segments.push(segment);
        }
    }
    if !segments
        .iter()
        .any(|segment| segment.address <= header.entry && header.entry < segment.end)
    {
        return Err("ELF entry is outside the executable payloads".into());
    }
    Ok(segments)
}
