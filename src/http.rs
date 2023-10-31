use std::collections::HashMap;

use anyhow::{bail, Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite};

use crate::types::ReadHalfBuf;

#[derive(PartialEq, Eq, Debug)]
pub enum HTTPReadState {
    Info,
    Headers,
    Body,
}

#[derive(Debug)]
pub struct HTTPObject {
    pub method: String,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub state: HTTPReadState,
    pub status_code: String,
    pub status: String,
}

// TODO: Split this impl into a trait and two different struct.
impl HTTPObject {
    /// Creates a new empty HTTPObject that you can use as request or response abstraction.
    pub fn new() -> HTTPObject {
        HTTPObject {
            method: String::new(),
            uri: String::new(),
            headers: HashMap::new(),
            body: vec![],
            state: HTTPReadState::Info,
            status: String::new(),
            status_code: String::new(),
        }
    }

    /// Reads the first line of the HTTP response
    pub fn read_response(&mut self, line: &String) -> Result<()> {
        let items = line.split(' ').collect::<Vec<_>>();

        if items.len() < 3 {
            bail!("Wrong number of elements at the response: {:?}", line);
        }

        let version = items[0].trim();
        let status_code = items[1].trim();
        let status = items[2..items.len()].join(" ");
        let status = status.trim();

        // Currently we only support HTTP 1.1
        match version {
            "HTTP/1.1" => {}
            _ => bail!("Unsupported HTTP version {}", version),
        }

        self.status = status.to_string();
        self.status_code = status_code.to_string();
        self.state = HTTPReadState::Headers;

        Ok(())
    }

    /// Reads the first line of the HTTP request.
    pub fn read_method(&mut self, line: &String) -> Result<()> {
        let (method, uri, version) =
            if let [method, uri, version] = line.split(' ').collect::<Vec<_>>().as_slice() {
                (method.trim(), uri.trim(), version.trim())
            } else {
                bail!("Wrong number of elements at the request: {:?}", line);
            };

        // Currently we only support HTTP 1.1
        match version {
            "HTTP/1.1" => {}
            _ => bail!("Unsupported HTTP version {}", version),
        }

        self.method = method.to_string();
        self.uri = uri.to_string();
        self.state = HTTPReadState::Headers;
        Ok(())
    }

    /// Reads the first line of the HTTP request/response and parses it.
    pub async fn read_info<T>(&mut self, buf: &mut ReadHalfBuf<T>) -> Result<()>
    where
        T: AsyncWrite + AsyncRead,
    {
        let mut line = String::new();
        if buf.read_line(&mut line).await? == 0 {
            bail!("Connection closed");
        }

        // It is a response
        if line.starts_with("HTTP") {
            self.read_response(&line)?;
        } else {
            self.read_method(&line)?;
        }

        Ok(())
    }

    /// Parses a header line and returns it.
    pub async fn read_header<T>(
        &mut self,
        buf: &mut ReadHalfBuf<T>,
    ) -> Result<Option<(String, String)>>
    where
        T: AsyncWrite + AsyncRead,
    {
        let mut line = String::new();
        if buf.read_line(&mut line).await? == 0 {
            return Ok(None);
        }

        if line == "\r\n" {
            return Ok(None);
        }

        let index = line.find(':').context("No ':' found in header")?;
        let (key, mut value) = line.split_at(index);
        let mut index = 0;

        for char in value.chars() {
            if char != ':' && !char.is_whitespace() {
                break;
            }
            index += 1;
        }

        if index != value.len() {
            value = &value[index..value.len() - 2];
        }

        let key = key.to_string();
        let value = value.to_string();

        Ok(Some((key, value)))
    }

    /// Reads all headers.
    pub async fn read_headers<T>(&mut self, buf: &mut ReadHalfBuf<T>) -> Result<()>
    where
        T: AsyncWrite + AsyncRead,
    {
        if self.state != HTTPReadState::Headers {
            bail!("Object not at the headers state.");
        }

        loop {
            let header = self.read_header(buf).await?;
            if header.is_none() {
                break;
            }

            let (key, value) = header.unwrap();
            self.headers.insert(key, value);
        }

        self.state = HTTPReadState::Body;
        Ok(())
    }

    /// Returns the value of HTTP header named `header_name`.
    /// Note: Use lowercase for matching.
    pub fn get_header(&self, header_name: &str) -> Option<&str> {
        for (key, value) in &self.headers {
            if key.to_lowercase() == header_name {
                return Some(value.as_str());
            }
        }

        None
    }

    /// Returns the mutable reference of HTTP header named `header_name`.
    /// Note: Use lowercase for matching.
    pub fn get_header_mul(&mut self, header_name: &str) -> Option<&mut String> {
        for (key, value) in &mut self.headers {
            if key.to_lowercase() == header_name {
                return Some(value);
            }
        }

        None
    }

    /// Reads the body of the HTTP request/response.
    pub async fn read_body<T>(&mut self, buf: &mut ReadHalfBuf<T>) -> Result<()>
    where
        T: AsyncWrite + AsyncRead,
    {
        if self.state != HTTPReadState::Body {
            bail!("Object not at the body state.");
        }

        // TODO: Add support for transport encoding chunks.
        let content_length = self.get_header("content-length");
        if content_length.is_some() {
            let content_length: usize = content_length.unwrap().parse()?;
            let mut read_byte_count: usize = 0;
            loop {
                let mut buffer = vec![0; content_length];
                let read = buf.read_exact(&mut buffer).await?;

                if read == 0 {
                    break;
                }

                self.body.extend(&buffer);
                read_byte_count += buffer.len();

                if read_byte_count >= content_length {
                    break;
                }
            }
        }

        self.headers.clear();
        self.method = String::new();
        self.uri = String::new();

        self.state = HTTPReadState::Info;

        Ok(())
    }
}
