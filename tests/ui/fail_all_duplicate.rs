// error-pattern: E0382
fn main() {
    let token = String::from("owned");
    let _ = gp::all!(token, token);
}
