# Manejo de Fechas en el Sistema de Subastas

## Estrategia General

El sistema utiliza una estrategia consistente para el manejo de fechas:

1. **Backend (Rust gRPC)**: Almacena fechas como UTC en la base de datos
2. **Protocolo gRPC**: Usa `google.protobuf.Timestamp` para comunicación
3. **Frontend (React + MUI)**: Convierte a zona horaria local para mostrar al usuario

## Formatos de Fecha por Contexto

### Base de Datos (PostgreSQL/SQLite)

```sql
-- Almacenado como TIMESTAMP (UTC)
CREATE TABLE auction (
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP NOT NULL,
    -- ...
);
```

### Protocolo gRPC

```protobuf
// auction.proto
message Auction {
    google.protobuf.Timestamp start_time = 6;
    google.protobuf.Timestamp end_time = 7;
    // ...
}
```

### Backend Rust

```rust
// Conversión de protobuf a NaiveDateTime
fn proto_timestamp_to_naive(ts: &Option<Timestamp>) -> Result<chrono::NaiveDateTime, Status> {
    let t = ts.as_ref().ok_or(Status::invalid_argument("timestamp faltante"))?;
    Ok(chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32)
        .ok_or(Status::invalid_argument("timestamp inválido"))?
        .naive_utc())
}

// Conversión de NaiveDateTime a protobuf
fn naive_to_proto_timestamp(dt: &chrono::NaiveDateTime) -> Option<Timestamp> {
    let dt_utc = dt.and_utc();
    Some(Timestamp {
        seconds: dt_utc.timestamp(),
        nanos: dt_utc.timestamp_subsec_nanos() as i32,
    })
}
```

### Frontend React + MUI

```javascript
// Conversión de gRPC timestamp a Date para MUI
export const grpcTimestampToDate = (timestamp) => {
  if (!timestamp || !timestamp.seconds) return null;
  return new Date(timestamp.seconds * 1000 + timestamp.nanos / 1000000);
};

// Conversión de Date de MUI a gRPC timestamp
export const dateToGrpcTimestamp = (date) => {
  if (!date) return null;
  const timestamp = Math.floor(date.getTime() / 1000);
  const nanos = (date.getTime() % 1000) * 1000000;
  return {
    seconds: timestamp,
    nanos: nanos,
  };
};
```

## Validaciones de Fecha

### Backend (Rust)

```rust
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
```

### Frontend (JavaScript)

```javascript
export const validateDateRange = (startDate, endDate) => {
  if (!startDate || !endDate) return false;
  return startDate < endDate && startDate > new Date();
};
```

## Configuración de MUI DateTimePicker

```jsx
import { DateTimePicker } from "@mui/x-date-pickers/DateTimePicker";
import { LocalizationProvider } from "@mui/x-date-pickers/LocalizationProvider";
import { AdapterDateFns } from "@mui/x-date-pickers/AdapterDateFns";
import { es } from "date-fns/locale";

// Uso recomendado
<LocalizationProvider dateAdapter={AdapterDateFns} adapterLocale={es}>
  <DateTimePicker
    label="Fecha de Inicio"
    value={startTime}
    onChange={handleStartTimeChange}
    renderInput={(params) => <TextField {...params} />}
    minDateTime={new Date()} // No permitir fechas pasadas
  />
</LocalizationProvider>;
```

## Dependencias del Frontend

Para el manejo óptimo de fechas en React + MUI:

```json
{
  "dependencies": {
    "@mui/material": "^5.x.x",
    "@mui/x-date-pickers": "^6.x.x",
    "date-fns": "^2.x.x",
    "@mui/lab": "^5.x.x"
  }
}
```

## Instalación de dependencias MUI

```bash
npm install @mui/x-date-pickers @mui/material @emotion/react @emotion/styled date-fns
```

## Consideraciones de Zona Horaria

1. **Servidor**: Siempre almacena en UTC
2. **Cliente**: Muestra en zona horaria local del usuario
3. **Comunicación**: Usa UTC en el protocolo gRPC
4. **Validación**: Se hace tanto en cliente como servidor

## Ejemplos de Uso

### Crear Subasta

```javascript
const createAuction = async (formData) => {
  const request = {
    title: formData.title,
    description: formData.description,
    start_time: dateToGrpcTimestamp(formData.startTime),
    end_time: dateToGrpcTimestamp(formData.endTime),
    base_price: formData.basePrice,
    min_bid_increment: formData.minBidIncrement,
    status: "active",
    currency: "USD",
  };

  const response = await auctionService.createAuction(request);
  return response;
};
```

### Mostrar Subasta

```javascript
const displayAuction = (auction) => {
  const startDate = grpcTimestampToDate(auction.start_time);
  const endDate = grpcTimestampToDate(auction.end_time);

  return {
    ...auction,
    formattedStartTime: formatDateForDisplay(startDate),
    formattedEndTime: formatDateForDisplay(endDate),
  };
};
```

## Buenas Prácticas

1. **Validación Dual**: Validar fechas tanto en frontend como backend
2. **UTC Interno**: Siempre almacenar en UTC internamente
3. **Conversión Tardía**: Convertir a zona horaria local solo para mostrar
4. **Mensajes Claros**: Proporcionar mensajes de error claros sobre fechas
5. **Restricciones**: No permitir fechas en el pasado para subastas nuevas
