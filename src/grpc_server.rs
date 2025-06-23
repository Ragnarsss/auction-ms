use tonic::{transport::Server, Request, Response, Status};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set}; 
use uuid::Uuid;
use crate::models::auction::{Entity as AuctionEntity, ActiveModel as AuctionActiveModel, Model as AuctionModel};
use crate::models::bid::{Entity as BidEntity, ActiveModel as BidActiveModel, Model as BidModel};
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

// Enum para los estados válidos de subasta
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuctionStatus {
    Pending,
    Active, 
    Completed,
    Cancelled,
}

impl AuctionStatus {
    fn as_str(&self) -> &'static str {
        match self {
            AuctionStatus::Pending => "pending",
            AuctionStatus::Active => "active",
            AuctionStatus::Completed => "completed",
            AuctionStatus::Cancelled => "cancelled",
        }
    }

    fn from_str(status: &str) -> Result<Self, Status> {
        match status.to_lowercase().as_str() {
            "pending" => Ok(AuctionStatus::Pending),
            "active" => Ok(AuctionStatus::Active),
            "completed" => Ok(AuctionStatus::Completed),
            "cancelled" => Ok(AuctionStatus::Cancelled),
            _ => Err(Status::invalid_argument(
                "Status inválido. Valores permitidos: pending, active, completed, cancelled"
            )),
        }
    }

    fn all_valid_statuses() -> Vec<&'static str> {
        vec!["pending", "active", "completed", "cancelled"]
    }
}

// Enum para las monedas válidas
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuctionCurrency {
    USD,
    EUR, 
    CLP,
    ARS,
    BRL,
    MXN,
}

impl AuctionCurrency {
    fn as_str(&self) -> &'static str {
        match self {
            AuctionCurrency::USD => "USD",
            AuctionCurrency::EUR => "EUR",
            AuctionCurrency::CLP => "CLP",
            AuctionCurrency::ARS => "ARS",
            AuctionCurrency::BRL => "BRL",
            AuctionCurrency::MXN => "MXN",
        }
    }

    fn from_str(currency: &str) -> Result<Self, Status> {
        match currency.to_uppercase().as_str() {
            "USD" => Ok(AuctionCurrency::USD),
            "EUR" => Ok(AuctionCurrency::EUR),
            "CLP" => Ok(AuctionCurrency::CLP),
            "ARS" => Ok(AuctionCurrency::ARS),
            "BRL" => Ok(AuctionCurrency::BRL),
            "MXN" => Ok(AuctionCurrency::MXN),
            _ => Err(Status::invalid_argument(
                "Moneda inválida. Valores permitidos: USD, EUR, CLP, ARS, BRL, MXN"
            )),
        }
    }

    fn all_valid_currencies() -> Vec<&'static str> {
        vec!["USD", "EUR", "CLP", "ARS", "BRL", "MXN"]
    }
}

