use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ActiveModelTrait, Set};
use uuid::Uuid;
use crate::models::auction::{Entity as Auction, Model as AuctionModel, ActiveModel as AuctionActiveModel};
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuctionDto {
    pub user_id: Uuid,
    pub item_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub start_time: chrono::NaiveDateTime,
    pub end_time: chrono::NaiveDateTime,
    pub base_price: rust_decimal::Decimal,
    pub min_bid_increment: rust_decimal::Decimal,
    pub status: String,
}

#[derive(Deserialize)]
pub struct UpdateAuctionDto {
    pub title: Option<String>,
    pub description: Option<String>,
    pub start_time: Option<NaiveDateTime>,
    pub end_time: Option<NaiveDateTime>,
    pub base_price: Option<Decimal>,
    pub min_bid_increment: Option<Decimal>,
    pub highest_bid: Option<Decimal>,
    pub status: Option<String>,
}

#[get("/auctions")]
pub async fn list_auctions(db:web::Data<DatabaseConnection>) -> impl Responder {
    let auctions: Result<Vec<AuctionModel>, sea_orm::DbErr> = Auction::find().all(db.get_ref()).await;
    match auctions {
        Ok(auctions) => HttpResponse::Ok().json(auctions),
        Err(err) => HttpResponse::InternalServerError().body(format!("Error fetching auctions: {}", err)),
    }
}

#[get("/auctions/{id}")]
pub async fn get_auction(db: web::Data<DatabaseConnection>, id: web::Path<Uuid>) -> impl Responder {
    match Auction::find_by_id(id.into_inner()).one(db.get_ref()).await {
        Ok(Some(auction)) => HttpResponse::Ok().json(auction),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/auctions")]
pub async fn create_auction(
    db: web::Data<DatabaseConnection>,
    data: web::Json<CreateAuctionDto>,
) -> impl Responder {
    let auction: AuctionActiveModel = AuctionActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(data.user_id),
        item_id: Set(data.item_id),
        title: Set(data.title.clone()),
        description: Set(data.description.clone()),
        start_time: Set(data.start_time),
        end_time: Set(data.end_time),
        base_price: Set(data.base_price),
        min_bid_increment: Set(data.min_bid_increment),
        highest_bid: Set(None),
        status: Set(data.status.clone()),
    };
    match auction.insert(db.get_ref()).await {
        Ok(inserted) => HttpResponse::Created().json(inserted),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[put("/auctions/{id}")]
pub async fn update_auction(
    db: web::Data<DatabaseConnection>,
    id: web::Path<Uuid>,
    data: web::Json<UpdateAuctionDto>,
) -> impl Responder {
    let id: Uuid = id.into_inner();
    let found: Result<Option<AuctionModel>, sea_orm::DbErr> = Auction::find_by_id(id).one(db.get_ref()).await;
    if let Ok(Some(model)) = found {
        let mut active: AuctionActiveModel = model.into();
        if let Some(title) = &data.title { active.title = Set(title.clone()); }
        if let Some(description) = &data.description { active.description = Set(Some(description.clone())); }
        if let Some(start_time) = data.start_time { active.start_time = Set(start_time); }
        if let Some(end_time) = data.end_time { active.end_time = Set(end_time); }
        if let Some(base_price) = data.base_price { active.base_price = Set(base_price); }
        if let Some(min_bid_increment) = data.min_bid_increment { active.min_bid_increment = Set(min_bid_increment); }
        if let Some(highest_bid) = data.highest_bid { active.highest_bid = Set(Some(highest_bid)); }
        if let Some(status) = &data.status { active.status = Set(status.clone()); }
        match active.update(db.get_ref()).await {
            Ok(updated) => HttpResponse::Ok().json(updated),
            Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
        }
    } else {
        HttpResponse::NotFound().body("Auction not found")
    }
}

#[delete("/auctions/{id}")]
pub async fn delete_auction(
    db: web::Data<DatabaseConnection>,
    id: web::Path<Uuid>,
) -> impl Responder {
    match Auction::delete_by_id(id.into_inner()).exec(db.get_ref()).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::NaiveDateTime;
    use rust_decimal::Decimal;

    #[test]
    fn test_create_auction_dto() {
        let dto = CreateAuctionDto {
            user_id: Uuid::new_v4(),
            item_id: Uuid::new_v4(),
            title: "Test Auction".to_string(),
            description: Some("desc".to_string()),
            start_time: NaiveDateTime::from_timestamp(1_600_000_000, 0),
            end_time: NaiveDateTime::from_timestamp(1_600_000_100, 0),
            base_price: Decimal::new(1000, 2),
            min_bid_increment: Decimal::new(100, 2),
            status: "active".to_string(),
        };
        assert_eq!(dto.title, "Test Auction");
        assert_eq!(dto.status, "active");
    }

    #[test]
    fn test_update_auction_dto() {
        let dto = UpdateAuctionDto {
            title: Some("Updated".to_string()),
            description: None,
            start_time: None,
            end_time: None,
            base_price: Some(Decimal::new(2000, 2)),
            min_bid_increment: None,
            highest_bid: None,
            status: Some("finished".to_string()),
        };
        assert_eq!(dto.title, Some("Updated".to_string()));
        assert_eq!(dto.status, Some("finished".to_string()));
    }
}