
use crate::helpers::helpers::get_next_element;

pub fn handle_echo_request(lines: &mut std::str::Lines) -> String {
    let echo_line = get_next_element(lines); // Skip the first line
    let len_of_echo_line = echo_line.len();
    let echo_response = format!("${}\r\n{}\r\n", len_of_echo_line, echo_line);
    return echo_response;
}