#[tonic::async_trait]
impl AuctionService for MyAuctionService {    
    async fn create_auction(
        &self,
        request: Request<CreateAuctionRequest>,
    ) -> Result<Response<CreateAuctionResponse>, Status> {
        log::info!("=== INICIANDO CREACIÓN DE SUBASTA ===");
        let req = request.into_inner();
        
        // Debug: Log the raw request structure
        log::debug!("Raw request: {:?}", req);
        log::debug!("Category field specifically: '{}'", req.category);
        log::debug!("Category field length: {}", req.category.len());
        log::debug!("Category field is_empty(): {}", req.category.is_empty());
        
        log::info!("Datos recibidos: user_id={}, item_id={}, title={}, description={}, category='{}', base_price={}, min_bid_increment={}, currency={}", 
            req.user_id, req.item_id, req.title, req.description, req.category, req.base_price, req.min_bid_increment, req.currency);
        
        // Validar fechas antes de crear la subasta
        log::debug!("Validando fechas de inicio y fin");
        let start_time = match proto_timestamp_to_naive(&req.start_time) {
            Ok(time) => {
                log::debug!("Fecha de inicio convertida: {}", time);
                time
            },
            Err(e) => {
                log::error!("Error al convertir fecha de inicio: {:?}", e);
                return Err(e);
            }
        };
        
        let end_time = match proto_timestamp_to_naive(&req.end_time) {
            Ok(time) => {
                log::debug!("Fecha de fin convertida: {}", time);
                time
            },
            Err(e) => {
                log::error!("Error al convertir fecha de fin: {:?}", e);
                return Err(e);
            }
        };
        
        if let Err(e) = validate_date_range(&start_time, &end_time) {
            log::error!("Error en validación de rango de fechas: {:?}", e);
            return Err(e);
        }
        log::info!("Validación de fechas exitosa");
        
        // Validar IDs como strings
        log::debug!("Validando IDs");
        if req.user_id.is_empty() {
            log::error!("user_id vacío");
            return Err(Status::invalid_argument("user_id no puede estar vacío"));
        }
        
        if req.item_id.is_empty() {
            log::error!("item_id vacío");
            return Err(Status::invalid_argument("item_id no puede estar vacío"));
        }

        // Validar que category no esté vacía
        if req.category.is_empty() {
            log::error!("category vacía - valor recibido: '{}'", req.category);
            log::error!("category bytes: {:?}", req.category.as_bytes());
            return Err(Status::invalid_argument("category no puede estar vacía"));
        }
        
        // También validar que no contenga solo espacios en blanco
        if req.category.trim().is_empty() {
            log::error!("category contiene solo espacios en blanco: '{}'", req.category);
            return Err(Status::invalid_argument("category no puede contener solo espacios en blanco"));
        }
        
        log::debug!("Category válida después de validación: '{}'", req.category);
        
        log::debug!("User ID válido (string): {}", req.user_id);
        log::debug!("Item ID válido (string): {}", req.item_id);
        log::debug!("Category válida: {}", req.category);
        
        // Validar y parsear precios (verificar que sean números válidos)
        log::debug!("Validando precios como números");
        let base_price = validate_numeric_string(&req.base_price, "base_price")?;
        let min_bid_increment = validate_numeric_string(&req.min_bid_increment, "min_bid_increment")?;
        
        // Validar moneda
        let currency = if req.currency.is_empty() {
            log::info!("Currency no especificada, usando USD por defecto");
            AuctionCurrency::USD
        } else {
            AuctionCurrency::from_str(&req.currency)?
        };
        log::debug!("Moneda válida: {}", currency.as_str());
        
        // Las subastas siempre se crean en estado "pending" por defecto en la base de datos
        let auction_status = AuctionStatus::Pending;
        log::info!("Creando subasta en estado: {}", auction_status.as_str());
        log::info!("Creando subasta con categoría: {}", req.category);
        
        let auction_id = Uuid::new_v4();
        log::info!("Generado nuevo ID de subasta: {}", auction_id);
        
        let auction = AuctionActiveModel {
            id: Set(auction_id),
            user_id: Set(req.user_id.clone()),
            item_id: Set(req.item_id.clone()),
            title: Set(req.title.clone()),
            description: Set(Some(req.description.clone())),
            category: Set(req.category.trim().to_string()), // Usar trim por si hay espacios
            start_time: Set(start_time),
            end_time: Set(end_time),
            base_price: Set(base_price),
            min_bid_increment: Set(min_bid_increment),
            highest_bid: Set(Default::default()),
            status: Set(auction_status.as_str().to_string()),
            currency: Set(currency.as_str().to_string()),
        };
        
        log::info!("Modelo creado - Intentando insertar en DB...");
        log::debug!("Valores del modelo: id={}, user_id={}, category={}, status={}, currency={}", 
            auction_id, req.user_id, req.category, auction_status.as_str(), currency.as_str());
            
        match auction.insert(&self.db).await {
            Ok(inserted) => {
                log::info!("✅ Subasta creada exitosamente con ID: {}", inserted.id);
                log::info!("✅ Datos verificados - categoría guardada: '{}', estado guardado: '{}'", 
                    inserted.category, inserted.status);
                log::debug!("Datos de subasta insertada: título='{}', usuario='{}', item='{}', categoría='{}', precio_base={}, moneda='{}'", 
                    inserted.title, inserted.user_id, inserted.item_id, inserted.category, inserted.base_price, inserted.currency);
                let proto_auction = map_model_to_proto(&inserted);
                log::info!("=== CREACIÓN DE SUBASTA COMPLETADA ===");
                Ok(Response::new(CreateAuctionResponse {
                    auction: Some(proto_auction),
                }))
            },
            Err(e) => {
                log::error!("❌ Error al insertar subasta en base de datos: {}", e);
                log::error!("Tipo de error: {:?}", e);
                log::error!("Detalles completos del error: {:#?}", e);
                
                // Intentar obtener más detalles del error de base de datos
                let error_msg = format!("Error de base de datos: {}. Verifique que la tabla 'auction' exista y tenga todas las columnas requeridas.", e);
                Err(Status::internal(error_msg))
            }
        }
    }

