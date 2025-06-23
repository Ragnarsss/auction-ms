# Etapa 1: Build
FROM rustlang/rust:nightly as builder
WORKDIR /usr/src/app

# Instala dependencias del sistema necesarias para compilar crates nativos
RUN apt-get update && apt-get install -y libssl-dev ca-certificates protobuf-compiler && rm -rf /var/lib/apt/lists/*

# Copia los archivos de dependencias y compila dependencias primero (cache eficiente)
COPY Cargo.toml Cargo.lock ./
COPY migration ./migration
COPY build.rs .
COPY proto ./proto
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true

# Copia el resto del código fuente y el archivo .env
COPY src ./src
COPY .env .env

# Compila el binario en release
RUN cargo build --release

# Etapa 2: Imagen final mínima
FROM debian:bookworm-slim as runtime
WORKDIR /app

# Instala solo las librerías necesarias para ejecutar el binario
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copia el binario y el .env desde la etapa de build
COPY --from=builder /usr/src/app/target/release/auction_ms .
COPY --from=builder /usr/src/app/.env .env

# Expone el puerto gRPC
EXPOSE 50051

# Permite que las variables de entorno se sobreescriban al correr el contenedor
ENV RUST_LOG=info

RUN ls -l /app

# Ejecuta el binario (dotenvy cargará .env automáticamente si existe)
CMD ["./auction_ms"]
