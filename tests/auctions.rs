use auction_ms::handlers::auctions::{CreateAuctionDto, UpdateAuctionDto};
use auction_ms::routes::auctions::auctions_routes;
use auction_ms::db;
use uuid::Uuid;
use chrono::{NaiveDateTime, Duration};
use rust_decimal::Decimal;
use actix_web::{test, web, App};
use auction_ms::handlers::auctions::{list_auctions, create_auction};
use serde_json::json;
use std::env;

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
async fn integration_create_auction() {
    let db_url = env::var("DATABASE_URL_TEST").expect("DATABASE_URL_TEST no está definida");
    let db = db::connect_with_url(&db_url).await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db.clone()))
            .configure(auctions_routes)
    ).await;

    let payload = json!({
        "user_id": Uuid::new_v4(),
        "item_id": Uuid::new_v4(),
        "title": "Create Auction",
        "description": "desc",
        "start_time": "2020-09-13T12:26:40",
        "end_time": "2020-09-13T12:28:20",
        "base_price": "10.00",
        "min_bid_increment": "1.00",
        "status": "active"
    });
    let req = test::TestRequest::post().uri("/auctions").set_json(&payload).to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(created["title"], "Create Auction");
}

#[actix_rt::test]
async fn integration_list_auctions() {
    let db_url = env::var("DATABASE_URL_TEST").expect("DATABASE_URL_TEST no está definida");
    let db = db::connect_with_url(&db_url).await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db.clone()))
            .configure(auctions_routes)
    ).await;

    // Crea una subasta para asegurar que la lista no esté vacía
    let payload = json!({
        "user_id": Uuid::new_v4(),
        "item_id": Uuid::new_v4(),
        "title": "List Auction",
        "description": "desc",
        "start_time": "2020-09-13T12:26:40",
        "end_time": "2020-09-13T12:28:20",
        "base_price": "10.00",
        "min_bid_increment": "1.00",
        "status": "active"
    });
    let req = test::TestRequest::post().uri("/auctions").set_json(&payload).to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::get().uri("/auctions").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let auctions: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let found = auctions.as_array().unwrap().iter().any(|a| a["title"] == "List Auction");
    assert!(found, "La subasta creada no fue encontrada en el listado");
}

#[actix_rt::test]
async fn integration_get_auction() {
    let db_url = env::var("DATABASE_URL_TEST").expect("DATABASE_URL_TEST no está definida");
    let db = db::connect_with_url(&db_url).await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db.clone()))
            .configure(auctions_routes)
    ).await;

    // Crea una subasta y obtén su id
    let payload = json!({
        "user_id": Uuid::new_v4(),
        "item_id": Uuid::new_v4(),
        "title": "Get Auction",
        "description": "desc",
        "start_time": "2020-09-13T12:26:40",
        "end_time": "2020-09-13T12:28:20",
        "base_price": "10.00",
        "min_bid_increment": "1.00",
        "status": "active"
    });
    let req = test::TestRequest::post().uri("/auctions").set_json(&payload).to_request();
    let resp = test::call_service(&app, req).await;
    let body = test::read_body(resp).await;
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let auction_id = created["id"].as_str().unwrap();

    let req = test::TestRequest::get().uri(&format!("/auctions/{}", auction_id)).to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let auction: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(auction["id"], auction_id);
}

#[actix_rt::test]
async fn integration_update_auction() {
    let db_url = env::var("DATABASE_URL_TEST").expect("DATABASE_URL_TEST no está definida");
    let db = db::connect_with_url(&db_url).await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db.clone()))
            .configure(auctions_routes)
    ).await;

    // Crea una subasta y obtén su id
    let payload = json!({
        "user_id": Uuid::new_v4(),
        "item_id": Uuid::new_v4(),
        "title": "Update Auction",
        "description": "desc",
        "start_time": "2020-09-13T12:26:40",
        "end_time": "2020-09-13T12:28:20",
        "base_price": "10.00",
        "min_bid_increment": "1.00",
        "status": "active"
    });
    let req = test::TestRequest::post().uri("/auctions").set_json(&payload).to_request();
    let resp = test::call_service(&app, req).await;
    let body = test::read_body(resp).await;
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let auction_id = created["id"].as_str().unwrap();

    // Actualiza la subasta
    let update_payload = json!({
        "title": "Updated Auction",
        "status": "finished"
    });
    let req = test::TestRequest::put()
        .uri(&format!("/auctions/{}", auction_id))
        .set_json(&update_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["title"], "Updated Auction");
    assert_eq!(updated["status"], "finished");
}

#[actix_rt::test]
async fn integration_delete_auction() {
    let db_url = env::var("DATABASE_URL_TEST").expect("DATABASE_URL_TEST no está definida");
    let db = db::connect_with_url(&db_url).await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db.clone()))
            .configure(auctions_routes)
    ).await;

    // Crea una subasta y obtén su id
    let payload = json!({
        "user_id": Uuid::new_v4(),
        "item_id": Uuid::new_v4(),
        "title": "Delete Auction",
        "description": "desc",
        "start_time": "2020-09-13T12:26:40",
        "end_time": "2020-09-13T12:28:20",
        "base_price": "10.00",
        "min_bid_increment": "1.00",
        "status": "active"
    });
    let req = test::TestRequest::post().uri("/auctions").set_json(&payload).to_request();
    let resp = test::call_service(&app, req).await;
    let body = test::read_body(resp).await;
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let auction_id = created["id"].as_str().unwrap();

    // Elimina la subasta
    let req = test::TestRequest::delete().uri(&format!("/auctions/{}", auction_id)).to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 204);

    // Verifica que ya no existe
    let req = test::TestRequest::get().uri(&format!("/auctions/{}", auction_id)).to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 404);
}