    async fn list_auctions(
        &self,
        _request: Request<ListAuctionsRequest>,
    ) -> Result<Response<ListAuctionsResponse>, Status> {
        log::info!("Recibida solicitud list_auctions");
        let auctions = AuctionEntity::find()
            .all(&self.db)
            .await
            .map_err(|e| Status::internal(format!("DB error: {}", e)))?;

        let mut proto_auctions = Vec::new();
        
        // Para cada subasta, obtener sus pujas
        for auction in auctions {
            let bids = BidEntity::find()
                .filter(crate::models::bid::Column::AuctionId.eq(auction.id))
                .order_by_desc(crate::models::bid::Column::CreatedAt)
                .all(&self.db)
                .await
                .map_err(|e| Status::internal(format!("DB error: {}", e)))?;
            
            let proto_auction = map_model_to_proto_with_bids(&auction, &bids);
            proto_auctions.push(proto_auction);
        }
        
        log::info!("Retornando {} subastas con sus pujas incluidas", proto_auctions.len());
        Ok(Response::new(ListAuctionsResponse {
            auctions: proto_auctions,
        }))
    }

    async fn get_auction(
        &self,
        request: Request<GetAuctionRequest>,
    ) -> Result<Response<GetAuctionResponse>, Status> {
        let req = request.into_inner();
        log::info!("Recibida solicitud get_auction con id={}", req.id);
        let id = uuid::Uuid::parse_str(&req.id).map_err(|_| Status::invalid_argument("id inválido"))?;
        
        // Obtener la subasta
        let auction = AuctionEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| Status::internal(format!("DB error: {}", e)))?;
        
        let Some(auction_model) = auction else {
            return Err(Status::not_found("Subasta no encontrada"));
        };

        // Obtener todas las pujas de la subasta
        let bids = BidEntity::find()
            .filter(crate::models::bid::Column::AuctionId.eq(id))
            .order_by_desc(crate::models::bid::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| Status::internal(format!("DB error: {}", e)))?;

        // Convertir a proto con las pujas incluidas
        let proto_auction = map_model_to_proto_with_bids(&auction_model, &bids);
        
