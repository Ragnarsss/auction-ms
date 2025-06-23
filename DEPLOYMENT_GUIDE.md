# Guía de Migración y Despliegue - Auction Microservice (Rust) - Windows

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

- **Rust** >= 1.70 (con Cargo)
- **PostgreSQL** >= 13 para Windows
- **Docker Desktop** para Windows
- **Protocol Buffers Compiler** (protoc) para Windows
- **Git** para Windows
- **Windows Terminal** o **PowerShell 7+** (recomendado)

### Instalación de Rust en Windows

```powershell
# Opción 1: Usando rustup (recomendado)
# Descargar e instalar desde https://rustup.rs/
# O ejecutar en PowerShell:
Invoke-WebRequest -Uri "https://win.rustup.rs/" -OutFile "rustup-init.exe"
.\rustup-init.exe

# Verificar instalación
rustc --version
cargo --version
```

### Dependencias del Sistema Windows

```powershell
# Instalar chocolatey (gestor de paquetes para Windows)
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# Instalar dependencias usando chocolatey
choco install git
choco install docker-desktop
choco install protoc

# Alternativamente, instalar manualmente:
# - Git: https://git-scm.com/download/win
# - Docker Desktop: https://www.docker.com/products/docker-desktop
# - Protocol Buffers: https://github.com/protocolbuffers/protobuf/releases
```

### Configuración de Visual Studio Build Tools

```powershell
# Instalar Visual Studio Build Tools (necesario para compilar Rust en Windows)
choco install visualstudio2022buildtools

# O descargar desde:
# https://visualstudio.microsoft.com/visual-cpp-build-tools/

# Asegurarse de instalar:
# - MSVC v143 - VS 2022 C++ x64/x86 build tools
# - Windows 10/11 SDK
```

## Preparación del Entorno

### 1. Clonar y Configurar el Proyecto

```powershell
# Clonar el repositorio
git clone <repository-url>
cd auction_ms

# Instalar dependencias
cargo build

# Generar código Protocol Buffers (se hace automáticamente con build.rs)
cargo build
```

### 2. Variables de Entorno

Crear archivo `.env` en la raíz del proyecto usando PowerShell:

```powershell
# Crear archivo .env
@"
# Base de datos
DATABASE_URL=postgres://auction_user:auction_password@localhost:5432/auction_db

# Servidor gRPC
GRPC_ADDRESS=0.0.0.0:50052

# Configuración de aplicación
RUST_LOG=info
ENVIRONMENT=development
"@ | Out-File -FilePath ".env" -Encoding UTF8
```

### 3. Configuración de Windows Defender y Firewall

```powershell
# Crear excepción en Windows Defender para la carpeta del proyecto (ejecutar como Administrador)
Add-MpPreference -ExclusionPath "C:\Users\zunig\OneDrive\Documentos\UCN\1-2025 ULTIMO SEMESTRE CTMMM\Arquitectura de sistemas\ev2\auction_ms"

# Abrir puertos en el firewall de Windows
New-NetFirewallRule -DisplayName "Auction Service gRPC" -Direction Inbound -Protocol TCP -LocalPort 50052
New-NetFirewallRule -DisplayName "PostgreSQL" -Direction Inbound -Protocol TCP -LocalPort 5432
New-NetFirewallRule -DisplayName "pgAdmin" -Direction Inbound -Protocol TCP -LocalPort 5050
```

## Migración de Base de Datos

### 1. Configuración de PostgreSQL en Windows

#### Opción A: Usando Docker Desktop (Recomendado)

```powershell
# Iniciar Docker Desktop
# Verificar que Docker esté corriendo
docker --version

# Levantar solo la base de datos y pgAdmin
docker-compose up auction_database pgadmin -d
```

#### Opción B: Instalación Nativa en Windows

```powershell
# Usando chocolatey
choco install postgresql

# O descargar desde: https://www.postgresql.org/download/windows/

# Iniciar servicio PostgreSQL
Start-Service postgresql-x64-13

# Configurar para inicio automático
Set-Service -Name postgresql-x64-13 -StartupType Automatic
```

### 2. Crear Base de Datos y Usuario (Windows)

```powershell
# Conectar usando psql de Windows
# Ajustar la ruta según tu instalación de PostgreSQL
$env:PATH += ";C:\Program Files\PostgreSQL\13\bin"

# Conectar como superusuario
psql -U postgres -h localhost

# En la consola de PostgreSQL:
```

```sql
-- Crear usuario y base de datos
CREATE USER auction_user WITH PASSWORD 'auction_password';
CREATE DATABASE auction_db OWNER auction_user;
GRANT ALL PRIVILEGES ON DATABASE auction_db TO auction_user;

-- Conectar a la nueva base de datos
\c auction_db

-- Crear extensiones necesarias
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
```

