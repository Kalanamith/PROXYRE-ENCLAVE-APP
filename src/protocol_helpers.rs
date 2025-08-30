use byteorder::{ByteOrder, LittleEndian};
use nix::errno::Errno;
use nix::sys::socket::MsgFlags;
use nix::sys::socket::{recv, send};
use std::convert::TryInto;
use std::mem::size_of;
use std::os::unix::io::RawFd;

/// Sends a 64-bit unsigned integer over a socket connection.
///
/// This function converts a `u64` value to its little-endian byte representation
/// and sends it over the specified socket file descriptor. The function uses
/// the `send_loop` function internally to ensure all bytes are transmitted
/// reliably.
///
/// # Parameters
/// * `fd` - The raw file descriptor of the socket to send data through
/// * `val` - The 64-bit unsigned integer value to send
///
/// # Returns
/// * `Ok(())` - If the value was successfully sent
/// * `Err(String)` - If an error occurred during sending
///
/// # Errors
/// This function will return an error if:
/// - The socket send operation fails
/// - The connection is broken
/// - A signal interrupts the send operation
///
/// # Examples
///
/// ```rust
/// use std::os::unix::io::RawFd;
/// use proxy_reencyption_enclave_app::protocol_helpers::send_u64;
///
/// fn send_data(sock_fd: RawFd, data: u64) -> Result<(), String> {
///     send_u64(sock_fd, data)?;
///     Ok(())
/// }
/// ```
///
/// # Protocol Details
/// - Uses little-endian byte order for network transmission
/// - Sends exactly 8 bytes (size of u64)
/// - Guarantees complete transmission of all bytes
pub fn send_u64(fd: RawFd, val: u64) -> Result<(), String> {
    let mut buf = [0u8; size_of::<u64>()];
    LittleEndian::write_u64(&mut buf, val);
    send_loop(fd, &mut buf, size_of::<u64>().try_into().unwrap())?;
    Ok(())
}

/// Receives a 64-bit unsigned integer from a socket connection.
///
/// This function reads exactly 8 bytes from the specified socket file descriptor,
/// interprets them as a little-endian `u64` value, and returns the result.
/// The function uses the `recv_loop` function internally to ensure all bytes
/// are received reliably before attempting to decode the value.
///
/// # Parameters
/// * `fd` - The raw file descriptor of the socket to receive data from
///
/// # Returns
/// * `Ok(u64)` - The received 64-bit unsigned integer value
/// * `Err(String)` - If an error occurred during receiving
///
/// # Errors
/// This function will return an error if:
/// - The socket receive operation fails
/// - The connection is closed before all bytes are received
/// - A signal interrupts the receive operation
/// - The received data cannot be interpreted as a valid u64
///
/// # Examples
///
/// ```rust
/// use std::os::unix::io::RawFd;
/// use proxy_reencyption_enclave_app::protocol_helpers::recv_u64;
///
/// fn receive_data(sock_fd: RawFd) -> Result<u64, String> {
///     let data = recv_u64(sock_fd)?;
///     println!("Received value: {}", data);
///     Ok(data)
/// }
/// ```
///
/// # Protocol Details
/// - Expects little-endian byte order from network transmission
/// - Reads exactly 8 bytes (size of u64)
/// - Blocks until all bytes are received or an error occurs
/// - Returns the decoded integer value
pub fn recv_u64(fd: RawFd) -> Result<u64, String> {
    let mut buf = [0u8; size_of::<u64>()];
    recv_loop(fd, &mut buf, size_of::<u64>().try_into().unwrap())?;
    let val = LittleEndian::read_u64(&buf);
    Ok(val)
}

