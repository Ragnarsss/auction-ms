# Guía de Migración y Despliegue - Auction Microservice

## Tabla de Contenidos

- [Prerrequisitos](#prerrequisitos)
- [Preparación del Entorno](#preparación-del-entorno)
- [Migración de Base de Datos](#migración-de-base-de-datos)
- [Configuración de la Aplicación](#configuración-de-la-aplicación)
- [Despliegue en Desarrollo](#despliegue-en-desarrollo)
- [Despliegue en Producción](#despliegue-en-producción)
- [Verificación y Testing](#verificación-y-testing)
- [Monitoreo y Logs](#monitoreo-y-logs)
- [Troubleshooting](#troubleshooting)

## Prerrequisitos

### Software Requerido

- **Go** >= 1.19
- **PostgreSQL** >= 13
- **Docker** >= 20.10
- **Docker Compose** >= 2.0
- **Protocol Buffers Compiler** (protoc)
- **Git**

### Go Dependencies

```bash
go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
```

## Preparación del Entorno

### 1. Clonar y Configurar el Proyecto

```bash
# Clonar el repositorio
git clone <repository-url>
cd auction_ms

# Instalar dependencias
go mod tidy

# Generar código Protocol Buffers
protoc --go_out=. --go-grpc_out=. proto/auction.proto
```

### 2. Variables de Entorno

Crear archivo `.env` en la raíz del proyecto:

```env
# Base de datos
DB_HOST=localhost
DB_PORT=5432
DB_NAME=auction_db
DB_USER=auction_user
DB_PASSWORD=auction_password
DB_SSL_MODE=disable

# Servidor
SERVER_PORT=8080
GRPC_PORT=50051

# Configuración de aplicación
LOG_LEVEL=info
ENVIRONMENT=development

# JWT (si se usa autenticación)
JWT_SECRET=your-jwt-secret-key

# Redis (si se usa para caché)
REDIS_HOST=localhost
REDIS_PORT=6379
REDIS_PASSWORD=
```

## Migración de Base de Datos

### 1. Configuración de PostgreSQL

#### Opción A: Instalación Local

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install postgresql postgresql-contrib

# CentOS/RHEL
sudo yum install postgresql postgresql-server postgresql-contrib

# Iniciar servicio
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

#### Opción B: Docker

```bash
docker run --name auction-postgres \
  -e POSTGRES_DB=auction_db \
  -e POSTGRES_USER=auction_user \
  -e POSTGRES_PASSWORD=auction_password \
  -p 5432:5432 \
  -d postgres:13
```

### 2. Crear Base de Datos y Usuario

```sql
-- Conectar como superusuario
sudo -u postgres psql

-- Crear usuario y base de datos
CREATE USER auction_user WITH PASSWORD 'auction_password';
CREATE DATABASE auction_db OWNER auction_user;
GRANT ALL PRIVILEGES ON DATABASE auction_db TO auction_user;

-- Conectar a la nueva base de datos
\c auction_db

-- Crear extensiones necesarias
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
```

### 3. Schema de Base de Datos

```sql
-- Tabla de subastas
CREATE TABLE auctions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id VARCHAR(255) NOT NULL,
    item_id VARCHAR(255) NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time TIMESTAMP WITH TIME ZONE NOT NULL,
    base_price DECIMAL(15,2) NOT NULL,
    min_bid_increment DECIMAL(15,2) NOT NULL,
    highest_bid DECIMAL(15,2) DEFAULT 0,
    status VARCHAR(50) DEFAULT 'active',
    currency VARCHAR(3) DEFAULT 'USD',
    category VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Tabla de pujas
CREATE TABLE bids (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    auction_id UUID NOT NULL REFERENCES auctions(id) ON DELETE CASCADE,
    user_id VARCHAR(255) NOT NULL,
    amount DECIMAL(15,2) NOT NULL,
    status VARCHAR(50) DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Índices para optimizar consultas
CREATE INDEX idx_auctions_user_id ON auctions(user_id);
CREATE INDEX idx_auctions_status ON auctions(status);
CREATE INDEX idx_auctions_end_time ON auctions(end_time);
CREATE INDEX idx_bids_auction_id ON bids(auction_id);
CREATE INDEX idx_bids_user_id ON bids(user_id);
CREATE INDEX idx_bids_amount ON bids(amount DESC);

-- Trigger para actualizar updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_auctions_updated_at BEFORE UPDATE
    ON auctions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
```

## Configuración de la Aplicación

### 1. Estructura de Directorios

```
auction_ms/
├── cmd/
│   └── server/
│       └── main.go
├── internal/
│   ├── config/
│   ├── handler/
│   ├── repository/
│   ├── service/
│   └── models/
├── proto/
│   └── auction.proto
├── pkg/
├── migrations/
├── docker/
├── .env
├── docker-compose.yml
├── Dockerfile
└── Makefile
```

### 2. Archivo Docker Compose

Crear `docker-compose.yml`:

```yaml
version: "3.8"

services:
  postgres:
    image: postgres:13
    environment:
      POSTGRES_DB: auction_db
      POSTGRES_USER: auction_user
      POSTGRES_PASSWORD: auction_password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./migrations:/docker-entrypoint-initdb.d
    networks:
      - auction-network

  redis:
    image: redis:6-alpine
    ports:
      - "6379:6379"
    networks:
      - auction-network

  auction-service:
    build: .
    ports:
      - "8080:8080"
      - "50051:50051"
    environment:
      - DB_HOST=postgres
      - REDIS_HOST=redis
    depends_on:
      - postgres
      - redis
    networks:
      - auction-network
    volumes:
      - ./.env:/app/.env

volumes:
  postgres_data:

networks:
  auction-network:
    driver: bridge
```

### 3. Dockerfile

```dockerfile
FROM golang:1.19-alpine AS builder

WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download

COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -o auction-service cmd/server/main.go

FROM alpine:latest
RUN apk --no-cache add ca-certificates
WORKDIR /root/

COPY --from=builder /app/auction-service .
COPY --from=builder /app/.env .

EXPOSE 8080 50051

CMD ["./auction-service"]
```

## Despliegue en Desarrollo

### 1. Usando Docker Compose

```bash
# Construir y levantar servicios
docker-compose up --build -d

# Verificar que los servicios estén corriendo
docker-compose ps

# Ver logs
docker-compose logs auction-service
```

### 2. Desarrollo Local

```bash
# Levantar solo la base de datos
docker-compose up postgres redis -d

# Ejecutar la aplicación localmente
go run cmd/server/main.go
```

### 3. Makefile para Comandos Comunes

Crear `Makefile`:

```makefile
.PHONY: build run test proto clean docker-build docker-run

build:
	go build -o bin/auction-service cmd/server/main.go

run:
	go run cmd/server/main.go

test:
	go test ./... -v

proto:
	protoc --go_out=. --go-grpc_out=. proto/auction.proto

clean:
	rm -rf bin/

docker-build:
	docker-compose build

docker-run:
	docker-compose up -d

docker-stop:
	docker-compose down

docker-logs:
	docker-compose logs -f auction-service

db-migrate:
	docker-compose exec postgres psql -U auction_user -d auction_db -f /docker-entrypoint-initdb.d/schema.sql
```

## Despliegue en Producción

### 1. Configuración de Servidor

```bash
# Actualizar sistema
sudo apt update && sudo apt upgrade -y

# Instalar Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Instalar Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/download/v2.10.2/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

### 2. Variables de Entorno de Producción

Crear `.env.production`:

```env
DB_HOST=production-db-host
DB_PORT=5432
DB_NAME=auction_db_prod
DB_USER=auction_user_prod
DB_PASSWORD=strong-production-password
DB_SSL_MODE=require

SERVER_PORT=8080
GRPC_PORT=50051

LOG_LEVEL=warn
ENVIRONMENT=production

JWT_SECRET=very-strong-jwt-secret-for-production

REDIS_HOST=production-redis-host
REDIS_PORT=6379
REDIS_PASSWORD=redis-production-password
```

### 3. Docker Compose para Producción

Crear `docker-compose.prod.yml`:

```yaml
version: "3.8"

services:
  auction-service:
    build: .
    ports:
      - "8080:8080"
      - "50051:50051"
    environment:
      - DB_HOST=${DB_HOST}
      - DB_PORT=${DB_PORT}
      - DB_NAME=${DB_NAME}
      - DB_USER=${DB_USER}
      - DB_PASSWORD=${DB_PASSWORD}
      - REDIS_HOST=${REDIS_HOST}
      - REDIS_PORT=${REDIS_PORT}
    restart: unless-stopped
    networks:
      - auction-network
    volumes:
      - ./.env.production:/app/.env

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    depends_on:
      - auction-service
    restart: unless-stopped
    networks:
      - auction-network

networks:
  auction-network:
    driver: bridge
```

### 4. Configuración de Nginx

Crear `nginx.conf`:

```nginx
events {
    worker_connections 1024;
}

http {
    upstream auction_backend {
        server auction-service:8080;
    }

    server {
        listen 80;
        server_name your-domain.com;
        return 301 https://$server_name$request_uri;
    }

    server {
        listen 443 ssl http2;
        server_name your-domain.com;

        ssl_certificate /etc/nginx/ssl/cert.pem;
        ssl_certificate_key /etc/nginx/ssl/key.pem;

        location / {
            proxy_pass http://auction_backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }
    }
}
```

## Verificación y Testing

### 1. Health Check

```bash
# Verificar que el servicio responda
curl http://localhost:8080/health

# Verificar conexión gRPC
grpcurl -plaintext localhost:50051 list
```

### 2. Tests de Integración

```bash
# Ejecutar tests
make test

# Test de carga básico
ab -n 1000 -c 10 http://localhost:8080/health
```

### 3. Verificación de Base de Datos

```bash
# Conectar a la base de datos
docker-compose exec postgres psql -U auction_user -d auction_db

# Verificar tablas
\dt

# Verificar datos de ejemplo
SELECT COUNT(*) FROM auctions;
SELECT COUNT(*) FROM bids;
```

## Monitoreo y Logs

### 1. Configuración de Logs

```bash
# Ver logs en tiempo real
docker-compose logs -f auction-service

# Logs con filtro por nivel
docker-compose logs auction-service | grep ERROR
```

### 2. Métricas Básicas

```bash
# Uso de recursos
docker stats

# Espacio en disco
df -h

# Conexiones a base de datos
docker-compose exec postgres psql -U auction_user -d auction_db -c "SELECT count(*) FROM pg_stat_activity;"
```

## Troubleshooting

### Problemas Comunes

#### 1. Error de Conexión a Base de Datos

```bash
# Verificar que PostgreSQL esté corriendo
docker-compose ps postgres

# Verificar logs de PostgreSQL
docker-compose logs postgres

# Probar conexión manual
docker-compose exec postgres psql -U auction_user -d auction_db
```

#### 2. Puerto en Uso

```bash
# Verificar qué proceso usa el puerto
lsof -i :8080
lsof -i :50051

# Cambiar puerto en .env si es necesario
SERVER_PORT=8081
GRPC_PORT=50052
```

#### 3. Problemas de Permisos

```bash
# Verificar permisos de archivos
ls -la .env

# Cambiar propietario si es necesario
sudo chown $USER:$USER .env
```

#### 4. Memoria Insuficiente

```bash
# Verificar uso de memoria
free -h

# Limpiar containers no utilizados
docker system prune -a
```

### Comandos de Emergencia

```bash
# Reinicio completo del sistema
docker-compose down
docker-compose up --build -d

# Backup de base de datos
docker-compose exec postgres pg_dump -U auction_user auction_db > backup.sql

# Restaurar backup
docker-compose exec -T postgres psql -U auction_user -d auction_db < backup.sql

# Limpiar logs
docker-compose logs --tail=0 auction-service

# Recrear volúmenes
docker-compose down -v
docker-compose up -d
```

## Mantenimiento

### 1. Actualización de la Aplicación

```bash
# Pull del código actualizado
git pull origin main

# Rebuilding y deploy
docker-compose build auction-service
docker-compose up -d auction-service
```

### 2. Backup Automatizado

Crear script `backup.sh`:

```bash
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
docker-compose exec postgres pg_dump -U auction_user auction_db > backups/auction_db_$DATE.sql
```

### 3. Monitoreo de Salud

Crear script `health_check.sh`:

```bash
#!/bin/bash
curl -f http://localhost:8080/health || exit 1
```

---

## Contacto y Soporte

Para problemas adicionales, consultar:

- Logs de aplicación: `docker-compose logs auction-service`
- Documentación de la API: `http://localhost:8080/docs`
- Repositorio del proyecto: [GitHub URL]