        Ok(Response::new(GetAuctionResponse {
            auction: Some(proto_auction),
        }))
    }

    async fn update_auction(
        &self,
        request: Request<UpdateAuctionRequest>,
    ) -> Result<Response<UpdateAuctionResponse>, Status> {
        let req = request.into_inner();
        log::info!("Recibida solicitud update_auction con id={}", req.id);
        let id = uuid::Uuid::parse_str(&req.id).map_err(|_| Status::invalid_argument("id inválido"))?;
        let found = AuctionEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| Status::internal(format!("DB error: {}", e)))?;
        let Some(model) = found else {
            return Err(Status::not_found("Subasta no encontrada"));
        };
        let mut active: AuctionActiveModel = model.into();
        
        if !req.title.is_empty() { 
            active.title = Set(req.title); 
        }
        if !req.description.is_empty() { 
            active.description = Set(Some(req.description)); 
        }
        if !req.category.is_empty() { 
            active.category = Set(req.category); 
        }
        if let Some(ts) = req.start_time { 
            active.start_time = Set(proto_timestamp_to_naive(&Some(ts))?); 
        }
        if let Some(ts) = req.end_time { 
            active.end_time = Set(proto_timestamp_to_naive(&Some(ts))?); 
        }
        if !req.base_price.is_empty() { 
            active.base_price = Set(validate_numeric_string(&req.base_price, "base_price")?); 
        }
        if !req.min_bid_increment.is_empty() { 
            active.min_bid_increment = Set(validate_numeric_string(&req.min_bid_increment, "min_bid_increment")?); 
        }
        if !req.highest_bid.is_empty() { 
            active.highest_bid = Set(Some(validate_numeric_string(&req.highest_bid, "highest_bid")?)); 
        }
        
        // Validar y actualizar currency si se proporciona
        if !req.currency.is_empty() {
            let new_currency = AuctionCurrency::from_str(&req.currency)?;
            log::info!("Cambiando currency de subasta a: {}", new_currency.as_str());
            active.currency = Set(new_currency.as_str().to_string());
        }
        
        // Validar y actualizar status si se proporciona
        if !req.status.is_empty() { 
            let new_status = AuctionStatus::from_str(&req.status)?;
            log::info!("Cambiando status de subasta a: {}", new_status.as_str());
            
            // Si el status cambia a "active", establecer start_time al momento actual
            if new_status == AuctionStatus::Active {
                log::info!("Activando subasta - estableciendo start_time al momento actual");
                active.start_time = Set(chrono::Utc::now().naive_utc());
            }
            active.status = Set(new_status.as_str().to_string()); 
        }
        
        let updated = active.update(&self.db).await.map_err(|e| Status::internal(format!("DB error: {}", e)))?;
        log::info!("Subasta actualizada exitosamente con ID: {}", updated.id);
        Ok(Response::new(UpdateAuctionResponse {
            auction: Some(map_model_to_proto(&updated)),
        }))
    }

    async fn delete_auction(
        &self,
        request: Request<DeleteAuctionRequest>,
    ) -> Result<Response<auction::Empty>, Status> {
        let req = request.into_inner();
        log::info!("Recibida solicitud delete_auction con id={}", req.id);
        let id = uuid::Uuid::parse_str(&req.id).map_err(|_| Status::invalid_argument("id inválido"))?;
        AuctionEntity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| Status::internal(format!("DB error: {}", e)))?;
        Ok(Response::new(auction::Empty {}))
    }

    async fn create_bid(
        &self,
        request: Request<CreateBidRequest>,
    ) -> Result<Response<CreateBidResponse>, Status> {
        log::info!("Recibida solicitud create_bid");
        let req = request.into_inner();
        log::info!("Datos recibidos: auction_id={}, user_id={}, amount={}", req.auction_id, req.user_id, req.amount);

        // Validar que la subasta existe
        let auction_id = Uuid::parse_str(&req.auction_id)
            .map_err(|_| Status::invalid_argument("auction_id inválido"))?;
        
        // Validar user_id como string (no UUID)
        if req.user_id.is_empty() {
            return Err(Status::invalid_argument("user_id no puede estar vacío"));
        }

        // Validar amount como número
        let bid_amount = validate_numeric_string(&req.amount, "amount")?;

        // Validar que la subasta esté activa
        let auction = AuctionEntity::find_by_id(auction_id)
            .one(&self.db)
            .await
            .map_err(|e| Status::internal(format!("DB error: {}", e)))?;
        
        let Some(auction_model) = auction else {
            return Err(Status::not_found("Subasta no encontrada"));
        };

        // Validar que la subasta esté activa usando el enum
        let current_status = AuctionStatus::from_str(&auction_model.status)?;
        if current_status != AuctionStatus::Active {
            return Err(Status::failed_precondition(
                format!("La subasta debe estar en estado 'active'. Estado actual: '{}'", 
                    auction_model.status)
            ));
        }

        // Validar que la subasta no haya terminado
        let now = chrono::Utc::now().naive_utc();
        if now > auction_model.end_time {
            return Err(Status::failed_precondition("La subasta ha terminado"));
        }

        // Validar que la subasta haya comenzado
        if now < auction_model.start_time {
            return Err(Status::failed_precondition("La subasta aún no ha comenzado"));
        }

        // Validar el monto de la puja
        let current_highest = auction_model.highest_bid.unwrap_or_default();
        if bid_amount <= current_highest {
            return Err(Status::failed_precondition("La puja debe ser mayor que la puja más alta actual"));
        }

        if bid_amount < auction_model.base_price {
            return Err(Status::failed_precondition("La puja debe ser mayor o igual al precio base"));
        }

        // Validar incremento mínimo
        let min_required = current_highest + auction_model.min_bid_increment;
        if bid_amount < min_required {
            return Err(Status::failed_precondition(
                format!("La puja debe ser al menos {}", min_required)
            ));
        }

        // Crear la puja
        let bid = BidActiveModel {
            id: Set(Uuid::new_v4()),
            auction_id: Set(auction_id),
            user_id: Set(req.user_id.clone()),
            amount: Set(bid_amount),
            created_at: Set(chrono::Utc::now().naive_utc()),
            status: Set("active".to_string()),
        };

        let inserted_bid = bid.insert(&self.db).await
            .map_err(|e| {
                log::error!("Error al insertar puja: {}", e);
                Status::internal(format!("DB error: {}", e))
            })?;

        // Actualizar la puja más alta en la subasta
        let mut auction_active: AuctionActiveModel = auction_model.into();
        auction_active.highest_bid = Set(Some(bid_amount));
        
        auction_active.update(&self.db).await
            .map_err(|e| {
                log::error!("Error al actualizar subasta: {}", e);
                Status::internal(format!("DB error: {}", e))
            })?;

        log::info!("Puja creada con id {}", inserted_bid.id);
        let proto_bid = map_bid_model_to_proto(&inserted_bid);
        
        Ok(Response::new(CreateBidResponse {
            bid: Some(proto_bid),
        }))
    }

    async fn list_bids(
        &self,
        request: Request<ListBidsRequest>,
    ) -> Result<Response<ListBidsResponse>, Status> {
        let req = request.into_inner();
        log::info!("Recibida solicitud list_bids para auction_id={}", req.auction_id);
        
        let auction_id = Uuid::parse_str(&req.auction_id)
            .map_err(|_| Status::invalid_argument("auction_id inválido"))?;

        let bids = BidEntity::find()
            .filter(crate::models::bid::Column::AuctionId.eq(auction_id))
            .all(&self.db)
            .await
            .map_err(|e| Status::internal(format!("DB error: {}", e)))?;

        let proto_bids = bids.iter().map(map_bid_model_to_proto).collect();
        
        Ok(Response::new(ListBidsResponse {
            bids: proto_bids,
        }))
    }

    async fn get_highest_bid(
        &self,
        request: Request<GetHighestBidRequest>,
    ) -> Result<Response<GetHighestBidResponse>, Status> {
        let req = request.into_inner();
        let auction_id = Uuid::parse_str(&req.auction_id)
            .map_err(|_| Status::invalid_argument("auction_id inválido"))?;

        let highest_bid = BidEntity::find()
            .filter(crate::models::bid::Column::AuctionId.eq(auction_id))
            .order_by_desc(crate::models::bid::Column::Amount)
            .one(&self.db)
            .await
            .map_err(|e| Status::internal(format!("DB error: {}", e)))?;

        match highest_bid {
            Some(bid) => Ok(Response::new(GetHighestBidResponse {
                bid: Some(map_bid_model_to_proto(&bid)),
            })),
            None => Err(Status::not_found("No hay pujas para esta subasta")),
        }
    }
}

