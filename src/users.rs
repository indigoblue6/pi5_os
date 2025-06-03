// User and Group Management for UNIX Compatibility
// POSIX user/group system implementation

use crate::uart::UART;
use heapless::{String, Vec};

const MAX_USERS: usize = 32;
const MAX_GROUPS: usize = 16;
const MAX_USERNAME: usize = 32;
const MAX_GROUPNAME: usize = 32;
const MAX_PASSWORD_HASH: usize = 64;
const MAX_HOME_PATH: usize = 64;
const MAX_SHELL_PATH: usize = 32;
const MAX_GECOS: usize = 128;

// User structure (similar to /etc/passwd)
#[derive(Debug, Clone)]
pub struct User {
    pub uid: u32,
    pub gid: u32,               // Primary group ID
    pub username: String<MAX_USERNAME>,
    pub password_hash: String<MAX_PASSWORD_HASH>,
    pub gecos: String<MAX_GECOS>, // Full name, office, phone, etc.
    pub home_dir: String<MAX_HOME_PATH>,
    pub shell: String<MAX_SHELL_PATH>,
    pub is_active: bool,
}

impl User {
    pub fn new(
        uid: u32,
        gid: u32,
        username: &str,
        password_hash: &str,
        gecos: &str,
        home_dir: &str,
        shell: &str,
    ) -> Result<Self, &'static str> {
        if username.len() > MAX_USERNAME {
            return Err("Username too long");
        }
        if password_hash.len() > MAX_PASSWORD_HASH {
            return Err("Password hash too long");
        }
        if home_dir.len() > MAX_HOME_PATH {
            return Err("Home directory path too long");
        }
        if shell.len() > MAX_SHELL_PATH {
            return Err("Shell path too long");
        }
        if gecos.len() > MAX_GECOS {
            return Err("GECOS field too long");
        }
        
        let mut user_name = String::new();
        let _ = user_name.push_str(username);
        
        let mut pass_hash = String::new();
        let _ = pass_hash.push_str(password_hash);
        
        let mut user_gecos = String::new();
        let _ = user_gecos.push_str(gecos);
        
        let mut user_home = String::new();
        let _ = user_home.push_str(home_dir);
        
        let mut user_shell = String::new();
        let _ = user_shell.push_str(shell);
        
        Ok(Self {
            uid,
            gid,
            username: user_name,
            password_hash: pass_hash,
            gecos: user_gecos,
            home_dir: user_home,
            shell: user_shell,
            is_active: true,
        })
    }
    
    pub fn is_root(&self) -> bool {
        self.uid == 0
    }
    
    pub fn verify_password(&self, password: &str) -> bool {
        // Simplified password verification
        // In real implementation, would use proper hashing (bcrypt, scrypt, etc.)
        if self.password_hash.starts_with("plain:") {
            let stored_password = &self.password_hash[6..];
            stored_password == password
        } else {
            // For demonstration, just compare directly
            self.password_hash.as_str() == password
        }
    }
}

// Group structure (similar to /etc/group)
#[derive(Debug, Clone)]
pub struct Group {
    pub gid: u32,
    pub groupname: String<MAX_GROUPNAME>,
    pub password_hash: String<MAX_PASSWORD_HASH>,
    pub members: Vec<u32, MAX_USERS>, // UIDs of group members
}

impl Group {
    pub fn new(gid: u32, groupname: &str, password_hash: &str) -> Result<Self, &'static str> {
        if groupname.len() > MAX_GROUPNAME {
            return Err("Group name too long");
        }
        if password_hash.len() > MAX_PASSWORD_HASH {
            return Err("Password hash too long");
        }
        
        let mut group_name = String::new();
        let _ = group_name.push_str(groupname);
        
        let mut pass_hash = String::new();
        let _ = pass_hash.push_str(password_hash);
        
