//! Types used in the file format for filkoll

use crate::interner::Handle;
use crate::interner::StringInterner;
use smallvec::SmallVec;
use type_hash::TypeHash;

#[derive(
    Debug, zerocopy::IntoBytes, zerocopy::FromBytes, zerocopy::Immutable, zerocopy::KnownLayout,
)]
#[repr(C)]
pub(crate) struct Header {
    pub(crate) magic: u32,
    /// INVARIANT: A version number that is manually incremented if the format
    /// changes in ways the the hash cannot catch.
    pub(crate) version: u32,
    /// A hash of the type of the root object
    pub(crate) type_hash: u64,
}

const DATA_VERSION: u32 = 2;

impl Default for Header {
    fn default() -> Self {
        const {
            assert!(size_of::<Self>().is_multiple_of(16));
        }

        Self {
            magic: 0x70757976, // FKOL
            version: DATA_VERSION,
            type_hash: Self::expected_hash(),
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/lock_hash.rs"));

impl Header {
    pub(crate) fn is_valid(&self) -> bool {
        self.magic == 0x70757976
            && self.version == DATA_VERSION
            && self.type_hash == Self::expected_hash()
    }

    #[inline]
    fn expected_hash() -> u64 {
        DataRoot::type_hash() ^ LOCK_HASH
    }
}

// INVARIANTS:
// * The following two entries must be kept in sync for type hash to be valid.
// * If the length of the SmallVec changes, [`DATA_VERSION`] MUST be updated.
pub(crate) type BinariesRecordVec = SmallVec<[Record; 2]>;
pub type BinariesRecordVecStd = Vec<Record>;

// INVARIANT: The following two entries must be kept in sync for type hash to be
// valid
type BinariesDataStd = std::collections::HashMap<String, BinariesRecordVecStd>;
pub(crate) type BinariesData = hashbrown::HashMap<String, BinariesRecordVec>;

/// Root object for the rkyv section of the data file
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, TypeHash)]
pub(crate) struct DataRoot {
    /// Name of repository (e.g. core or extra)
    pub(crate) repository: String,
    /// Interner for paths and package names
    pub(crate) interner: StringInterner,
    /// Mapping from binary name to data about binary
    #[type_hash(as = "BinariesDataStd")]
    pub(crate) binaries: BinariesData,
}

/// A record of a file: which package and directory it belongs to
#[derive(Debug, Copy, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, TypeHash)]
pub(crate) struct Record {
    pub(crate) package: PackageRef,
    pub(crate) directory: DirectoryRef,
}

#[derive(Debug, Clone, Copy, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, TypeHash)]
#[repr(transparent)]
pub(crate) struct PackageRef(pub(crate) Handle);

#[derive(Debug, Clone, Copy, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, TypeHash)]
#[repr(transparent)]
pub(crate) struct DirectoryRef(pub(crate) Handle);
