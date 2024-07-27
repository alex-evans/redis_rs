
pub fn handle_info_request() -> String {
    let role: String = "master".to_string();
    let response = format!("${}\r\nrole:{}\r\n", role.len(), role);
    return response;
}