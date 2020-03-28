use reqwest::StatusCode;
use retry::{Error, OperationResult};
use retry::delay::Fixed;
use retry::retry;
use serde::de::DeserializeOwned;

use sonar_api_model::{SonarAuthenticationValidationResult, SonarErrors, SonarGeneratedToken, SonarGroup, SonarGroupCreationRequest, SonarGroups, SonarPermissionTemplates, SonarProperty, SonarUser, SonarUserGroups, SonarUsers, SonarAdminUpdateRequest};

pub struct SonarApi {
    url: String,
    username: String,
    password: String,
}

// See documentation on [URL]/web_api/
impl SonarApi {
    pub fn new(url: String,
               username: String,
               _password_provider: PasswordProvider,
               number_attempts: usize) -> SonarApi {
        let base_url = if url.ends_with("/") { url.chars().take(url.len() - 1).collect() } else { url };

        match SonarApi::wait_ready(&base_url, number_attempts) {
            Ok(_) => {
                let password = SonarApi::find_password(&base_url, &username, _password_provider);

                SonarApi { url: base_url, username: username.to_string(), password }
            }
            // TODO avoid panic
            Err(_error) => panic!(format!("Error while trying to connect to the API: {:#?}.", _error))
        }
    }

    pub fn set_property(&self, property: &SonarProperty) {
        debug!("Setting property [{}] = [{}].", property.name, property.value);

        let resp = self.execute_post(
            self.build_url("/api/settings/set", &vec![("key", property.name.as_str()), ("value", property.value.as_str())]).as_str()
        );

        SonarApi::assert_response(resp, format!("Error while setting property [{}].", property.name));
    }

    pub fn create_group(&self, group: &SonarGroupCreationRequest) {
        debug!("Creating group [{}].", group.name);

        if !self.group_exists(group.name.as_str()) {
            let resp = self.execute_post(
                self.build_url("/api/user_groups/create", &vec![("name", group.name.as_str()), ("description", group.description.as_str())]).as_str()
            );

            SonarApi::assert_response(resp, format!("Error while creating group [{}].", group.name));
        } else {
            let group_id = self.get_group_by_name(group.name.as_str()).expect("Expecting a group.").id.to_string();
            let resp =
                self.execute_post(
                    self.build_url(
                        "/api/user_groups/update",
                        &vec![("id", group_id.as_str()), ("description", group.description.as_str())],
                    ).as_str()
                );

            SonarApi::assert_response(resp, format!("Error while updating group [{}].", group.name));
        }

        let templates = self.get_permission_templates();
        for permission in templates.permissions {
            if group.permissions.contains(&permission.key) {
                self.add_permission_to_group(&group.name, &permission.key);
            } else {
                self.remove_permission_to_group(&group.name, &permission.key);
            }
        }
    }

    pub fn add_permission_to_group(&self, group: &String, permission: &String) {
        debug!("Assign permission [{}] to group [{}].", permission, group);

        let resp = self.execute_post(
            self.build_url(
                "/api/permissions/add_group_to_template",
                &vec![("groupName", group.as_str()), ("permission", permission.as_str()), ("templateId", ::DEFAULT_TEMPLATE_NAME)],
            ).as_str()
        );

        SonarApi::assert_response(resp, format!("Error while creating permission [{}].", permission));
    }

    pub fn remove_permission_to_group(&self, group: &String, permission: &String) {
        debug!("Remove permission [{}] to group [{}].", permission, group);

        let resp = self.execute_post(
            self.build_url(
                "/api/permissions/remove_group_from_template",
                &vec![("groupName", group), ("permission", permission), ("templateId", ::DEFAULT_TEMPLATE_NAME)],
            ).as_str()
        );

        SonarApi::assert_response(resp, format!("Error while removing permission [{}].", permission));
    }

    pub fn group_exists(&self, _name: &str) -> bool {
        return self.get_group_by_name(_name).is_some();
    }

