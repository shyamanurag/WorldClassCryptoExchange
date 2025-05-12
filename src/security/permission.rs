// src/security/permission.rs - Permission management
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use uuid::Uuid;
use log::{debug, info};

/// Permission service for role-based access control
pub struct PermissionService {
    /// User roles (user ID -> set of roles)
    user_roles: RwLock<HashMap<Uuid, HashSet<String>>>,
    
    /// Role permissions (role -> set of permissions)
    role_permissions: RwLock<HashMap<String, HashSet<String>>>,
}

impl PermissionService {
    /// Create a new permission service
    pub fn new() -> Self {
        let mut role_permissions = HashMap::new();
        
        // Define default roles and permissions
        let admin_permissions = HashSet::from([
            "users:read",
            "users:write",
            "users:delete",
            "trading:read",
            "trading:write",
            "trading:delete",
            "assets:read",
            "assets:write",
            "assets:delete",
            "wallets:read",
            "wallets:write",
            "metrics:read",
            "admin:access",
        ].map(String::from));
        
        let moderator_permissions = HashSet::from([
            "users:read",
            "trading:read",
            "trading:write",
            "assets:read",
            "wallets:read",
            "metrics:read",
        ].map(String::from));
        
        let user_permissions = HashSet::from([
            "trading:read",
            "trading:write",
            "assets:read",
            "wallets:read",
        ].map(String::from));
        
        role_permissions.insert("admin".to_string(), admin_permissions);
        role_permissions.insert("moderator".to_string(), moderator_permissions);
        role_permissions.insert("user".to_string(), user_permissions);
        
        info!("Initialized permission service with default roles and permissions");
        
        Self {
            user_roles: RwLock::new(HashMap::new()),
            role_permissions: RwLock::new(role_permissions),
        }
    }
    
    /// Assign a role to a user
    pub fn assign_role(&self, user_id: Uuid, role: &str) {
        let mut user_roles = self.user_roles.write().unwrap();
        
        user_roles
            .entry(user_id)
            .or_insert_with(HashSet::new)
            .insert(role.to_string());
        
        debug!("Assigned role '{}' to user {}", role, user_id);
    }
    
    /// Remove a role from a user
    pub fn remove_role(&self, user_id: Uuid, role: &str) {
        let mut user_roles = self.user_roles.write().unwrap();
        
        if let Some(roles) = user_roles.get_mut(&user_id) {
            roles.remove(role);
            debug!("Removed role '{}' from user {}", role, user_id);
        }
    }
    
    /// Get all roles for a user
    pub fn get_user_roles(&self, user_id: Uuid) -> HashSet<String> {
        let user_roles = self.user_roles.read().unwrap();
        
        user_roles
            .get(&user_id)
            .cloned()
            .unwrap_or_else(HashSet::new)
    }
    
    /// Check if a user has a specific role
    pub fn has_role(&self, user_id: Uuid, role: &str) -> bool {
        let user_roles = self.user_roles.read().unwrap();
        
        user_roles
            .get(&user_id)
            .map(|roles| roles.contains(role))
            .unwrap_or(false)
    }
    
    /// Add a permission to a role
    pub fn add_permission(&self, role: &str, permission: &str) {
        let mut role_permissions = self.role_permissions.write().unwrap();
        
        role_permissions
            .entry(role.to_string())
            .or_insert_with(HashSet::new)
            .insert(permission.to_string());
        
        debug!("Added permission '{}' to role '{}'", permission, role);
    }
    
    /// Remove a permission from a role
    pub fn remove_permission(&self, role: &str, permission: &str) {
        let mut role_permissions = self.role_permissions.write().unwrap();
        
        if let Some(permissions) = role_permissions.get_mut(role) {
            permissions.remove(permission);
            debug!("Removed permission '{}' from role '{}'", permission, role);
        }
    }
    
    /// Get all permissions for a role
    pub fn get_role_permissions(&self, role: &str) -> HashSet<String> {
        let role_permissions = self.role_permissions.read().unwrap();
        
        role_permissions
            .get(role)
            .cloned()
            .unwrap_or_else(HashSet::new)
    }
    
    /// Get all permissions for a user
    pub fn get_user_permissions(&self, user_id: Uuid) -> HashSet<String> {
        let user_roles = self.get_user_roles(user_id);
        let role_permissions = self.role_permissions.read().unwrap();
        
        let mut permissions = HashSet::new();
        
        for role in user_roles {
            if let Some(role_perms) = role_permissions.get(&role) {
                permissions.extend(role_perms.clone());
            }
        }
        
        permissions
    }
    
    /// Check if a user has a specific permission
    pub fn has_permission(&self, user_id: Uuid, permission: &str) -> bool {
        let permissions = self.get_user_permissions(user_id);
        
        permissions.contains(permission)
    }
    
    /// Check if a user has any of the specified permissions
    pub fn has_any_permission(&self, user_id: Uuid, permissions: &[&str]) -> bool {
        let user_permissions = self.get_user_permissions(user_id);
        
        permissions.iter().any(|&p| user_permissions.contains(p))
    }
    
    /// Check if a user has all of the specified permissions
    pub fn has_all_permissions(&self, user_id: Uuid, permissions: &[&str]) -> bool {
        let user_permissions = self.get_user_permissions(user_id);
        
        permissions.iter().all(|&p| user_permissions.contains(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_assign_and_check_role() {
        let service = PermissionService::new();
        let user_id = Uuid::new_v4();
        
        service.assign_role(user_id, "admin");
        
        assert!(service.has_role(user_id, "admin"));
        assert!(!service.has_role(user_id, "user"));
    }
    
    #[test]
    fn test_remove_role() {
        let service = PermissionService::new();
        let user_id = Uuid::new_v4();
        
        service.assign_role(user_id, "admin");
        service.remove_role(user_id, "admin");
        
        assert!(!service.has_role(user_id, "admin"));
    }
    
    #[test]
    fn test_get_user_roles() {
        let service = PermissionService::new();
        let user_id =
