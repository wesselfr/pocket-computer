use esp_hal::peripherals::FLASH;
use esp_partition_table::{PartitionEntry, PartitionTable};
use esp_storage::FlashStorage;
use heapless::Vec;
use mem_fs::MemFs;

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
        // TODO: Use an A/B system, so we don't lose the previous data on an unexpected power loss.
        if let Some(partition) = &self.get_memfs_partition() {
            let data = self.dump_to_vec(&memfs);

            if data.len() > partition.size {
                return Err(StorageError::InvalidSize);
            }

            let result =
                embedded_storage::Storage::write(&mut self.storage, partition.offset, &data)
                    .map_err(|_| StorageError::WriteFail);
            result
        } else {
            Err(StorageError::PartitionNotFound)
        }
    }

    pub fn restore_memfs(&mut self) -> Result<MemFs, StorageError> {
        if let Some(partition) = &self.get_memfs_partition() {
            let mut restored_memfs = MemFs::new();

            let mut data: [u8; mem_fs::DEFAULT_STORAGE_SIZE + 100] =
                [0; mem_fs::DEFAULT_STORAGE_SIZE + 100];
            if embedded_storage::ReadStorage::read(&mut self.storage, partition.offset, &mut data)
                .is_ok()
            {
                if data.starts_with(b"MEMFS") {
                    self.restore_from_slice(&mut restored_memfs, &data).unwrap();
                    Ok(restored_memfs)
                } else {
                    Err(StorageError::NotFound)
                }
            } else {
                Err(StorageError::ReadFail)
            }
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