// Función helper para convertir modelo de puja a proto
fn map_bid_model_to_proto(model: &BidModel) -> auction::Bid {
    auction::Bid {
        id: model.id.to_string(),
        auction_id: model.auction_id.to_string(),
        user_id: model.user_id.to_string(),
        amount: model.amount.to_string(),
        created_at: naive_to_proto_timestamp(&model.created_at),
        status: model.status.clone(),
    }
}

fn proto_timestamp_to_naive(ts: &Option<Timestamp>) -> Result<chrono::NaiveDateTime, Status> {
    let t = ts.as_ref().ok_or(Status::invalid_argument("timestamp faltante"))?;
    // Usar DateTime::from_timestamp en lugar de NaiveDateTime::from_timestamp_opt
    Ok(chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32)
        .ok_or(Status::invalid_argument("timestamp inválido"))?
        .naive_utc())
}

fn naive_to_proto_timestamp(dt: &chrono::NaiveDateTime) -> Option<Timestamp> {
    let dt_utc = dt.and_utc();
    Some(Timestamp {
        seconds: dt_utc.timestamp(),
        nanos: dt_utc.timestamp_subsec_nanos() as i32,
    })
}

// Función para validar rangos de fechas (útil para frontend)
fn validate_date_range(start: &chrono::NaiveDateTime, end: &chrono::NaiveDateTime) -> Result<(), Status> {
    if start >= end {
        return Err(Status::invalid_argument("La fecha de inicio debe ser anterior a la fecha de fin"));
    }
    
    let now = chrono::Utc::now().naive_utc();
    if start < &now {
        return Err(Status::invalid_argument("La fecha de inicio no puede ser en el pasado"));
    }
    
    Ok(())
}

