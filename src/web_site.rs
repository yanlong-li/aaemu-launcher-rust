pub fn open_website(url:&str) -> Result<(), Box<dyn std::error::Error>> {

    opener::open(url)?;

    Ok(())
}