        Ok(Self {
            gid,
            groupname: group_name,
            password_hash: pass_hash,
            members: Vec::new(),
        })
    }
    
    pub fn add_member(&mut self, uid: u32) -> Result<(), &'static str> {
        if self.members.contains(&uid) {
            return Ok(()); // Already a member
        }
        
        if self.members.is_full() {
            return Err("Group is full");
        }
        
        let _ = self.members.push(uid);
        Ok(())
    }
    
    pub fn remove_member(&mut self, uid: u32) -> bool {
        if let Some(pos) = self.members.iter().position(|&x| x == uid) {
            self.members.remove(pos);
            true
        } else {
            false
        }
    }
    
    pub fn is_member(&self, uid: u32) -> bool {
        self.members.contains(&uid)
    }
}

// User and Group Manager
pub struct UserManager {
    users: Vec<User, MAX_USERS>,
    groups: Vec<Group, MAX_GROUPS>,
    next_uid: u32,
    next_gid: u32,
    current_uid: u32,
    current_gid: u32,
}

impl UserManager {
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            groups: Vec::new(),
            next_uid: 1000, // Start regular users at 1000
            next_gid: 1000, // Start regular groups at 1000
            current_uid: 0, // Start as root
            current_gid: 0, // Start as root group
        }
    }
    
    pub fn init_system_users(&mut self) -> Result<(), &'static str> {
        // Create root user (UID 0)
        let root_user = User::new(
            0,
            0,
            "root",
            "plain:root", // Simple password for demo
            "root,,,",
            "/root",
            "/bin/sh",
        )?;
        let _ = self.users.push(root_user);
        
        // Create root group (GID 0)
        let root_group = Group::new(0, "root", "")?;
        let _ = self.groups.push(root_group);
        
        // Create wheel group (GID 1) for sudo-like functionality
        let wheel_group = Group::new(1, "wheel", "")?;
        let _ = self.groups.push(wheel_group);
        
        // Create users group (GID 100)
        let users_group = Group::new(100, "users", "")?;
        let _ = self.groups.push(users_group);
        
        // Create nobody user (UID 65534)
        let nobody_user = User::new(
            65534,
            65534,
            "nobody",
            "*", // No login
            "nobody,,,",
            "/",
            "/bin/false",
        )?;
        let _ = self.users.push(nobody_user);
        
        // Create nobody group (GID 65534)
        let nobody_group = Group::new(65534, "nobody", "")?;
        let _ = self.groups.push(nobody_group);
        
        UART.write_str("System users and groups initialized\n");
        Ok(())
    }
    
    pub fn create_user(
        &mut self,
        username: &str,
        password: &str,
        gecos: &str,
        home_dir: &str,
        shell: &str,
    ) -> Result<u32, &'static str> {
        // Check if username already exists
        for user in &self.users {
            if user.username.as_str() == username {
                return Err("Username already exists");
            }
        }
        
        if self.users.is_full() {
            return Err("Too many users");
        }
        
        let uid = self.next_uid;
        self.next_uid += 1;
        
        let mut password_hash: String<64> = String::new();
        let _ = password_hash.push_str("plain:");
        let _ = password_hash.push_str(password);
        let user = User::new(uid, 100, username, &password_hash, gecos, home_dir, shell)?;
        
        let _ = self.users.push(user);
        
        // Add user to default users group
        if let Some(users_group) = self.get_group_mut(100) {
            let _ = users_group.add_member(uid);
        }
        
        UART.write_str("Created user ");
        UART.write_str(username);
        UART.write_str(" with UID ");
        UART.put_hex(uid);
        UART.write_str("\n");
        
        Ok(uid)
    }
    
    pub fn create_group(&mut self, groupname: &str) -> Result<u32, &'static str> {
        // Check if group name already exists
        for group in &self.groups {
            if group.groupname.as_str() == groupname {
                return Err("Group name already exists");
            }
        }
        
        if self.groups.is_full() {
            return Err("Too many groups");
        }
        
        let gid = self.next_gid;
        self.next_gid += 1;
        
        let group = Group::new(gid, groupname, "")?;
        let _ = self.groups.push(group);
        
        UART.write_str("Created group ");
        UART.write_str(groupname);
        UART.write_str(" with GID ");
        UART.put_hex(gid);
        UART.write_str("\n");
        
        Ok(gid)
    }
    
    pub fn authenticate(&mut self, username: &str, password: &str) -> Result<u32, &'static str> {
        for user in &self.users {
            if user.username.as_str() == username && user.is_active {
                if user.verify_password(password) {
                    self.current_uid = user.uid;
                    self.current_gid = user.gid;
                    
                    UART.write_str("User ");
                    UART.write_str(username);
                    UART.write_str(" authenticated successfully\n");
                    
                    return Ok(user.uid);
                } else {
                    return Err("Invalid password");
                }
            }
        }
        
        Err("User not found")
    }
    
    pub fn get_user(&self, uid: u32) -> Option<&User> {
        self.users.iter().find(|u| u.uid == uid)
    }
    
    pub fn get_user_by_name(&self, username: &str) -> Option<&User> {
        self.users.iter().find(|u| u.username.as_str() == username)
    }
    
    pub fn get_group(&self, gid: u32) -> Option<&Group> {
        self.groups.iter().find(|g| g.gid == gid)
    }
    
    pub fn get_group_mut(&mut self, gid: u32) -> Option<&mut Group> {
        self.groups.iter_mut().find(|g| g.gid == gid)
    }
    
    pub fn get_group_by_name(&self, groupname: &str) -> Option<&Group> {
        self.groups.iter().find(|g| g.groupname.as_str() == groupname)
    }
    
    pub fn add_user_to_group(&mut self, uid: u32, gid: u32) -> Result<(), &'static str> {
        // Verify user exists
        if self.get_user(uid).is_none() {
            return Err("User not found");
        }
        
        // Add to group
        if let Some(group) = self.get_group_mut(gid) {
            group.add_member(uid)
        } else {
            Err("Group not found")
        }
    }
    
    pub fn remove_user_from_group(&mut self, uid: u32, gid: u32) -> Result<(), &'static str> {
        if let Some(group) = self.get_group_mut(gid) {
            if group.remove_member(uid) {
                Ok(())
            } else {
                Err("User not in group")
            }
        } else {
            Err("Group not found")
        }
    }
    
    pub fn is_user_in_group(&self, uid: u32, gid: u32) -> bool {
        if let Some(group) = self.get_group(gid) {
            group.is_member(uid)
        } else {
            false
        }
    }
    
    pub fn get_user_groups(&self, uid: u32) -> Vec<u32, MAX_GROUPS> {
        let mut groups = Vec::new();
        
        // Add primary group
        if let Some(user) = self.get_user(uid) {
            let _ = groups.push(user.gid);
        }
        
        // Add supplementary groups
        for group in &self.groups {
            if group.is_member(uid) && !groups.contains(&group.gid) {
                if !groups.is_full() {
                    let _ = groups.push(group.gid);
                }
            }
        }
        
        groups
    }
    
    pub fn check_permission(&self, uid: u32, required_uid: u32, required_gid: u32) -> bool {
        // Root can do anything
        if uid == 0 {
            return true;
        }
        
        // User can access their own resources
        if uid == required_uid {
            return true;
        }
        
        // Check group membership
        self.is_user_in_group(uid, required_gid)
    }
    
    pub fn switch_user(&mut self, target_uid: u32) -> Result<(), &'static str> {
        // Check if user exists and get user data first
        let (user_uid, user_gid, username) = {
            if let Some(user) = self.get_user(target_uid) {
                (user.uid, user.gid, user.username.clone())
            } else {
                return Err("User not found");
            }
        };
        
        // Only root can switch to any user, others can only switch to themselves
        if self.current_uid != 0 && self.current_uid != target_uid {
            return Err("Permission denied");
        }
        
        self.current_uid = user_uid;
        self.current_gid = user_gid;
        
        UART.write_str("Switched to user ");
        UART.write_str(username.as_str());
        UART.write_str(" (UID ");
        UART.put_hex(target_uid);
        UART.write_str(")\n");
        
        Ok(())
    }
    
    pub fn current_user(&self) -> (u32, u32) {
        (self.current_uid, self.current_gid)
    }
    
    pub fn is_root(&self) -> bool {
        self.current_uid == 0
    }
    
    pub fn list_users(&self) -> &[User] {
        &self.users
    }
    
    pub fn list_groups(&self) -> &[Group] {
        &self.groups
    }
    
    pub fn delete_user(&mut self, uid: u32) -> Result<(), &'static str> {
        if uid == 0 {
            return Err("Cannot delete root user");
        }
        
        // Remove from all groups
        for group in &mut self.groups {
            group.remove_member(uid);
        }
        
        // Remove user
        for (i, user) in self.users.iter().enumerate() {
            if user.uid == uid {
                let username = user.username.clone();
                self.users.remove(i);
                
                UART.write_str("Deleted user ");
                UART.write_str(username.as_str());
                UART.write_str(" (UID ");
                UART.put_hex(uid);
                UART.write_str(")\n");
                
                return Ok(());
            }
        }
        
        Err("User not found")
    }
    
    pub fn get_stats(&self) -> (usize, usize) {
        (self.users.len(), self.groups.len())
    }
}

