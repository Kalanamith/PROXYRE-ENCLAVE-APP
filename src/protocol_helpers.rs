use byteorder::{ByteOrder, LittleEndian};
use nix::errno::Errno;
use nix::sys::socket::MsgFlags;
use nix::sys::socket::{recv, send};
use std::convert::TryInto;
use std::mem::size_of;
use std::os::unix::io::RawFd;

pub fn send_u64(fd: RawFd, val: u64) -> Result<(), String> {
    let mut buf = [0u8; size_of::<u64>()];
    LittleEndian::write_u64(&mut buf, val);
    send_loop(fd, &mut buf, size_of::<u64>().try_into().unwrap())?;
    Ok(())
}

pub fn recv_u64(fd: RawFd) -> Result<u64, String> {
    let mut buf = [0u8; size_of::<u64>()];
    recv_loop(fd, &mut buf, size_of::<u64>().try_into().unwrap())?;
    let val = LittleEndian::read_u64(&buf);
    Ok(val)
}

/// Send `len` bytes from `buf` to a connection-oriented socket
pub fn send_loop(fd: RawFd, buf: &[u8], len: u64) -> Result<(), String> {
    let len: usize = len.try_into().map_err(|err| format!("{:?}", err))?;
    let mut send_bytes = 0;

    while send_bytes < len {
        let size = match send(fd, &buf[send_bytes..len], MsgFlags::empty()) {
            Ok(size) => size,
            Err(nix::Error::EINTR) => 0,
            Err(err) => return Err(format!("{:?}", err)),
        };
        send_bytes += size;
    }

    Ok(())
}

