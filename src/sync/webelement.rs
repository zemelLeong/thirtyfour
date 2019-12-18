use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::write;

use base64::decode;

use crate::common::command::Command;
use crate::common::connection_common::unwrap;
use crate::common::keys::TypingData;
use crate::common::types::{ElementId, ElementRect, ElementRef, SessionId};
use crate::error::WebDriverResult;
use crate::sync::RemoteConnectionSync;
use crate::By;

/// Unwrap the raw JSON into a WebElement struct.
pub fn unwrap_element_sync(
    conn: Arc<RemoteConnectionSync>,
    session_id: SessionId,
    value: &serde_json::Value,
) -> WebDriverResult<WebElement> {
    let elem_id: ElementRef = serde_json::from_value(value.clone())?;
    Ok(WebElement::new(
        conn,
        session_id,
        ElementId::from(elem_id.id),
    ))
}

/// Unwrap the raw JSON into a Vec of WebElement structs.
pub fn unwrap_elements_sync(
    conn: &Arc<RemoteConnectionSync>,
    session_id: &SessionId,
    value: &serde_json::Value,
) -> WebDriverResult<Vec<WebElement>> {
    let values: Vec<ElementRef> = serde_json::from_value(value.clone())?;
    Ok(values
        .into_iter()
        .map(|x| WebElement::new(conn.clone(), session_id.clone(), ElementId::from(x.id)))
        .collect())
}

/// The WebElement struct encapsulates a single element on a page.
///
/// WebElement structs are generally not constructed manually, but rather
/// they are returned from a 'find_element()' operation using a WebDriver.
///
/// # Example:
/// ```ignore
/// use thirtyfour::By;
/// let elem = driver.find_element(By::Name("elementName"))?;
/// ```
///
/// You can also search for a child element of another element as follows:
/// ```ignore
/// use thirtyfour::By;
/// let elem = driver.find_element(By::Css("div.myclass"))?;
/// let child_elem = elem.find_element(By::Name("elementName"))?;
/// ```
///
/// Elements can be clicked using the `click()` method, and you can send
/// input to an element using the `send_keys()` method.
///
#[derive(Debug, Clone)]
pub struct WebElement {
    pub element_id: ElementId,
    session_id: SessionId,
    conn: Arc<RemoteConnectionSync>,
}

impl WebElement {
    /// Create a new WebElement struct.
    ///
    /// Typically you would not call this directly. WebElement structs are
    /// usually constructed by calling one of the find_element*() methods
    /// either on WebDriver or another WebElement.
    pub fn new(
        conn: Arc<RemoteConnectionSync>,
        session_id: SessionId,
        element_id: ElementId,
    ) -> Self {
        WebElement {
            conn,
            session_id,
            element_id,
        }
    }

    /// Get the bounding rectangle for this WebElement.
    pub fn rect(&self) -> WebDriverResult<ElementRect> {
        let v = self
            .conn
            .execute(Command::GetElementRect(&self.session_id, &self.element_id))?;
        let r: ElementRect = serde_json::from_value((&v["value"]).clone())?;
        Ok(r)
    }

    /// Get the tag name for this WebElement.
    pub fn tag_name(&self) -> WebDriverResult<String> {
        let v = self.conn.execute(Command::GetElementTagName(
            &self.session_id,
            &self.element_id,
        ))?;
        unwrap(&v["value"])
    }

    /// Get the text contents for this WebElement.
    pub fn text(&self) -> WebDriverResult<String> {
        let v = self
            .conn
            .execute(Command::GetElementText(&self.session_id, &self.element_id))?;
        unwrap(&v["value"])
    }