// Global user manager
static mut GLOBAL_USER_MANAGER: UserManager = UserManager {
    users: Vec::new(),
    groups: Vec::new(),
    next_uid: 1000,
    next_gid: 1000,
    current_uid: 0,
    current_gid: 0,
};

pub fn init_users() -> Result<(), &'static str> {
    unsafe {
        GLOBAL_USER_MANAGER = UserManager::new();
        GLOBAL_USER_MANAGER.init_system_users()?;
    }
    UART.write_str("User management system initialized\n");
    Ok(())
}

pub fn create_user(
    username: &str,
    password: &str,
    gecos: &str,
    home_dir: &str,
    shell: &str,
) -> Result<u32, &'static str> {
    unsafe { GLOBAL_USER_MANAGER.create_user(username, password, gecos, home_dir, shell) }
}

pub fn authenticate_user(username: &str, password: &str) -> Result<u32, &'static str> {
    unsafe { GLOBAL_USER_MANAGER.authenticate(username, password) }
}

pub fn get_current_user() -> (u32, u32) {
    unsafe { GLOBAL_USER_MANAGER.current_user() }
}

pub fn switch_user(uid: u32) -> Result<(), &'static str> {
    unsafe { GLOBAL_USER_MANAGER.switch_user(uid) }
}

pub fn is_root() -> bool {
    unsafe { GLOBAL_USER_MANAGER.is_root() }
}