/// Receive `len` bytes from a connection-orriented socket
pub fn recv_loop(fd: RawFd, buf: &mut [u8], len: u64) -> Result<(), String> {
    let len: usize = len.try_into().map_err(|err| format!("{:?}", err))?;
    let mut recv_bytes = 0;

    while recv_bytes < len {
        let size = match recv(fd, &mut buf[recv_bytes..len], MsgFlags::empty()) {
            Ok(size) => size,
            Err(nix::Error::EINTR) => 0,
            Err(err) => return Err(format!("{:?}", err)),
        };
        recv_bytes += size;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::{ByteOrder, LittleEndian};

    // Test byte order conversion functions
    #[test]
    fn test_u64_conversion_round_trip() {
        let test_values = vec![
            0u64,
            1u64,
            255u64,
            65535u64,
            4294967295u64,
            18446744073709551615u64, // u64::MAX
            12345678901234567890u64,
        ];

        for &value in &test_values {
            let mut buf = [0u8; 8];
            LittleEndian::write_u64(&mut buf, value);
            let read_value = LittleEndian::read_u64(&buf);
            assert_eq!(value, read_value, "Round-trip conversion failed for value: {}", value);
        }
    }

    #[test]
    fn test_endianness_consistency() {
        let value: u64 = 0x0123456789ABCDEF;
        let mut buf = [0u8; 8];

        LittleEndian::write_u64(&mut buf, value);

        // Verify little-endian byte order
        assert_eq!(buf[0], 0xEF);
        assert_eq!(buf[1], 0xCD);
        assert_eq!(buf[2], 0xAB);
        assert_eq!(buf[3], 0x89);
        assert_eq!(buf[4], 0x67);
        assert_eq!(buf[5], 0x45);
        assert_eq!(buf[6], 0x23);
        assert_eq!(buf[7], 0x01);
    }

    // Test buffer size calculations
    #[test]
    fn test_buffer_size_calculations() {
        assert_eq!(std::mem::size_of::<u64>(), 8);
        assert_eq!(std::mem::size_of::<u32>(), 4);
        assert_eq!(std::mem::size_of::<u16>(), 2);
        assert_eq!(std::mem::size_of::<u8>(), 1);
    }

    // Test byte order operations with different integer types
    #[test]
    fn test_u32_byte_order() {
        let value: u32 = 0x12345678;
        let mut buf = [0u8; 4];

        LittleEndian::write_u32(&mut buf, value);
        let read_value = LittleEndian::read_u32(&buf);

        assert_eq!(value, read_value);
        assert_eq!(buf[0], 0x78);
        assert_eq!(buf[1], 0x56);
        assert_eq!(buf[2], 0x34);
        assert_eq!(buf[3], 0x12);
    }

    #[test]
    fn test_u16_byte_order() {
        let value: u16 = 0x1234;
        let mut buf = [0u8; 2];

        LittleEndian::write_u16(&mut buf, value);
        let read_value = LittleEndian::read_u16(&buf);

        assert_eq!(value, read_value);
        assert_eq!(buf[0], 0x34);
        assert_eq!(buf[1], 0x12);
    }

    // Test error handling for size conversions
    #[test]
    fn test_size_conversion() {
        // Test that we can convert usize to u64 for the len parameter
        let size: usize = 1024;
        let converted: u64 = size.try_into().unwrap();
        assert_eq!(converted, 1024u64);

        // Test that very large sizes would fail appropriately
        let large_size: usize = usize::MAX;
        let result: Result<u64, _> = large_size.try_into();
        if std::mem::size_of::<usize>() > std::mem::size_of::<u64>() {
            // On platforms where usize > u64, this would fail
            assert!(result.is_err());
        } else {
            assert!(result.is_ok());
        }
    }

    // Test buffer bounds checking
    #[test]
    fn test_buffer_bounds() {
        let mut buf = [0u8; 8];
        let test_value: u64 = 42;

        LittleEndian::write_u64(&mut buf, test_value);

        // Test that all bytes in buffer are accessible
        for i in 0..8 {
            let _ = buf[i]; // Should not panic
        }

        // Verify the written value can be read back
        let read_value = LittleEndian::read_u64(&buf);
        assert_eq!(test_value, read_value);
    }

    // Test multiple conversions in sequence
    #[test]
    fn test_multiple_conversions() {
        let values = [1u64, 2u64, 3u64, 4u64, 5u64];
        let mut buf = [0u8; 8];
        let mut results = Vec::new();

        for &value in &values {
            LittleEndian::write_u64(&mut buf, value);
            let read_value = LittleEndian::read_u64(&buf);
            results.push(read_value);
        }

        assert_eq!(values.to_vec(), results);
    }

    // Test edge cases
    #[test]
    fn test_edge_cases() {
        // Test with zero
        let mut buf = [0u8; 8];
        LittleEndian::write_u64(&mut buf, 0u64);
        assert_eq!(LittleEndian::read_u64(&buf), 0u64);

        // Test with maximum value
        let max_value = u64::MAX;
        LittleEndian::write_u64(&mut buf, max_value);
        assert_eq!(LittleEndian::read_u64(&buf), max_value);
    }

    // Test buffer manipulation doesn't affect other data
    #[test]
    fn test_buffer_isolation() {
        let mut buf1 = [0u8; 8];
        let mut buf2 = [0u8; 8];

        let value1: u64 = 0x1111111111111111;
        let value2: u64 = 0x2222222222222222;

        LittleEndian::write_u64(&mut buf1, value1);
        LittleEndian::write_u64(&mut buf2, value2);

        // Buffers should be independent
        assert_eq!(LittleEndian::read_u64(&buf1), value1);
        assert_eq!(LittleEndian::read_u64(&buf2), value2);

        // Verify the buffers are actually different
        assert_ne!(buf1, buf2);
    }

    // Test that byteorder operations are consistent across calls
    #[test]
    fn test_consistent_behavior() {
        let value: u64 = 0xFEDCBA9876543210;
        let mut buf1 = [0u8; 8];
        let mut buf2 = [0u8; 8];

        // Same value should produce same buffer content
        LittleEndian::write_u64(&mut buf1, value);
        LittleEndian::write_u64(&mut buf2, value);

        assert_eq!(buf1, buf2);
        assert_eq!(LittleEndian::read_u64(&buf1), LittleEndian::read_u64(&buf2));
    }
}
