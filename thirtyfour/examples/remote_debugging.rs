//! Requires chromedriver running on port 9515:
//!
//!     chromedriver --port=9515
//!     chrome --remote-debugging-port=9222 --user-data-dir="H:\WebWorkspace\tauri-learn\browser-data"
//!
//! Run as follows:
//!
//!     cargo run --example tokio_async

use thirtyfour::prelude::*;
use tokio;

#[tokio::main]
async fn main() -> Result<(), WebDriverError> {
    let mut caps = DesiredCapabilities::chrome();
    caps.set_debugger_address("localhost:9222")?;
    let driver = WebDriver::new("http://localhost:9515", caps).await?;
    driver.goto("https://www.baidu.com").await?;
    Ok(())
}