pub fn get_user_info(uid: u32) -> Option<(String<MAX_USERNAME>, u32, String<MAX_HOME_PATH>)> {
    unsafe {
        if let Some(user) = GLOBAL_USER_MANAGER.get_user(uid) {
            Some((user.username.clone(), user.gid, user.home_dir.clone()))
        } else {
            None
        }
    }
}

pub fn get_user_by_name(username: &str) -> Option<u32> {
    unsafe {
        if let Some(user) = GLOBAL_USER_MANAGER.get_user_by_name(username) {
            Some(user.uid)
        } else {
            None
        }
    }
}

pub fn check_permission(uid: u32, required_uid: u32, required_gid: u32) -> bool {
    unsafe { GLOBAL_USER_MANAGER.check_permission(uid, required_uid, required_gid) }
}

pub fn add_user_to_group(uid: u32, gid: u32) -> Result<(), &'static str> {
    unsafe { GLOBAL_USER_MANAGER.add_user_to_group(uid, gid) }
}

pub fn get_user_groups(uid: u32) -> Vec<u32, MAX_GROUPS> {
    unsafe { GLOBAL_USER_MANAGER.get_user_groups(uid) }
}

pub fn list_all_users() -> Vec<(u32, String<MAX_USERNAME>), MAX_USERS> {
    let mut result = Vec::new();
    unsafe {
        for user in GLOBAL_USER_MANAGER.list_users() {
            if !result.is_full() {
                let _ = result.push((user.uid, user.username.clone()));
            }
        }
    }
    result
}

pub fn list_all_groups() -> Vec<(u32, String<MAX_GROUPNAME>), MAX_GROUPS> {
    let mut result = Vec::new();
    unsafe {
        for group in GLOBAL_USER_MANAGER.list_groups() {
            if !result.is_full() {
                let _ = result.push((group.gid, group.groupname.clone()));
            }
        }
    }
    result
}

pub fn get_user_stats() -> (usize, usize) {
    unsafe { GLOBAL_USER_MANAGER.get_stats() }
}
