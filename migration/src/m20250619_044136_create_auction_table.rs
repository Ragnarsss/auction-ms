use sea_orm_migration::{prelude::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Auction::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Auction::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Auction::UserId).uuid().not_null())
                    .col(ColumnDef::new(Auction::ItemId).uuid().not_null())
                    .col(ColumnDef::new(Auction::Title).string().not_null())
                    .col(ColumnDef::new(Auction::Description).string().null())
                    .col(ColumnDef::new(Auction::StartTime).date_time().not_null())
                    .col(ColumnDef::new(Auction::EndTime).date_time().not_null())
                    .col(ColumnDef::new(Auction::BasePrice).decimal().not_null())
                    .col(ColumnDef::new(Auction::MinBidIncrement).decimal().not_null())
                    .col(ColumnDef::new(Auction::HighestBid).decimal().null())
                    .col(ColumnDef::new(Auction::Status).string().not_null())
                    .col(ColumnDef::new(Auction::Currency).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Auction::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Auction {
    Table,
    Id,
    UserId,
    ItemId,
    Title,
    Description,
    StartTime,
    EndTime,
    BasePrice,
    MinBidIncrement,
    HighestBid,
    Status,
    Currency,
}
