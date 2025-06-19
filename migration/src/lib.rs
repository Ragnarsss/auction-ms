pub use sea_orm_migration::prelude::*;

mod m20250619_044136_create_auction_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250619_044136_create_auction_table::Migration),
        ]
    }
}