// Función para crear timestamp desde ISO string (útil para recibir fechas desde React)
fn iso_string_to_timestamp(iso_string: &str) -> Result<Timestamp, Status> {
    let dt = chrono::DateTime::parse_from_rfc3339(iso_string)
        .map_err(|_| Status::invalid_argument("Formato de fecha ISO inválido"))?;
    
    Ok(Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    })
}

fn map_model_to_proto(model: &AuctionModel) -> auction::Auction {
    auction::Auction {
        id: model.id.to_string(),
        user_id: model.user_id.to_string(),
        item_id: model.item_id.to_string(),
        title: model.title.clone(),
        description: model.description.clone().unwrap_or_default(),
        category: model.category.clone(),
        start_time: naive_to_proto_timestamp(&model.start_time),
        end_time: naive_to_proto_timestamp(&model.end_time),
        base_price: model.base_price.to_string(),
        min_bid_increment: model.min_bid_increment.to_string(),
        highest_bid: model.highest_bid.map_or("0".to_string(), |hb| hb.to_string()),
        status: model.status.clone(),
        currency: model.currency.clone(),
        bids: vec![], 
    }
}

fn map_model_to_proto_with_bids(model: &AuctionModel, bids: &Vec<BidModel>) -> auction::Auction {
    let mut proto_auction = map_model_to_proto(model);
    proto_auction.bids = bids.iter().map(map_bid_model_to_proto).collect();
    proto_auction
}

