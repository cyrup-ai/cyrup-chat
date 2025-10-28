use google_oauth::{Login, AccessType, CallbackMode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("Testing OAuth listener...");
    
    let port = 8080;
    let callback_mode = CallbackMode::Server { port };
    
    let result = Login::from_env()
        .scopes(["https://www.googleapis.com/auth/userinfo.email"])?
        .access_type(AccessType::Offline)
        .callback_mode(callback_mode)
        .timeout(std::time::Duration::from_secs(30))
        .login()
        .await;
        
    match result {
        Ok(_response) => println!("Success! OAuth authentication completed successfully"),
        Err(e) => println!("Error: {:?}", e),
    }
    
    Ok(())
}