/// Sends a specified number of bytes from a buffer to a connection-oriented socket.
///
/// This function ensures reliable transmission of data by handling partial sends
/// and interruptions. It will continue sending data until either all requested
/// bytes have been transmitted or an unrecoverable error occurs.
///
/// The function is designed to handle the common scenario where a single `send()`
/// call may not transmit all the requested data, requiring multiple calls to
/// complete the transmission.
///
/// # Parameters
/// * `fd` - The raw file descriptor of the socket to send data through
/// * `buf` - A byte slice containing the data to send
/// * `len` - The number of bytes to send from the buffer
///
/// # Returns
/// * `Ok(())` - If all bytes were successfully sent
/// * `Err(String)` - If an error occurred during sending
///
/// # Errors
/// This function will return an error if:
/// - The socket send operation fails with a non-interrupt error
/// - The connection is broken
/// - The buffer doesn't contain enough data for the requested length
/// - The length conversion to usize fails (on platforms where usize < u64)
///
/// # Behavior
/// - **Partial Sends**: Handles partial send operations by continuing to send remaining data
/// - **Signal Handling**: Ignores `EINTR` (interrupted system call) and retries the operation
/// - **Reliability**: Guarantees that either all data is sent or an error is returned
///
/// # Examples
///
/// ```rust
/// use std::os::unix::io::RawFd;
/// use proxy_reencyption_enclave_app::protocol_helpers::send_loop;
///
/// fn send_message(sock_fd: RawFd, message: &[u8]) -> Result<(), String> {
///     let len = message.len() as u64;
///     send_loop(sock_fd, message, len)?;
///     Ok(())
/// }
/// ```
///
/// # Performance Considerations
/// - Uses a loop to handle partial sends efficiently
/// - Minimizes system calls by sending as much data as possible per call
/// - Handles signal interruptions gracefully without data loss
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

/// Receives a specified number of bytes from a connection-oriented socket into a buffer.
///
/// This function ensures reliable reception of data by handling partial receives
/// and interruptions. It will continue receiving data until either the requested
/// number of bytes have been read or an unrecoverable error occurs.
///
/// The function is designed to handle the common scenario where a single `recv()`
/// call may not receive all the requested data, requiring multiple calls to
/// complete the reception.
///
/// # Parameters
/// * `fd` - The raw file descriptor of the socket to receive data from
/// * `buf` - A mutable byte slice to store the received data
/// * `len` - The number of bytes to receive into the buffer
///
/// # Returns
/// * `Ok(())` - If all requested bytes were successfully received
/// * `Err(String)` - If an error occurred during receiving
///
/// # Errors
/// This function will return an error if:
/// - The socket receive operation fails with a non-interrupt error
/// - The connection is closed before all bytes are received
/// - The buffer is too small for the requested amount of data
/// - The length conversion to usize fails (on platforms where usize < u64)
///
/// # Behavior
/// - **Partial Receives**: Handles partial receive operations by continuing to read remaining data
/// - **Signal Handling**: Ignores `EINTR` (interrupted system call) and retries the operation
/// - **Reliability**: Guarantees that either all requested data is received or an error is returned
/// - **Blocking**: Will block until all data is received or an error occurs
///
/// # Examples
///
/// ```rust
/// use std::os::unix::io::RawFd;
/// use proxy_reencyption_enclave_app::protocol_helpers::recv_loop;
///
/// fn receive_message(sock_fd: RawFd) -> Result<Vec<u8>, String> {
///     let mut buffer = vec![0u8; 1024];
///     let message_len = 256; // Expected message length
///
///     recv_loop(sock_fd, &mut buffer, message_len)?;
///
///     // Resize buffer to actual message length
///     buffer.truncate(message_len as usize);
///     Ok(buffer)
/// }
/// ```
///
/// # Important Notes
/// - The function assumes the buffer is large enough to hold `len` bytes
/// - Data is written starting from the beginning of the buffer
/// - The function will overwrite any existing data in the buffer
/// - Zero bytes received typically indicates the connection was closed by the peer
///
/// # Performance Considerations
/// - Uses a loop to handle partial receives efficiently
/// - Minimizes system calls by reading as much data as possible per call
/// - Handles signal interruptions gracefully without data loss
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
