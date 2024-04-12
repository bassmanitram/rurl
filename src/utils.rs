//
// Utilities we kinda need but that are, at the moment, unstable
//
#[inline]
pub const fn trim_ascii_start(this: &[u8]) -> &[u8] {
	let mut bytes = this;
	// Note: A pattern matching based approach (instead of indexing) allows
	// making the function const.
	while let [first, rest @ ..] = bytes {
		if *first <= 32_u8 {
			bytes = rest;
		} else {
			break;
		}
	}
	bytes
}

/// Returns a byte slice with trailing ASCII whitespace bytes removed.
///
/// 'Whitespace' refers to the definition used by
/// `u8::is_ascii_whitespace`.
///
/// # Examples
///
/// ```
/// #![feature(byte_slice_trim_ascii)]
///
/// assert_eq!(b"\r hello world\n ".trim_ascii_end(), b"\r hello world");
/// assert_eq!(b"  ".trim_ascii_end(), b"");
/// assert_eq!(b"".trim_ascii_end(), b"");
/// ```
#[inline]
pub const fn trim_ascii_end(this: &[u8]) -> &[u8] {
	let mut bytes = this;
	// Note: A pattern matching based approach (instead of indexing) allows
	// making the function const.
	while let [rest @ .., last] = bytes {
		if *last <= 32_u8 {
			bytes = rest;
		} else {
			break;
		}
	}
	bytes
}

/// Returns a byte slice with leading and trailing ASCII whitespace bytes
/// removed.
///
/// 'Whitespace' refers to the definition used by
/// `u8::is_ascii_whitespace`.
///
/// # Examples
///
/// ```
/// #![feature(byte_slice_trim_ascii)]
///
/// assert_eq!(b"\r hello world\n ".trim_ascii(), b"hello world");
/// assert_eq!(b"  ".trim_ascii(), b"");
/// assert_eq!(b"".trim_ascii(), b"");
/// ```
#[inline]
pub const fn trim_ascii(this: &[u8]) -> &[u8] {
	let a = trim_ascii_start(this);
	trim_ascii_end(a)
}