use actix_web::web;
use crate::handlers::auctions::{
    list_auctions, get_auction, create_auction, update_auction, delete_auction
};

pub fn auctions_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_auctions)
        .service(get_auction)
        .service(create_auction)
        .service(update_auction)
        .service(delete_auction);
}
