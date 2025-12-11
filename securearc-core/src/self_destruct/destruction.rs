//! Self-destruct mechanism execution

use crate::format::header::SecurityHeader;
use crate::format::keyslot::KeySlot;
use crate::SecureArcError;
use rand::{thread_rng, RngCore};

/// Self-destruct executor
pub struct SelfDestruct;

impl SelfDestruct {
    /// Execute self-destruction on the archive
    /// This permanently destroys all key material and marks the archive as destroyed
    pub fn execute_destruction(
        header: &mut SecurityHeader,
        key_slots: &mut [KeySlot],
    ) -> Result<(), SecureArcError> {
        // Step 1: Zero all key slots
        for slot in key_slots.iter_mut() {
            slot.zeroize();
        }

        // Step 2: Corrupt security header KDF parameters
        // Overwrite with random data to make key derivation impossible
        let mut rng = thread_rng();
        rng.fill_bytes(&mut header.salt);
        header.kdf_memory = rng.next_u32();
        header.kdf_iterations = rng.next_u32();
        header.kdf_parallelism = rng.next_u32();

        // Step 3: Set destruction flag
        header.destroyed = true;

        // Step 4: Corrupt checksum (already invalid due to parameter changes)
        rng.fill_bytes(&mut header.checksum);

        Ok(())
    }

    /// Verify if archive has been destroyed
    pub fn is_destroyed(header: &SecurityHeader) -> bool {
        header.destroyed || header.should_destroy()
    }

    /// Check if key slots are all zeroized
    pub fn are_key_slots_destroyed(key_slots: &[KeySlot]) -> bool {
        key_slots.iter().all(|slot| slot.is_zeroized())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::keyslot::KeySlot;

    #[test]
    fn test_destruction_execution() {
        let mut header = SecurityHeader::new(5).unwrap();
        let mut key_slots = vec![KeySlot::new(0), KeySlot::new(1)];

        // Initialize key slots with dummy data
        key_slots[0].encrypted_key = vec![1, 2, 3, 4, 5];
        key_slots[0].active = true;
        key_slots[1].encrypted_key = vec![6, 7, 8, 9, 10];
        key_slots[1].active = true;

        // Execute destruction
        SelfDestruct::execute_destruction(&mut header, &mut key_slots).unwrap();

        // Verify destruction
        assert!(SelfDestruct::is_destroyed(&header));
        assert!(SelfDestruct::are_key_slots_destroyed(&key_slots));
        assert!(header.destroyed);
    }
}