pub async fn start_grpc_server() -> Result<(), Box<dyn std::error::Error>> {
    let db = crate::db::connect().await;
    let service = MyAuctionService { db };
let addr = std::env::var("GRPC_ADDRESS").unwrap_or_else(|_| "0.0.0.0:50052".to_string());
    let addr = addr.parse().unwrap();
    log::info!("Servidor gRPC escuchando en {}", addr);
    Server::builder()
        .add_service(AuctionServiceServer::new(service))
        .serve(addr)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{Database, DatabaseConnection, Statement, ConnectionTrait};
    use tonic::Request;

    async fn setup_test_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        
        // Crear tabla auction con status y currency
        db.execute(Statement::from_string(
            db.get_database_backend(),
            r#"
CREATE TABLE auction (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    item_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    category TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    base_price NUMERIC NOT NULL,
    min_bid_increment NUMERIC NOT NULL,
    highest_bid NUMERIC NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    currency TEXT NOT NULL DEFAULT 'USD'
);
            "#.to_owned(),
        )).await.unwrap();

        // Crear tabla bid
        db.execute(Statement::from_string(
            db.get_database_backend(),
            r#"
CREATE TABLE bid (
    id TEXT PRIMARY KEY,
    auction_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    amount NUMERIC NOT NULL,
    created_at TEXT NOT NULL,
    status TEXT NOT NULL,
    FOREIGN KEY (auction_id) REFERENCES auction (id)
);
            "#.to_owned(),
        )).await.unwrap();
        
        db
    }

    async fn setup_service() -> MyAuctionService {
        let db = setup_test_db().await;
        MyAuctionService { db }
    }

    #[tokio::test]
    async fn test_list_auctions_empty() {
        let service = setup_service().await;
        let req = Request::new(ListAuctionsRequest {});
        let response = service.list_auctions(req).await.unwrap().into_inner();
        assert_eq!(response.auctions.len(), 0);
    }

    #[tokio::test]
    async fn test_create_auction_ok() {
        let service = setup_service().await;
        let req = CreateAuctionRequest {
            user_id: uuid::Uuid::new_v4().to_string(),
            item_id: uuid::Uuid::new_v4().to_string(),
            title: "Test Auction".to_string(),
            description: "desc".to_string(),
            category: "Electronics".to_string(),
            start_time: Some(prost_types::Timestamp { seconds: 1_600_000_000, nanos: 0 }),
            end_time: Some(prost_types::Timestamp { seconds: 1_600_000_100, nanos: 0 }),
            base_price: "100.00".to_string(),
            min_bid_increment: "10.00".to_string(),
            highest_bid: "".to_string(),
            currency: "EUR".to_string(), // Prueba con moneda diferente
        };
        let response = service.create_auction(Request::new(req)).await.unwrap().into_inner();
        let auction = response.auction.unwrap();
        assert_eq!(auction.title, "Test Auction");
        assert_eq!(auction.category, "Electronics");
        assert_eq!(auction.currency, "EUR"); // Verificar la moneda seleccionada
        assert_eq!(auction.status, "pending"); // Verificar que se crea como "pending"
    }

    #[tokio::test]
    async fn test_create_auction_default_currency() {
        let service = setup_service().await;
        let req = CreateAuctionRequest {
            user_id: uuid::Uuid::new_v4().to_string(),
            item_id: uuid::Uuid::new_v4().to_string(),
            title: "Test Auction".to_string(),
            description: "desc".to_string(),
            category: "Electronics".to_string(),
            start_time: Some(prost_types::Timestamp { seconds: 1_600_000_000, nanos: 0 }),
            end_time: Some(prost_types::Timestamp { seconds: 1_600_000_100, nanos: 0 }),
            base_price: "100.00".to_string(),
            min_bid_increment: "10.00".to_string(),
            highest_bid: "".to_string(),
            currency: "".to_string(), // Sin especificar moneda
        };
        let response = service.create_auction(Request::new(req)).await.unwrap().into_inner();
        let auction = response.auction.unwrap();
        assert_eq!(auction.currency, "USD"); // Debe usar USD por defecto
    }

    #[tokio::test]
    async fn test_auction_status_validation() {
        // Probar que el enum funciona correctamente
        assert_eq!(AuctionStatus::Pending.as_str(), "pending");
        assert_eq!(AuctionStatus::Active.as_str(), "active");
        assert_eq!(AuctionStatus::Completed.as_str(), "completed");
        assert_eq!(AuctionStatus::Cancelled.as_str(), "cancelled");
        
        // Probar conversión desde string
        assert!(AuctionStatus::from_str("pending").is_ok());
        assert!(AuctionStatus::from_str("active").is_ok());
        assert!(AuctionStatus::from_str("completed").is_ok());
        assert!(AuctionStatus::from_str("cancelled").is_ok());
        assert!(AuctionStatus::from_str("invalid").is_err());
    }

    #[tokio::test]
    async fn test_currency_validation() {
        // Probar que el enum funciona correctamente
        assert_eq!(AuctionCurrency::USD.as_str(), "USD");
        assert_eq!(AuctionCurrency::EUR.as_str(), "EUR");
        assert_eq!(AuctionCurrency::CLP.as_str(), "CLP");
        
        // Probar conversión desde string
        assert!(AuctionCurrency::from_str("USD").is_ok());
        assert!(AuctionCurrency::from_str("eur").is_ok()); // Case insensitive
        assert!(AuctionCurrency::from_str("clp").is_ok());
        assert!(AuctionCurrency::from_str("INVALID").is_err());
    }

    #[tokio::test]
    async fn test_create_bid_ok() {
        let service = setup_service().await;
        
        // Primero crear una subasta
        let auction_req = CreateAuctionRequest {
            user_id: uuid::Uuid::new_v4().to_string(),
            item_id: uuid::Uuid::new_v4().to_string(),
            title: "Test Auction".to_string(),
            description: "desc".to_string(),
            category: "Electronics".to_string(),
            start_time: Some(prost_types::Timestamp { seconds: chrono::Utc::now().timestamp() + 100, nanos: 0 }),
            end_time: Some(prost_types::Timestamp { seconds: chrono::Utc::now().timestamp() + 3600, nanos: 0 }),
            base_price: "100.00".to_string(),
            min_bid_increment: "10.00".to_string(),
            highest_bid: "".to_string(),
            currency: "USD".to_string(),
        };
        
        let auction_response = service.create_auction(Request::new(auction_req)).await.unwrap().into_inner();
        let auction_id = auction_response.auction.unwrap().id;

        // Crear una puja
        let bid_req = CreateBidRequest {
            auction_id: auction_id.clone(),
            user_id: uuid::Uuid::new_v4().to_string(),
            amount: "120.00".to_string(),
        };

        let bid_response = service.create_bid(Request::new(bid_req)).await.unwrap().into_inner();
        let bid = bid_response.bid.unwrap();
        
        assert_eq!(bid.auction_id, auction_id);
        assert_eq!(bid.amount, "120.00");
        assert_eq!(bid.status, "active");
    }

    #[tokio::test]
    async fn test_get_auction_with_bids() {
        let service = setup_service().await;
        
        // Crear una subasta
        let auction_req = CreateAuctionRequest {
            user_id: uuid::Uuid::new_v4().to_string(),
            item_id: uuid::Uuid::new_v4().to_string(),
            title: "Test Auction".to_string(),
            description: "desc".to_string(),
            category: "Electronics".to_string(),
            start_time: Some(prost_types::Timestamp { seconds: chrono::Utc::now().timestamp() + 100, nanos: 0 }),
            end_time: Some(prost_types::Timestamp { seconds: chrono::Utc::now().timestamp() + 3600, nanos: 0 }),
            base_price: "100.00".to_string(),
            min_bid_increment: "10.00".to_string(),
            highest_bid: "".to_string(),
            currency: "USD".to_string(),
        };
        
        let auction_response = service.create_auction(Request::new(auction_req)).await.unwrap().into_inner();
        let auction_id = auction_response.auction.unwrap().id;

        // Crear algunas pujas
        for i in 1..=3 {
            let bid_req = CreateBidRequest {
                auction_id: auction_id.clone(),
                user_id: uuid::Uuid::new_v4().to_string(),
                amount: format!("{}.00", 100 + i * 10),
            };
            service.create_bid(Request::new(bid_req)).await.unwrap();
        }

        // Obtener la subasta con pujas
        let get_req = GetAuctionRequest {
            id: auction_id.clone(),
        };
        let response = service.get_auction(Request::new(get_req)).await.unwrap().into_inner();
        let auction = response.auction.unwrap();
        
        assert_eq!(auction.id, auction_id);
        assert_eq!(auction.bids.len(), 3);
        assert_eq!(auction.bids[0].amount, "130.00"); // La puja más reciente primero
    }

    #[tokio::test]
    async fn test_model_structure() {
        use crate::models::auction::{ActiveModel, Entity};
        use sea_orm::Set;
        
        let db = setup_test_db().await;
        
        let auction = ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            user_id: Set("test_user".to_string()),
            item_id: Set("test_item".to_string()),
            title: Set("Test Title".to_string()),
            description: Set(Some("Test Description".to_string())),
            category: Set("Electronics".to_string()),
            start_time: Set(chrono::Utc::now().naive_utc()),
            end_time: Set(chrono::Utc::now().naive_utc() + chrono::Duration::hours(1)),
            base_price: Set(rust_decimal::Decimal::from(100)),
            min_bid_increment: Set(rust_decimal::Decimal::from(10)),
            highest_bid: Set(Some(rust_decimal::Decimal::ZERO)),
            status: Set("pending".to_string()),
            currency: Set("USD".to_string()),
        };
        
        let result = auction.insert(&db).await;
        assert!(result.is_ok(), "Failed to insert auction: {:?}", result.err());
    }
}

// Función helper para validar que un string representa un número válido
fn validate_numeric_string(value: &str, field_name: &str) -> Result<rust_decimal::Decimal, Status> {
    if value.is_empty() {
        return Err(Status::invalid_argument(format!("{} no puede estar vacío", field_name)));
    }
    
    value.parse::<rust_decimal::Decimal>()
        .map_err(|e| {
            log::error!("Error al parsear {} '{}': {}", field_name, value, e);
            Status::invalid_argument(format!("{} debe ser un número válido", field_name))
        })
}



