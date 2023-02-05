#[nusion::entry]
pub fn entry() -> Result<(), Box<dyn std::error::Error>> {
   println!("Hello from the other side!");
   std::thread::sleep(std::time::Duration::from_secs(5));
   return Ok(());
}

