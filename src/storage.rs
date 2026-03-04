use esp_hal::peripherals::FLASH;
use esp_partition_table::{PartitionEntry, PartitionTable};
use esp_storage::FlashStorage;
use heapless::Vec;
use mem_fs::{FsErr, MemFs};

#[derive(Default)]
struct SlotHeader {
    pub magic: [u8; 4],
    pub sequence: u32,
}
const HEADER_SIZE: usize = size_of::<SlotHeader>();

impl SlotHeader {
    fn new(sequence: u32) -> Self {
        Self {
            magic: *b"SHAB",
            sequence,
        }
    }

    fn encode(&self) -> [u8; HEADER_SIZE] {
        let mut out = [0u8; HEADER_SIZE];
        out[0..4].copy_from_slice(&self.magic);
        out[4..8].copy_from_slice(&self.sequence.to_le_bytes());
        out
    }

    fn decode(buf: &[u8; HEADER_SIZE]) -> Option<Self> {
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&buf[0..4]);
        if magic != *b"SHAB" {
            return None;
        }

        let mut seq_bytes = [0u8; 4];
        seq_bytes.copy_from_slice(&buf[4..8]);
        let sequence = u32::from_le_bytes(seq_bytes);

        Some(Self { magic, sequence })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Slot {
    A,
    B,
}

impl Slot {
    fn other(self) -> Slot {
        match self {
            Slot::A => Slot::B,
            Slot::B => Slot::A,
        }
    }
}

pub struct Storage<'a> {
    storage: FlashStorage<'a>,
    partition_table: PartitionTable,
}

#[derive(Debug)]
pub enum StorageError {
    PartitionNotFound,
    InvalidSize,
    WriteFail,
    ReadFail,
    NotFound,
    FsErr(FsErr),
}

impl<'a> Storage<'a> {
    pub fn new(flash: FLASH<'a>) -> Self {
        Self {
            storage: FlashStorage::new(flash),
            partition_table: PartitionTable::new(
                PartitionTable::DEFAULT_ADDR,
                PartitionTable::MAX_SIZE,
            ),
        }
    }

    pub fn dump_memfs(&mut self, memfs: &mut MemFs) -> Result<(), StorageError> {
        if let Some(partition) = &self.get_memfs_partition() {
            let slot = self
                .get_valid_slot(partition.offset, true)
                .unwrap_or(Slot::A);

            let offset = self.slot_offset(partition.offset, slot);

            let data = self.dump_to_vec(&memfs);
            if data.len() > MemFs::serialized_max_size() {
                return Err(StorageError::InvalidSize);
            }

            let result = embedded_storage::Storage::write(
                &mut self.storage,
                offset + HEADER_SIZE as u32,
                &data,
            );

            // Update header after write is complete
            if result.is_ok() {
                let new_seq = self
                    .read_slot_header(self.slot_offset(partition.offset, slot.other()))
                    .map_or(0, |header| header.sequence)
                    .wrapping_add(1);

                let header_data = SlotHeader::new(new_seq).encode();

                embedded_storage::Storage::write(&mut self.storage, offset, &header_data)
                    .map_err(|_err| StorageError::WriteFail)?;

                return Ok(());
            }
            Err(StorageError::WriteFail)
        } else {
            Err(StorageError::PartitionNotFound)
        }
    }

    pub fn restore_memfs(&mut self) -> Result<MemFs, StorageError> {
        if let Some(partition) = &self.get_memfs_partition() {
            let slot = self
                .get_valid_slot(partition.offset, false)
                .ok_or(StorageError::NotFound)?;
            let offset = self.slot_offset(partition.offset, slot) + HEADER_SIZE as u32;

            let mut restored_memfs = MemFs::new();
            let mut data: [u8; MemFs::serialized_max_size()] = [0; MemFs::serialized_max_size()];

            embedded_storage::ReadStorage::read(&mut self.storage, offset, &mut data)
                .map_err(|_err| StorageError::ReadFail)?;

            if data.starts_with(b"MEMFS") {
                self.restore_from_slice(&mut restored_memfs, &data)
                    .map_err(|err| StorageError::FsErr(err))?;
                return Ok(restored_memfs);
            }

            Err(StorageError::ReadFail)
        } else {
            Err(StorageError::PartitionNotFound)
        }
    }

    fn get_memfs_partition(&mut self) -> Option<PartitionEntry> {
        let entries: Vec<esp_partition_table::PartitionEntry, 32> = self
            .partition_table
            .read_storage(&mut self.storage, /*check_md5=*/ None)
            .ok()
            .unwrap();

        if let Some(entry) = entries.iter().find(|e| e.name() == "memfs") {
            Some(entry.clone())
        } else {
            None
        }
    }

    fn slot_offset(&self, partition_offset: u32, slot: Slot) -> u32 {
        let slot_size = (HEADER_SIZE + MemFs::serialized_max_size()) as u32;

        match slot {
            Slot::A => partition_offset,
            Slot::B => partition_offset + slot_size,
        }
    }

    fn read_slot_header(&mut self, offset: u32) -> Option<SlotHeader> {
        let mut buf = [0u8; HEADER_SIZE];
        embedded_storage::ReadStorage::read(&mut self.storage, offset, &mut buf).ok()?;
        SlotHeader::decode(&buf)
    }

    fn get_valid_slot(&mut self, partition_offset: u32, write: bool) -> Option<Slot> {
        let a_offset = self.slot_offset(partition_offset, Slot::A);
        let b_offset = self.slot_offset(partition_offset, Slot::B);

        let a = self.read_slot_header(a_offset);
        let b = self.read_slot_header(b_offset);

        let slot = match (a, b) {
            (None, None) => None,
            (Some(_), None) => Some(Slot::A),
            (None, Some(_)) => Some(Slot::B),
            (Some(ha), Some(hb)) => Some(if ha.sequence >= hb.sequence {
                Slot::A
            } else {
                Slot::B
            }),
        }?;

        if write {
            return Some(slot.other());
        } else {
            return Some(slot);
        }
    }

    fn dump_to_vec(&self, fs: &MemFs) -> Vec<u8, 4196> {
        let mut out = Vec::new();
        fs.dump(|chunk| out.extend_from_slice(chunk).unwrap())
            .unwrap();
        out
    }

    fn restore_from_slice(&self, fs: &mut MemFs, data: &[u8]) -> Result<(), mem_fs::FsErr> {
        let mut pos = 0usize;
        fs.restore(|buf| {
            let end = pos + buf.len();
            if end > data.len() {
                return Err(mem_fs::FsErr::Corrupt);
            }
            buf.copy_from_slice(&data[pos..end]);
            pos = end;
            Ok(())
        })
    }
}
