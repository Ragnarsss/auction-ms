// Importa la función dotenv para cargar variables de entorno desde un archivo .env
use dotenvy::dotenv;
// Importa el módulo env para acceder a variables de entorno del sistema
use std::env;

// Define una función pública llamada init
pub fn init() {
    // Carga las variables de entorno desde el archivo .env, si existe
    dotenv().ok();
    // Inicializa el logger de entorno para registrar mensajes en la consola
    env_logger::init();
}