    pub fn create_user(&mut self, user: &SonarUser) {
        debug!("Creating user [{}].", user.login);

        if user.login.eq(::ADMIN_USER) {
            panic!("Cannot update admin user.");
        }

        if !self.user_exists(user.login.as_str()) {
            let password = user.password.clone().unwrap_or("password".to_string());

            let resp = self.execute_post(
                self.build_url(
                    "/api/users/create",
                    &vec![("login", user.login.as_str()), ("name", user.name.as_str()), ("password", password.as_str())],
                ).as_str()
            );

            SonarApi::assert_response(resp, format!("Error while creating user [{}].", user.login));
        } else {
            let resp = self.execute_post(
                self.build_url("/api/users/update", &vec![("login", user.login.as_str()), ("name", user.name.as_str())]).as_str()
            );

            SonarApi::assert_response(resp, format!("Error while updating user [{}].", user.login));

            if user.password.is_some() {
                self.change_user_password(&user.login, user.password.as_ref().unwrap());
            }
        }

        let current_user_groups = self.get_user_groups(&user.login.to_string());

        for current_user_group in &current_user_groups {
            if user.groups.contains(current_user_group) && current_user_group != ::DEFAULT_GROUP {
                self.remove_user_from_group(&user.login, current_user_group);
            }
        }

        for group in &user.groups {
            if !current_user_groups.contains(group) {
                self.add_user_to_group(&user.login, group);
            }
        }
    }

    pub fn user_exists(&self, _login: &str) -> bool {
        let resp = self.execute_get(self.build_url("/api/users/search", &vec![("q", _login)]).as_str());

        let users: SonarUsers = SonarApi::assert_deserialize_response::<SonarUsers>(resp, format!("Cannot deserialize response checking if login [{}] exists.", _login));

        for user in &users.users {
            if user.login == _login {
                return true;
            }
        }

        return false;
    }

    pub fn add_user_to_group(&self, user: &String, group: &String) {
        debug!("Add user [{}] to group [{}].", user, group);

        let resp = self.execute_post(
            self.build_url("/api/user_groups/add_user", &vec![("login", user), ("name", group)]).as_str()
        );

        SonarApi::assert_response(resp, format!("Error while adding user [{}] to group [{}].", user, group));
    }

    pub fn remove_user_from_group(&self, user: &String, group: &String) {
        debug!("Remove user [{}] from group [{}].", user, group);

        let resp = self.execute_post(
            self.build_url("/api/user_groups/remove_user", &vec![("login", user), ("name", group)]).as_str()
        );

        SonarApi::assert_response(resp, format!("Error while removing user [{}] from user [{}].", user, group));
    }

    pub fn get_user_groups(&self, _user: &String) -> Vec<String> {
        let resp = self.execute_get(self.build_url("/api/users/groups", &vec![("login", _user)]).as_str());

        let groups: SonarUserGroups =
            SonarApi::assert_deserialize_response::<SonarUserGroups>(resp, format!("Cannot deserialize response retrieving groups of user [{}].", _user));

        if groups.paging.page_size == groups.paging.total {
            panic!(format!("Pagination of user groups is not supported. The user is [{}].", _user))
        }

        return groups.groups.iter().map(|member_ship| member_ship.name.to_string()).collect::<Vec<String>>();
    }

    pub fn get_permission_templates(&self) -> SonarPermissionTemplates {
        let resp = self.execute_get(self.build_url("/api/permissions/search_templates", &vec![("q", ::DEFAULT_TEMPLATE_NAME)]).as_str());

        return SonarApi::assert_deserialize_response::<SonarPermissionTemplates>(resp, "Cannot deserialize permission templates.".to_string());
    }

    pub fn change_user_password(&mut self, user: &String, password: &String) {
        let resp =
            self.execute_post(
                self.build_url("/api/users/change_password", &vec![("login", user), ("password", password)]).as_str()
            );

        SonarApi::assert_response(resp, format!("Error while changing user's password [{}].", user));
    }

    pub fn change_current_user_password(&mut self, password: &String) {
        if self.password.ne(password) {
            let resp = self.execute_post(
                self.build_url(
                    "/api/users/change_password",
                    &vec![("login", self.username.as_str()), ("password", password.as_str()), ("previousPassword", self.password.as_str())],
                ).as_str()
            );


            SonarApi::assert_response(resp, format!("Error while changing user's password [{}].", &self.username));

            self.password = password.to_string();
        }
    }

    pub fn update_admin(&mut self, update_request: &SonarAdminUpdateRequest) {
        if update_request.password.is_some() {
            self.change_current_user_password(&update_request.password.as_ref().unwrap());
        }
    }