### 3. Ejecutar Migraciones en Windows

```powershell
# Instalar sea-orm-cli usando cargo
cargo install sea-orm-cli

# Verificar que esté en el PATH
sea-orm-cli --version

# Ejecutar migraciones
sea-orm-cli migrate up -d postgres://auction_user:auction_password@localhost:5432/auction_db
```

### 4. Verificar Tablas Creadas

```powershell
# Conectar a la base de datos
psql -U auction_user -d auction_db -h localhost

# En la consola de PostgreSQL:
# \dt
# \d auction
```

## Configuración de la Aplicación

### 1. Estructura de Directorios (Windows)

```
auction_ms\
├── src\
│   ├── main.rs
│   ├── grpc_server.rs
│   ├── config.rs
│   ├── models.rs
│   └── db.rs
├── migration\
│   └── src\
│       └── m20250619_044136_create_auction_table.rs
├── proto\
│   └── auction.proto
├── Cargo.toml
├── build.rs
├── Dockerfile
├── docker-compose.yml
└── .env
```

### 2. Configuración Actual de Docker Compose

El archivo `docker-compose.yml` ya está configurado con:

- PostgreSQL en puerto 5432
- pgAdmin en puerto 5050
- Aplicación Rust en puerto 50052

### 3. Variables de Entorno por Ambiente

```bash
# Desarrollo (.env)
DATABASE_URL=postgres://auction_user:auction_password@localhost:5432/auction_db
GRPC_ADDRESS=0.0.0.0:50052
RUST_LOG=info

# Producción (.env.production)
DATABASE_URL=postgres://auction_user:auction_password@auction_database:5432/auction_db
GRPC_ADDRESS=0.0.0.0:50052
RUST_LOG=warn
```

## Despliegue en Desarrollo

### 1. Usando Docker Compose (Recomendado)

```bash
# Construir y levantar todos los servicios
docker-compose up --build -d

# Verificar que los servicios estén corriendo
docker-compose ps

# Ver logs de la aplicación
docker-compose logs -f auction_ms
```

### 2. Desarrollo Local

```bash
# Levantar solo la base de datos
docker-compose up auction_database -d

# Ejecutar migraciones
sea-orm-cli migrate up

# Ejecutar la aplicación localmente
cargo run
```

### 3. Comandos Útiles

```bash
# Compilar en modo debug
cargo build

# Compilar en modo release
cargo build --release

# Ejecutar tests
cargo test

# Limpiar artifacts de compilación
cargo clean

# Verificar formato del código
cargo fmt --check

# Linting
cargo clippy
```

## Despliegue en Producción

### 1. Configuración de Servidor

```bash
# Actualizar sistema
sudo apt update && sudo apt upgrade -y

# Instalar Docker y Docker Compose
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Instalar Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/download/v2.10.2/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

### 2. Docker Compose para Producción

Crear `docker-compose.prod.yml`:

```yaml
services:
  auction_database:
    image: postgres:latest
    environment:
      POSTGRES_USER: auction_user
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_DB: auction_db
    volumes:
      - auction_data:/var/lib/postgresql/data
    restart: unless-stopped
    networks:
      - auction-network

  auction_ms:
    build: .
    depends_on:
      - auction_database
    environment:
      DATABASE_URL: postgres://auction_user:${DB_PASSWORD}@auction_database:5432/auction_db
      GRPC_ADDRESS: "0.0.0.0:50052"
      RUST_LOG: "warn"
    ports:
      - "50052:50052"
    restart: unless-stopped
    networks:
      - auction-network

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    depends_on:
      - auction_ms
    restart: unless-stopped
    networks:
      - auction-network

volumes:
  auction_data:

networks:
  auction-network:
    driver: bridge
```

### 3. Configuración de Nginx para gRPC

Crear `nginx.conf`:

```nginx
events {
    worker_connections 1024;
}

http {
    upstream grpc_backend {
        server auction_ms:50052;
    }

    server {
        listen 80 http2;
        server_name your-domain.com;

        location / {
            grpc_pass grpc://grpc_backend;
            grpc_set_header Host $host;
            grpc_set_header X-Real-IP $remote_addr;
            grpc_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            grpc_set_header X-Forwarded-Proto $scheme;
        }
    }
}
```

## Verificación y Testing

### 1. Health Check para gRPC

```bash
# Instalar grpcurl para testing
go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest

# Listar servicios disponibles
grpcurl -plaintext localhost:50052 list

