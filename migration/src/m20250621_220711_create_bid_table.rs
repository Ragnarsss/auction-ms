use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Bid::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Bid::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Bid::AuctionId).uuid().not_null())
                    .col(ColumnDef::new(Bid::UserId).string().not_null())
                    .col(ColumnDef::new(Bid::Amount).decimal().not_null())
                    .col(ColumnDef::new(Bid::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Bid::Status).string().not_null().default("active"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_bid_auction")
                            .from(Bid::Table, Bid::AuctionId)
                            .to(Auction::Table, Auction::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Bid::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Bid {
    Table,
    Id,
    AuctionId,
    UserId,
    Amount,
    CreatedAt,
    Status,
}

#[derive(Iden)]
enum Auction {
    Table,
    Id,
}
