use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Role {
    Admin,       // Может выдавать сертификаты
    Dispatcher,  // Может читать SOS
    Medic,       // Волонтер-медик
    Transporter, // Волонтер-водитель
    Civilian,    // Обычный пользователь
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VolunteerCredential {
    pub volunteer_did: String,     // ID волонтера (Public Key)
    pub organization_did: String,  // ID организации, выдавшей сертификат
    pub role: Role,
    pub valid_until: u64,
    pub signature: Vec<u8>,        // Подпись организации
}

// Заявка на волонтерство
#[derive(Serialize, Deserialize, Debug)]
pub struct VolunteerApplication {
    pub name: String,
    pub skills: Vec<String>,
    pub contact_info: String, // Может быть зашифровано ключом организации
}
