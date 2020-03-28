use std::vec::Vec;

use config_file_model::{Group, Admin};
use config_file_model::Property;
use config_file_model::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarPaging {
    #[serde(rename(deserialize = "pageIndex"))]
    pub page_index: usize,

    #[serde(rename(deserialize = "pageSize"))]
    pub page_size: usize,

    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarGroupCreationRequest {
    pub name: String,
    pub description: String,

    #[serde(skip_deserializing)]
    pub permissions: Vec<String>,
}

impl SonarGroupCreationRequest {
    pub fn from_configuration_group(group: &Group,  _resolver: fn(&String) -> String) -> SonarGroupCreationRequest {
        SonarGroupCreationRequest {
            name: _resolver(&group.name),
            description: _resolver(&group.description),
            permissions: group.permissions.clone().into_iter().map(|permission| _resolver(&permission)).rev().collect()
        }
    }

    pub fn from_configuration_groups(groups: &Vec<Group>,  _resolver: fn(&String) -> String) -> Vec<SonarGroupCreationRequest> {
        let mut mapped = Vec::new();
        for i in 0..groups.len() {
            mapped.push(SonarGroupCreationRequest::from_configuration_group(groups.get(i).unwrap(), _resolver));
        }

        return mapped;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarGroups {
    pub paging: SonarPaging,
    pub groups: Vec<SonarGroup>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarGroup {
    pub id: usize,
    pub name: String,
    pub description: String,

    #[serde(skip_deserializing)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarProperty {
    pub name: String,
    pub value: String,
}

impl SonarProperty {
    pub fn from_configuration_property(property: &Property, _resolver: fn(&String) -> String) -> SonarProperty {
        SonarProperty {
            name: _resolver(&property.name),
            value: _resolver(&property.value),
        }
    }

    pub fn from_configuration_properties(properties: &Vec<Property>, _resolver: fn(&String) -> String) -> Vec<SonarProperty> {
        let mut mapped = Vec::new();
        for i in 0..properties.len() {
            mapped.push(SonarProperty::from_configuration_property(properties.get(i).unwrap(), _resolver));
        }

        return mapped;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarAdminUpdateRequest {
    pub password: Option<String>,
}

impl SonarAdminUpdateRequest {

    pub fn from_configuration(admin: &Admin, _resolver: fn(&String) -> String) -> SonarAdminUpdateRequest {
        SonarAdminUpdateRequest {
            password: admin.password.as_ref().map(|password| _resolver(&password.to_string()))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarUser {
    pub login: String,
    pub name: String,
    pub password: Option<String>,
    pub groups: Vec<String>,
}

impl SonarUser {
    pub fn from_configuration_user(user: &User, _resolver: fn(&String) -> String) -> SonarUser {
        SonarUser {
            login: _resolver(&user.login),
            name: _resolver(&user.name),
            password: Some(_resolver(&user.password)),
            groups: user.groups.clone().unwrap_or(Vec::new()).into_iter().map(|group| _resolver(&group)).collect(),
        }
    }

    pub fn from_configuration_users(users: &Vec<User>, _resolver: fn(&String) -> String) -> Vec<SonarUser> {
        let mut mapped = Vec::new();
        for i in 0..users.len() {
            mapped.push(SonarUser::from_configuration_user(users.get(i).unwrap(), _resolver));
        }

        return mapped;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarUsers {
    pub paging: SonarPaging,
    pub users: Vec<SonarUser>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarGroupMembers {
    pub users: Vec<SonarGroupMembership>,

    #[serde(rename(deserialize = "p"))]
    pub page_index: usize,

    #[serde(rename(deserialize = "ps"))]
    pub page_size: usize,

    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarGroupMembership {
    pub name: String,
    pub login: String,
    pub selected: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarUserGroups {
    pub paging: SonarPaging,
    pub groups: Vec<SonarUserMembership>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarUserMembership {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub selected: bool,
    pub default: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarPermissionTemplates {
    #[serde(rename(deserialize = "permissionTemplates"))]
    pub permission_templates: Vec<SonarPermissionTemplate>,

    #[serde(rename(deserialize = "defaultTemplates"))]
    pub default_templates: Vec<SonarDefaultTemplate>,

    pub permissions: Vec<SonarPermission>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarDefaultTemplate {
    #[serde(rename(deserialize = "templateId"))]
    pub template_id: String,

    pub qualifier: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarPermission {
    pub key: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarPermissionTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub permissions: Vec<SonarPermissionTemplatePermission>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarPermissionTemplatePermission {
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarGeneratedToken {
    pub login: String,
    pub name: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarErrors {
    pub errors: Option<Vec<SonarError>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarError {
    pub msg: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SonarAuthenticationValidationResult {
    pub valid: bool,
}