    pub fn generate_user_token(&self, user: &String, name: &String) -> String {
        let resp = self.execute_post(
            self.build_url("/api/user_tokens/generate", &vec![("login", user), ("name", name)]).as_str()
        );

        return SonarApi::assert_deserialize_response::<SonarGeneratedToken>(resp, format!("Error while generating user token [{}].", name)).token;
    }

    fn wait_ready(url: &String, number_attempts: usize) -> Result<reqwest::StatusCode, Error<reqwest::StatusCode>> {
        debug!("Checking if SonarQube is available on URL [{}].", url);

        let mut attempt = 0;
        return retry(Fixed::from_millis(1000).take(number_attempts), || {
            attempt = attempt + 1;
            debug!("Attempt number to connect to the API {}.", attempt);

            let resp = reqwest::blocking::Client::new()
                .get(&[url, "/api/server/version"].concat())
                .send();

            match resp {
                Ok(resp) => match resp.status() {
                    StatusCode::OK => OperationResult::Ok(resp.status()),
                    x => OperationResult::Retry(x),
                },
                Err(_error) => OperationResult::Retry(StatusCode::SERVICE_UNAVAILABLE)
            }
        });
    }

    fn find_password(url: &String, username: &String, _password_provider: PasswordProvider) -> String {
        for password in _password_provider.strategies {
            if password.is_some() {
                let password = password.unwrap().to_string();
                let resp = reqwest::blocking::Client::new()
                    .get(&[url, "/api/authentication/validate"].concat())
                    .basic_auth(username, Some(&password))
                    .send();

                match resp {
                    Ok(resp) => {
                        match resp.status() {
                            StatusCode::OK => {
                                let x = resp.json::<SonarAuthenticationValidationResult>()
                                    .expect("Cannot deserialize authentication check result.");

                                if x.valid {
                                    return password;
                                } else {
                                    continue;
                                }
                            }
                            StatusCode::UNAUTHORIZED => continue,
                            _ => continue // TODO error
                        }
                    }
                    _ => continue // TODO error
                }
            }
        }

        // TODO avoid panic
        panic!(format!("Cannot authenticate user [{}]. Please check its credentials.", username));
    }

    fn execute_get(&self, path: &str) -> reqwest::blocking::Response {
        return reqwest::blocking::Client::new()
            .get(path)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .expect("Error while connecting to SonarQube.");
    }

    fn execute_post(&self, path: &str) -> reqwest::blocking::Response {
        return reqwest::blocking::Client::new()
            .post(path)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .expect("Error while connecting to SonarQube.");
    }

    fn build_url(&self, path: &str, iter: &Vec<(&str, &str)>) -> String {
        return reqwest::Url::parse_with_params(&format!("{}{}", self.url, &path), iter).unwrap().to_string();
    }

    fn assert_response(resp: reqwest::blocking::Response, msg: String) {
        if !resp.status().is_success() {
            panic!(format!("{}.\nResponse ({}):\n{:#?}", msg, resp.status(), resp.json::<SonarErrors>()));
        }
    }

    fn assert_deserialize_response<T: DeserializeOwned>(resp: reqwest::blocking::Response, msg: String) -> T {
        if !resp.status().is_success() {
            // TODO avoid panic
            panic!(format!("{}.\nResponse ({}):\n{:#?}", msg, resp.status(), resp.json::<SonarErrors>()));
        } else {
            return resp.json::<T>()
                .expect(msg.as_str());
        }
    }

    fn get_group_by_name(&self, _name: &str) -> Option<SonarGroup> {
        let resp = self.execute_get(self.build_url("/api/user_groups/search", &vec![("q", _name)]).as_str());

        let groups: SonarGroups = resp.json::<SonarGroups>()
            .expect("Cannot deserialize response.");

        for group in groups.groups {
            if group.name == _name {
                return Some(group);
            }
        }

        return None;
    }
}

pub struct PasswordProvider {
    pub strategies: Vec<Option<String>>,
}

impl PasswordProvider {
    pub fn specific_password(password: String) -> PasswordProvider {
        PasswordProvider { strategies: vec![Some(password)] }
    }

    pub fn password_or_fallback(password: Option<String>, fallback_password: Option<String>) -> PasswordProvider {
        PasswordProvider { strategies: vec![password, fallback_password] }
    }
}
