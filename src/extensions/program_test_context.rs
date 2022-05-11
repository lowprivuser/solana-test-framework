use async_trait::async_trait;
use solana_program_test::ProgramTestContext;
use solana_sdk::sysvar::clock::Clock;

#[async_trait]
pub trait ProgramTestContextExtension {
    /// Calculate slot number from the provided timestamp
    async fn warp_to_timestamp(&mut self, timestamp: i64);
}

#[async_trait]
impl ProgramTestContextExtension for ProgramTestContext {
    async fn warp_to_timestamp(&mut self, timestamp: i64) {
        let clock: Clock = self.banks_client.get_sysvar().await.unwrap();
        let mut slot = clock.slot;
        let mut now = clock.unix_timestamp;

        while now < timestamp {
            slot = slot + 2;
            self.warp_to_slot(slot);
            let clock: Clock = self.banks_client.get_sysvar().await.unwrap();
            now = clock.unix_timestamp;
        }

        let clock: Clock = self.banks_client.get_sysvar().await.unwrap();
        now = clock.unix_timestamp;
    }
}