# Listar métodos del servicio
grpcurl -plaintext localhost:50052 list auction.AuctionService

# Test de conexión
grpcurl -plaintext localhost:50052 auction.AuctionService/ListAuctions
```

### 2. Tests de la Aplicación

```bash
# Ejecutar todos los tests
cargo test

# Ejecutar tests con output verbose
cargo test -- --nocapture

# Ejecutar tests específicos
cargo test test_create_auction
```

### 3. Verificación de Base de Datos

```bash
# Conectar a la base de datos via Docker
docker-compose exec auction_database psql -U auction_user -d auction_db

# O usando pgAdmin en http://localhost:5050
# Usuario: admin@admin.com
# Password: auction_password

# Verificar tablas
\dt

# Verificar estructura de auction
\d auction
```

## Monitoreo y Logs

### 1. Configuración de Logs

```bash
# Ver logs en tiempo real
docker-compose logs -f auction_ms

# Ver logs de la base de datos
docker-compose logs -f auction_database

# Logs con nivel específico
RUST_LOG=debug cargo run
```

### 2. Métricas de Sistema

```bash
# Uso de recursos de contenedores
docker stats

# Espacio en disco
df -h

# Memoria del sistema
free -h

# Conexiones activas a PostgreSQL
docker-compose exec auction_database psql -U auction_user -d auction_db -c "SELECT count(*) FROM pg_stat_activity WHERE state = 'active';"
```

## Troubleshooting

### Problemas Comunes

#### 1. Error de Compilación de Protobuf

```bash
# Verificar que protoc esté instalado
protoc --version

# Reinstalar protobuf compiler
sudo apt install protobuf-compiler

# Limpiar y recompilar
cargo clean
cargo build
```

#### 2. Error de Conexión a Base de Datos

```bash
# Verificar que PostgreSQL esté corriendo
docker-compose ps auction_database

# Verificar logs de PostgreSQL
docker-compose logs auction_database

# Probar conexión manual
docker-compose exec auction_database psql -U auction_user -d auction_db
```

#### 3. Puerto en Uso

```bash
# Verificar qué proceso usa el puerto
lsof -i :50052
netstat -tulpn | grep :50052

# Cambiar puerto en docker-compose.yml si es necesario
```

#### 4. Problemas de Migración

```bash
# Verificar estado de migraciones
sea-orm-cli migrate status

# Revertir última migración
sea-orm-cli migrate down

# Aplicar migraciones específicas
sea-orm-cli migrate up -n 1
```

### Comandos de Emergencia

```bash
# Reinicio completo del sistema
docker-compose down
docker-compose up --build -d

# Backup de base de datos
docker-compose exec auction_database pg_dump -U auction_user auction_db > backup_$(date +%Y%m%d_%H%M%S).sql

# Restaurar backup
docker-compose exec -T auction_database psql -U auction_user -d auction_db < backup.sql

# Limpiar volúmenes de Docker
docker-compose down -v
docker volume prune

# Reconstruir imagen sin cache
docker-compose build --no-cache auction_ms
```

## Mantenimiento

### 1. Actualización de la Aplicación

```bash
# Pull del código actualizado
git pull origin main

# Recompilar y desplegar
docker-compose build auction_ms
docker-compose up -d auction_ms
```

### 2. Backup Automatizado

Crear script `backup.sh`:

```bash
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
docker-compose exec auction_database pg_dump -U auction_user auction_db > backups/auction_db_$DATE.sql
echo "Backup creado: backups/auction_db_$DATE.sql"
```

### 3. Actualización de Dependencias

```bash
# Actualizar Cargo.lock
cargo update

# Verificar dependencias desactualizadas
cargo audit

# Verificar vulnerabilidades de seguridad
cargo audit --db ~/.cargo/advisory-db
```

### 4. Monitoreo de Performance

```bash
# Profiling de la aplicación
cargo install flamegraph
cargo flamegraph --bin auction_ms

# Benchmarks
cargo bench
```

---

## Contacto y Soporte

Para problemas adicionales, consultar:

- Logs de aplicación: `docker-compose logs auction_ms`
- pgAdmin: `http://localhost:5050`
- Documentación de Sea-ORM: https://www.sea-ql.org/SeaORM/
- Documentación de Tonic: https://github.com/hyperium/tonic
  docker-compose logs auction-service | grep ERROR

````

### 2. Métricas Básicas

```bash
# Uso de recursos
docker stats

# Espacio en disco
df -h

# Conexiones a base de datos
docker-compose exec postgres psql -U auction_user -d auction_db -c "SELECT count(*) FROM pg_stat_activity;"
````

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
