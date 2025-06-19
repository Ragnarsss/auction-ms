use auction_ms::handlers::auctions::{CreateAuctionDto, UpdateAuctionDto};
use auction_ms::routes::auctions::auctions_routes;
use auction_ms::db;
use uuid::Uuid;
use chrono::{NaiveDateTime, Duration};
use rust_decimal::Decimal;
use actix_web::{test, web, App};
use auction_ms::handlers::auctions::{list_auctions, create_auction};
use serde_json::json;

#[actix_rt::test]
async fn test_create_auction_dto() {
    let dto = CreateAuctionDto {
        user_id: Uuid::new_v4(),
        item_id: Uuid::new_v4(),
        title: "Test Auction".to_string(),
        description: Some("desc".to_string()),
        start_time: NaiveDateTime::UNIX_EPOCH + Duration::seconds(1_600_000_000),
        end_time: NaiveDateTime::UNIX_EPOCH + Duration::seconds(1_600_000_100),
        base_price: Decimal::new(1000, 2),
        min_bid_increment: Decimal::new(100, 2),
        status: "active".to_string(),
    };
    assert_eq!(dto.title, "Test Auction");
    assert_eq!(dto.status, "active");
}

#[test]
async fn test_update_auction_dto() {
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

#[actix_rt::test]
async fn integration_list_auctions() {
    let db = db::connect().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .configure(auctions_routes)
    ).await;

    let req = test::TestRequest::get().uri("/auctions").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn integration_create_auction() {
    let db = db::connect().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .configure(auctions_routes)
    ).await;

    let payload = json!({
        "user_id": Uuid::new_v4(),
        "item_id": Uuid::new_v4(),
        "title": "Integration Auction",
        "description": "desc",
        "start_time": "2020-09-13T12:26:40",
        "end_time": "2020-09-13T12:28:20",
        "base_price": "10.00",
        "min_bid_increment": "1.00",
        "status": "active"
    });

    let req = test::TestRequest::post()
        .uri("/auctions")
        .set_json(&payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn integration_create_auction_with_dto() {
    let db = db::connect().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .configure(auctions_routes)
    ).await;

    let dto = CreateAuctionDto {
        user_id: Uuid::new_v4(),
        item_id: Uuid::new_v4(),
        title: "Integration Auction DTO".to_string(),
        description: Some("desc desde DTO".to_string()),
        start_time: chrono::NaiveDateTime::from_timestamp_opt(1_600_000_000, 0).unwrap(),
        end_time: chrono::NaiveDateTime::from_timestamp_opt(1_600_000_100, 0).unwrap(),
        base_price: Decimal::new(1000, 2),
        min_bid_increment: Decimal::new(100, 2),
        status: "active".to_string(),
    };

    let payload = serde_json::to_value(&dto).unwrap();

    let req = test::TestRequest::post()
        .uri("/auctions")
        .set_json(&payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
