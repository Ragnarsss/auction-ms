use tonic::{transport::Server, Request, Response, Status};
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set};
use uuid::Uuid;
use crate::models::auction::{Entity as AuctionEntity, ActiveModel as AuctionActiveModel, Model as AuctionModel};
use prost_types::Timestamp;


pub mod auction {
    tonic::include_proto!("auction");
}

use auction::auction_service_server::{AuctionService, AuctionServiceServer};
use auction::*;

#[derive(Clone)]
pub struct MyAuctionService {
    db: DatabaseConnection,
}

#[tonic::async_trait]
impl AuctionService for MyAuctionService {
    async fn create_auction(
        &self,
        request: Request<CreateAuctionRequest>,
    ) -> Result<Response<CreateAuctionResponse>, Status> {
        let req = request.into_inner();
        let auction = AuctionActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(Uuid::parse_str(&req.user_id).map_err(|_| Status::invalid_argument("user_id inválido"))?),
            item_id: Set(Uuid::parse_str(&req.item_id).map_err(|_| Status::invalid_argument("item_id inválido"))?),
            title: Set(req.title.clone()),
            description: Set(Some(req.description.clone())),
            start_time: Set(proto_timestamp_to_naive(&req.start_time)?),
            end_time: Set(proto_timestamp_to_naive(&req.end_time)?),
            base_price: Set(req.base_price.parse().map_err(|_| Status::invalid_argument("base_price inválido"))?),
            min_bid_increment: Set(req.min_bid_increment.parse().map_err(|_| Status::invalid_argument("min_bid_increment inválido"))?),
            highest_bid: Set(None),
            status: Set(req.status.clone()),
        };
        let inserted = auction.insert(&self.db).await.map_err(|e| Status::internal(format!("DB error: {}", e)))?;
        let proto_auction = map_model_to_proto(&inserted);
        Ok(Response::new(CreateAuctionResponse {
            auction: Some(proto_auction),
        }))
    }

    async fn list_auctions(
        &self,
        _request: Request<ListAuctionsRequest>,
    ) -> Result<Response<ListAuctionsResponse>, Status> {
        let auctions = AuctionEntity::find()
            .all(&self.db)
            .await
            .map_err(|e| Status::internal(format!("DB error: {}", e)))?;
        let proto_auctions = auctions.iter().map(map_model_to_proto).collect();
        Ok(Response::new(ListAuctionsResponse {
            auctions: proto_auctions,
        }))
    }

    async fn get_auction(
        &self,
        request: Request<GetAuctionRequest>,
    ) -> Result<Response<GetAuctionResponse>, Status> {
        let req = request.into_inner();
        let id = uuid::Uuid::parse_str(&req.id).map_err(|_| Status::invalid_argument("id inválido"))?;
        let auction = AuctionEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| Status::internal(format!("DB error: {}", e)))?;
        match auction {
            Some(model) => Ok(Response::new(GetAuctionResponse {
                auction: Some(map_model_to_proto(&model)),
            })),
            None => Err(Status::not_found("Subasta no encontrada")),
        }
    }

    async fn update_auction(
        &self,
        request: Request<UpdateAuctionRequest>,
    ) -> Result<Response<UpdateAuctionResponse>, Status> {
        let req = request.into_inner();
        let id = uuid::Uuid::parse_str(&req.id).map_err(|_| Status::invalid_argument("id inválido"))?;
        let found = AuctionEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| Status::internal(format!("DB error: {}", e)))?;
        let Some(model) = found else {
            return Err(Status::not_found("Subasta no encontrada"));
        };
        let mut active: AuctionActiveModel = model.into();
        if !req.title.is_empty() { active.title = Set(req.title); }
        if !req.description.is_empty() { active.description = Set(Some(req.description)); }
        if let Some(ts) = req.start_time { active.start_time = Set(proto_timestamp_to_naive(&Some(ts))?); }
        if let Some(ts) = req.end_time { active.end_time = Set(proto_timestamp_to_naive(&Some(ts))?); }
        if !req.base_price.is_empty() { active.base_price = Set(req.base_price.parse().map_err(|_| Status::invalid_argument("base_price inválido"))?); }
        if !req.min_bid_increment.is_empty() { active.min_bid_increment = Set(req.min_bid_increment.parse().map_err(|_| Status::invalid_argument("min_bid_increment inválido"))?); }
        if !req.highest_bid.is_empty() { active.highest_bid = Set(Some(req.highest_bid.parse().map_err(|_| Status::invalid_argument("highest_bid inválido"))?)); }
        if !req.status.is_empty() { active.status = Set(req.status); }
        let updated = active.update(&self.db).await.map_err(|e| Status::internal(format!("DB error: {}", e)))?;
        Ok(Response::new(UpdateAuctionResponse {
            auction: Some(map_model_to_proto(&updated)),
        }))
    }

    async fn delete_auction(
        &self,
        request: Request<DeleteAuctionRequest>,
    ) -> Result<Response<auction::Empty>, Status> {
        let req = request.into_inner();
        let id = uuid::Uuid::parse_str(&req.id).map_err(|_| Status::invalid_argument("id inválido"))?;
        AuctionEntity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| Status::internal(format!("DB error: {}", e)))?;
        Ok(Response::new(auction::Empty {}))
    }
}

fn proto_timestamp_to_naive(ts: &Option<Timestamp>) -> Result<chrono::NaiveDateTime, Status> {
    let t = ts.as_ref().ok_or(Status::invalid_argument("timestamp faltante"))?;
    chrono::NaiveDateTime::from_timestamp_opt(t.seconds, t.nanos as u32)
        .ok_or(Status::invalid_argument("timestamp inválido"))
}

fn naive_to_proto_timestamp(dt: &chrono::NaiveDateTime) -> Option<Timestamp> {
    let dt_utc = dt.and_utc();
    Some(Timestamp {
        seconds: dt_utc.timestamp(),
        nanos: dt_utc.timestamp_subsec_nanos() as i32,
    })
}

fn map_model_to_proto(model: &AuctionModel) -> auction::Auction {
    auction::Auction {
        id: model.id.to_string(),
        user_id: model.user_id.to_string(),
        item_id: model.item_id.to_string(),
        title: model.title.clone(),
        description: model.description.clone().unwrap_or_default(),
        start_time: naive_to_proto_timestamp(&model.start_time),
        end_time: naive_to_proto_timestamp(&model.end_time),
        base_price: model.base_price.to_string(),
        min_bid_increment: model.min_bid_increment.to_string(),
        highest_bid: model.highest_bid.map(|v| v.to_string()).unwrap_or_default(),
        status: model.status.clone(),
    }
}

pub async fn start_grpc_server() -> Result<(), Box<dyn std::error::Error>> {
    let db = crate::db::connect().await;
    let service = MyAuctionService { db };
    let addr = "[::1]:50051".parse()?;
    Server::builder()
        .add_service(AuctionServiceServer::new(service))
        .serve(addr)
        .await?;
    Ok(())
}
