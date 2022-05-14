use std::fs::File;
use std::io::Write;
use std::thread::sleep;
use std::time::{Duration, Instant};

use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::Browser;

fn main() -> anyhow::Result<()> {
    browse()
}

fn browse() -> anyhow::Result<()> {
    let browser = Browser::default()?;

    let tab = browser.wait_for_initial_tab()?;
    tab.navigate_to("http://localhost/yarrosco/yarrosco_chat.html")?;
    let _content = tab.wait_for_element("#content")?;
    sleep(Duration::from_secs_f32(0.5));
    let now = Instant::now();
    // Trying to make the background transparent (this will not work as is, it has to be sent elsewhere)
    // let r = tab.evaluate(
    //     "client.Emulation.setDefaultBackgroundColorOverride({
    //     color: { r: 1, g: 0, b: 0, a: 0 }
    //   })",
    //     true,
    // )?;
    // dbg!(r);
    let png_data =
        tab.capture_screenshot(CaptureScreenshotFormatOption::Png, Some(0), None, true)?;
    dbg!(now.elapsed());

    let mut file = File::create("test.png")?;
    file.write_all(&png_data)?;
    dbg!(now.elapsed());
    Ok(())
}
