// src/security/authorization.rs
use std::collections::HashSet;
use uuid::Uuid;

pub struct Permission(String);

impl Permission {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}

pub struct Role {
    name: String,
    permissions: HashSet<String>,
}

impl Role {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            permissions: HashSet::new(),
        }
    }
    
    pub fn add_permission(&mut self, permission: &str) {
        self.permissions.insert(permission.to_string());
    }
    
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(permission)
    }
}

pub struct PermissionService {
    roles: std::collections::HashMap<String, Role>,
    user_roles: std::collections::HashMap<Uuid, HashSet<String>>,
}

impl PermissionService {
    pub fn new() -> Self {
        Self {
            roles: std::collections::HashMap::new(),
            user_roles: std::collections::HashMap::new(),
        }
    }
    
    pub fn add_role(&mut self, role: Role) {
        self.roles.insert(role.name.clone(), role);
    }
    
    pub fn assign_role_to_user(&mut self, user_id: Uuid, role_name: &str) {
        self.user_roles.entry(user_id)
            .or_insert_with(HashSet::new)
            .insert(role_name.to_string());
    }
    
    pub fn user_has_permission(&self, user_id: Uuid, permission: &str) -> bool {
        if let Some(user_roles) = self.user_roles.get(&user_id) {
            for role_name in user_roles {
                if let Some(role) = self.roles.get(role_name) {
                    if role.has_permission(permission) {
                        return true;
                    }
                }
            }
        }
        false
    }
}
