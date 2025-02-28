//! Custom interner with rkyv support
//!
//! This is made to be fast to read from once created. As such it is single
//! threaded during creation.

use type_hash::TypeHash;

/// Handle into a [`StringInterner`]
#[derive(
    Debug,
    PartialEq,
    Eq,
    Hash,
    Clone,
    Copy,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    TypeHash,
)]
#[rkyv(derive(Debug, PartialEq, Eq, Hash, Clone, Copy))]
pub struct Handle {
    offset: u32,
}

/// A read only string arena
///
/// This serves as the type that can be serialised with rkyv.
#[derive(Debug, Default, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, TypeHash)]
pub struct StringInterner {
    data: Vec<u8>,
}

/// A builder for [`StringInterner`]
#[derive(Debug, Default)]
pub struct StringInternerBuilder {
    data: Vec<u8>,
    lookup: hashbrown::HashMap<String, Handle>,
}

impl StringInternerBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Intern a string and return a handle to it
    pub fn intern(&mut self, value: &str) -> Handle {
        if let Some(handle) = self.lookup.get(value) {
            return *handle;
        }

        let offset = self.data.len() as u32;
        let size: u16 = value.len().try_into().expect("Too long string");
        self.data.push(size.to_le_bytes()[0]);
        self.data.push(size.to_le_bytes()[1]);
        self.data.extend_from_slice(value.as_bytes());
        let handle = Handle { offset };
        // Possible improvement: self referential data structure to save on RAM?
        self.lookup.insert(value.to_string(), handle);
        handle
    }

    /// Finalize the builder and create a readonly version
    pub fn into_readonly(self) -> StringInterner {
        StringInterner { data: self.data }
    }
}

impl ArchivedStringInterner {
    /// Get the string for a given handle
    pub fn get(&self, handle: ArchivedHandle) -> &str {
        let data = self.get_raw(handle);

        std::str::from_utf8(data).expect("Invalid utf8")
    }

    /// Get the raw bytes for a given handle
    pub fn get_raw(&self, handle: ArchivedHandle) -> &[u8] {
        let offset = handle.offset.to_native() as usize;
        assert!(self.data.len() >= offset);

        // Get the string size at offset (u16)
        let size = u16::from_le_bytes([self.data[offset], self.data[offset + 1]]) as usize;
        // Get the string at offset + 2
        &self.data[(offset + 2)..(offset + 2 + size)]
    }

    #[cfg(test)]
    pub fn iter(&self) -> InternerIter<'_> {
        InternerIter {
            data: &self.data,
            offset: 0,
        }
    }
}

/// Iterator type for an interner
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct InternerIter<'a> {
    data: &'a [u8],
    offset: usize,
}

#[cfg(test)]
impl<'a> Iterator for InternerIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.data.len() {
            return None;
        }

        let size =
            u16::from_le_bytes([self.data[self.offset], self.data[self.offset + 1]]) as usize;
        let start = self.offset + 2;
        let end = start + size;
        self.offset = end;
        // SAFETY: We only allow valid UTF-8 to be added.
        Some(unsafe { std::str::from_utf8_unchecked(&self.data[start..end]) })
    }
}

/// Needed for the test below, other than that this is a bit of a footgun.
#[cfg(test)]
impl From<Handle> for ArchivedHandle {
    fn from(handle: Handle) -> Self {
        Self {
            offset: rkyv::rend::u32_le::from_native(handle.offset),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interner() {
        let mut builder = StringInternerBuilder::new();
        let a = builder.intern("a");
        let b = builder.intern("bcde");
        let c = builder.intern("c");
        let d = builder.intern("d");
        let e = builder.intern("eåäö");
        let f = builder.intern("a");

        let interner = builder.into_readonly();

        let serialised = rkyv::api::high::to_bytes::<rkyv::rancor::Error>(&interner).unwrap();
        let archived =
            rkyv::api::high::access::<ArchivedStringInterner, rkyv::rancor::Error>(&serialised)
                .unwrap();

        assert_eq!(archived.get(a.into()), "a");
        assert_eq!(archived.get(b.into()), "bcde");
        assert_eq!(archived.get(c.into()), "c");
        assert_eq!(archived.get(d.into()), "d");
        assert_eq!(archived.get(e.into()), "eåäö");
        assert_eq!(archived.get(f.into()), "a");

        // Check that we actually deduplicate
        assert_eq!(
            archived.iter().collect::<Vec<_>>(),
            vec!["a", "bcde", "c", "d", "eåäö"]
        );
    }
}
