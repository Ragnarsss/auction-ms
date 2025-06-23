// Ejemplo de integración con React + MUI DateTimePicker
// Archivo: frontend/src/utils/dateHelpers.js

import { format, parseISO, formatISO } from "date-fns";
import { es } from "date-fns/locale";

/**
 * Convierte timestamp de gRPC a Date object para MUI
 * @param {Object} timestamp - Timestamp de gRPC con { seconds, nanos }
 * @returns {Date}
 */
export const grpcTimestampToDate = (timestamp) => {
  if (!timestamp || !timestamp.seconds) return null;
  return new Date(timestamp.seconds * 1000 + timestamp.nanos / 1000000);
};

/**
 * Convierte Date object de MUI a timestamp para gRPC
 * @param {Date} date - Date object de MUI DateTimePicker
 * @returns {Object} - Timestamp para gRPC
 */
export const dateToGrpcTimestamp = (date) => {
  if (!date) return null;
  const timestamp = Math.floor(date.getTime() / 1000);
  const nanos = (date.getTime() % 1000) * 1000000;
  return {
    seconds: timestamp,
    nanos: nanos,
  };
};

/**
 * Formatea fecha para mostrar en la UI
 * @param {Date} date
 * @returns {string}
 */
export const formatDateForDisplay = (date) => {
  if (!date) return "";
  return format(date, "dd/MM/yyyy HH:mm", { locale: es });
};

/**
 * Valida que una fecha de inicio sea anterior a una de fin
 * @param {Date} startDate
 * @param {Date} endDate
 * @returns {boolean}
 */
export const validateDateRange = (startDate, endDate) => {
  if (!startDate || !endDate) return false;
  return startDate < endDate && startDate > new Date();
};

// Componente de ejemplo para crear subasta
// Archivo: frontend/src/components/CreateAuctionForm.jsx

import React, { useState } from "react";
import {
  Box,
  TextField,
  Button,
  Alert,
  Grid,
  Paper,
  Typography,
} from "@mui/material";
import { DateTimePicker } from "@mui/x-date-pickers/DateTimePicker";
import { LocalizationProvider } from "@mui/x-date-pickers/LocalizationProvider";
import { AdapterDateFns } from "@mui/x-date-pickers/AdapterDateFns";
import { es } from "date-fns/locale";
import {
  dateToGrpcTimestamp,
  validateDateRange,
  formatDateForDisplay,
} from "../utils/dateHelpers";

const CreateAuctionForm = () => {
  const [formData, setFormData] = useState({
    title: "",
    description: "",
    startTime: null,
    endTime: null,
    basePrice: "",
    minBidIncrement: "",
    currency: "USD",
  });
  const [errors, setErrors] = useState({});

  const handleDateChange = (field) => (newValue) => {
    setFormData((prev) => ({
      ...prev,
      [field]: newValue,
    }));

    // Validar fechas en tiempo real
    if (field === "startTime" || field === "endTime") {
      const start = field === "startTime" ? newValue : formData.startTime;
      const end = field === "endTime" ? newValue : formData.endTime;

      if (start && end && !validateDateRange(start, end)) {
        setErrors((prev) => ({
          ...prev,
          dateRange:
            "La fecha de inicio debe ser anterior a la fecha de fin y en el futuro",
        }));
      } else {
        setErrors((prev) => {
          const { dateRange, ...rest } = prev;
          return rest;
        });
      }
    }
  };

  const handleSubmit = async (e) => {
    e.preventDefault();

    // Validar fechas
    if (!validateDateRange(formData.startTime, formData.endTime)) {
      setErrors({ dateRange: "Fechas inválidas" });
      return;
    }

    // Preparar datos para gRPC
    const auctionData = {
      user_id: "user-123", // Obtener del contexto de autenticación
      item_id: "item-456", // Obtener del formulario o contexto
      title: formData.title,
      description: formData.description,
      start_time: dateToGrpcTimestamp(formData.startTime),
      end_time: dateToGrpcTimestamp(formData.endTime),
      base_price: formData.basePrice,
      min_bid_increment: formData.minBidIncrement,
      status: "active",
      currency: formData.currency,
    };

    try {
      // Llamar a tu servicio gRPC
      const response = await auctionService.createAuction(auctionData);
      console.log("Subasta creada:", response);
      // Redirect o mostrar éxito
    } catch (error) {
      console.error("Error al crear subasta:", error);
      setErrors({ submit: error.message });
    }
  };

  return (
    <LocalizationProvider dateAdapter={AdapterDateFns} adapterLocale={es}>
      <Paper elevation={3} sx={{ p: 3, maxWidth: 600, mx: "auto" }}>
        <Typography variant="h5" gutterBottom>
          Crear Nueva Subasta
        </Typography>

        <Box component="form" onSubmit={handleSubmit}>
          <Grid container spacing={2}>
            <Grid item xs={12}>
              <TextField
                fullWidth
                label="Título"
                value={formData.title}
                onChange={(e) =>
                  setFormData((prev) => ({ ...prev, title: e.target.value }))
                }
                required
              />
            </Grid>

            <Grid item xs={12}>
              <TextField
                fullWidth
                label="Descripción"
                multiline
                rows={3}
                value={formData.description}
                onChange={(e) =>
                  setFormData((prev) => ({
                    ...prev,
                    description: e.target.value,
                  }))
                }
              />
            </Grid>

            <Grid item xs={12} sm={6}>
              <DateTimePicker
                label="Fecha de Inicio"
                value={formData.startTime}
                onChange={handleDateChange("startTime")}
                renderInput={(params) => (
                  <TextField {...params} fullWidth required />
                )}
                minDateTime={new Date()} // No permitir fechas pasadas
              />
            </Grid>

            <Grid item xs={12} sm={6}>
              <DateTimePicker
                label="Fecha de Fin"
                value={formData.endTime}
                onChange={handleDateChange("endTime")}
                renderInput={(params) => (
                  <TextField {...params} fullWidth required />
                )}
                minDateTime={formData.startTime || new Date()}
              />
            </Grid>

            <Grid item xs={12} sm={6}>
              <TextField
                fullWidth
                label="Precio Base"
                type="number"
                value={formData.basePrice}
                onChange={(e) =>
                  setFormData((prev) => ({
                    ...prev,
                    basePrice: e.target.value,
                  }))
                }
                required
                InputProps={{ inputProps: { min: 0, step: 0.01 } }}
              />
            </Grid>

            <Grid item xs={12} sm={6}>
              <TextField
                fullWidth
                label="Incremento Mínimo"
                type="number"
                value={formData.minBidIncrement}
                onChange={(e) =>
                  setFormData((prev) => ({
                    ...prev,
                    minBidIncrement: e.target.value,
                  }))
                }
                required
                InputProps={{ inputProps: { min: 0.01, step: 0.01 } }}
              />
            </Grid>

            {errors.dateRange && (
              <Grid item xs={12}>
                <Alert severity="error">{errors.dateRange}</Alert>
              </Grid>
            )}

            {errors.submit && (
              <Grid item xs={12}>
                <Alert severity="error">{errors.submit}</Alert>
              </Grid>
            )}

            <Grid item xs={12}>
              <Button
                type="submit"
                variant="contained"
                fullWidth
                size="large"
                disabled={!!errors.dateRange}
              >
                Crear Subasta
              </Button>
            </Grid>
          </Grid>
        </Box>
      </Paper>
    </LocalizationProvider>
  );
};

export default CreateAuctionForm;
