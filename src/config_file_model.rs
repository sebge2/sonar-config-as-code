#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationFile {
    pub admin: Option<Admin>,
    pub properties: Option<Vec<Property>>,
    pub users: Option<Vec<User>>,
    pub groups: Option<Vec<Group>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Admin {
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Property {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub login: String,
    pub password: String,
    pub groups: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub description: String,
    pub permissions: Vec<String>,
}