    /// Click the WebElement.
    pub fn click(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::ElementClick(&self.session_id, &self.element_id))?;
        Ok(())
    }

    /// Clear the WebElement contents.
    pub fn clear(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::ElementClear(&self.session_id, &self.element_id))?;
        Ok(())
    }

    /// Get the specified property.
    pub fn get_property(&self, name: &str) -> WebDriverResult<String> {
        let v = self.conn.execute(Command::GetElementProperty(
            &self.session_id,
            &self.element_id,
            name.to_owned(),
        ))?;
        unwrap(&v["value"])
    }

    /// Get the specified attribute.
    pub fn get_attribute(&self, name: &str) -> WebDriverResult<String> {
        let v = self.conn.execute(Command::GetElementAttribute(
            &self.session_id,
            &self.element_id,
            name.to_owned(),
        ))?;
        unwrap(&v["value"])
    }

    /// Get the specified CSS property.
    pub fn get_css_property(&self, name: &str) -> WebDriverResult<String> {
        let v = self.conn.execute(Command::GetElementCSSValue(
            &self.session_id,
            &self.element_id,
            name.to_owned(),
        ))?;
        unwrap(&v["value"])
    }

    /// Return true if the WebElement is currently selected, otherwise false.
    pub fn is_selected(&self) -> WebDriverResult<bool> {
        let v = self.conn.execute(Command::IsElementSelected(
            &self.session_id,
            &self.element_id,
        ))?;
        unwrap(&v["value"])
    }

    /// Return true if the WebElement is currently enabled, otherwise false.
    pub fn is_enabled(&self) -> WebDriverResult<bool> {
        let v = self.conn.execute(Command::IsElementEnabled(
            &self.session_id,
            &self.element_id,
        ))?;
        unwrap(&v["value"])
    }

    /// Search for a child element of this WebElement using the specified
    /// selector.
    ///
    /// # Example:
    /// ```ignore
    /// use thirtyfour::By;
    ///
    /// let child_elem = elem.find_element(By::Id("theElementId"))?;
    /// ```
    pub fn find_element(&self, by: By) -> WebDriverResult<WebElement> {
        let v = self.conn.execute(Command::FindElementFromElement(
            &self.session_id,
            &self.element_id,
            by,
        ))?;
        unwrap_element_sync(self.conn.clone(), self.session_id.clone(), &v["value"])
    }

    /// Search for all child elements of this WebElement that match the
    /// specified selector.
    ///
    /// # Example:
    /// ```ignore
    /// let child_elems = elem.find_elements(By::Class("some-class"))?;
    /// for child_elem in child_elems {
    ///     println!("Found child element: {}", child_elem);
    /// }
    /// ```
    pub fn find_elements(&self, by: By) -> WebDriverResult<Vec<WebElement>> {
        let v = self.conn.execute(Command::FindElementsFromElement(
            &self.session_id,
            &self.element_id,
            by,
        ))?;
        unwrap_elements_sync(&self.conn, &self.session_id, &v["value"])
    }

    /// Send the specified input.
    pub fn send_keys(&self, keys: TypingData) -> WebDriverResult<()> {
        self.conn.execute(Command::ElementSendKeys(
            &self.session_id,
            &self.element_id,
            keys,
        ))?;
        Ok(())
    }

    /// Take a screenshot of this WebElement and return it as a base64-encoded
    /// String.
    pub fn screenshot_as_base64(&self) -> WebDriverResult<String> {
        let v = self.conn.execute(Command::TakeElementScreenshot(
            &self.session_id,
            &self.element_id,
        ))?;
        unwrap(&v["value"])
    }

    /// Take a screenshot of this WebElement and return it as PNG bytes.
    pub fn screenshot_as_png(&self) -> WebDriverResult<Vec<u8>> {
        let s = self.screenshot_as_base64()?;
        let bytes: Vec<u8> = decode(&s)?;
        Ok(bytes)
    }

    /// Take a screenshot of this WebElement and write it to the specified
    /// filename.
    pub fn screenshot(&self, path: &Path) -> WebDriverResult<()> {
        let png = self.screenshot_as_png()?;
        let mut file = File::create(path)?;
        file.write_all(&png)?;
        Ok(())
    }
}

impl fmt::Display for WebElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            r#"(session="{}", element="{}")"#,
            self.session_id, self.element_id
        )
    }
}