use std::fmt;
use reqwest;
use reqwest::{Client, Response, StatusCode};
use sxd_document::writer::format_document;
use sxd_document::{parser, Package};
use sxd_document::dom::Document;
use sxd_xpath::evaluate_xpath;
use cookie::{Cookie, CookieJar};

#[derive(Debug)]
pub enum RpcError {
	ConnectionError(reqwest::Error),
	InvalidResponse
}

impl fmt::Display for RpcError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			&RpcError::InvalidResponse => write!(f, "The inwx api did not return a valid response"),
			&RpcError::ConnectionError(ref e) => write!(f, "Could not connect to the inwx api: {}", e)
		}
	}
}

pub struct RpcRequestParameter {
	pub name: &'static str,
	pub value: RpcRequestParameterValue,
}

pub enum RpcRequestParameterValue {
	String(String),
	Int(i32),
}

pub struct RpcRequest {
	body: Vec<u8>,
	method: String
}

impl RpcRequest {
	pub fn new(method: &str, parameters: &[RpcRequestParameter]) -> RpcRequest {
		let package = Package::new();
		let doc = package.as_document();

		let method_call = doc.create_element("methodCall");
		doc.root().append_child(method_call);

		let method_name = doc.create_element("methodName");
		method_name.append_child(doc.create_text(method));
		method_call.append_child(method_name);

		let params = doc.create_element("params");
		method_call.append_child(params);
		let param = doc.create_element("param");
		params.append_child(param);
		let value = doc.create_element("value");
		param.append_child(value);
		let s = doc.create_element("struct");
		value.append_child(s);

		for param in parameters {
			let member = doc.create_element("member");

			let name = doc.create_element("name");
			name.append_child(doc.create_text(&param.name));
			member.append_child(name);

			let value = doc.create_element("value");
			member.append_child(value);
			match param.value {
				RpcRequestParameterValue::String(ref val) => {
					let string = doc.create_element("string");
					string.append_child(doc.create_text(val));
					value.append_child(string);
				}
				RpcRequestParameterValue::Int(ref val) => {
					let string = doc.create_element("int");
					string.append_child(doc.create_text(&val.to_string()));
					value.append_child(string);
				}
			}

			s.append_child(member);
		}

		let mut body = Vec::new();
		format_document(&doc, &mut body).unwrap();

		RpcRequest {
			body: body,
			method: method.to_owned(),
		}
	}

	pub fn get_method(&self) -> String {
		self.method.clone()
	}

	pub fn send(self, url: &str, cookies: &mut CookieJar) -> Result<RpcResponse, RpcError> {
		let client = Client::new();

		let mut request = client
			.post(url)
			.body(self.body);

		let cookie_values: Vec<String> = cookies
			.iter()
			.map(|cookie| format!("{}", cookie.encoded()))
			.collect();

		if cookie_values.len() > 0 {
			let cookie_values = cookie_values.join(";");
			request = request.header(reqwest::header::COOKIE, cookie_values);
		}

		let response = request.send().map_err(|e| RpcError::ConnectionError(e))?;

		RpcResponse::new(response, cookies).ok_or(RpcError::InvalidResponse)
	}
}

pub struct RpcResponse {
	success: bool,
	package: Package,
}

impl RpcResponse {
	fn new(mut response: Response, cookies: &mut CookieJar) -> Option<RpcResponse> {
		if response.status() == StatusCode::OK {
			if let Ok(ref response_text) = response.text() {
				if let Ok(package) = parser::parse(response_text) {
					let mut success = false;

					for header in response.headers().get_all(reqwest::header::SET_COOKIE) {
						if let Ok(value) = header.to_str() {
							if let Ok(cookie) = Cookie::parse(value.to_owned()) {
								cookies.add(cookie);
							}
						}
					}

					if let Ok(code) = evaluate_xpath(&package.as_document(), "/methodResponse/params/param/value/struct/member[name/text()=\"code\"]/value/int/text()") {
						if let Ok(code) = code.string().parse::<i32>() {
							if code < 2000 {
								success = true;
							}
						}
					}

					return Some(RpcResponse {
						success,
						package,
					});
				}
			}
		}

		None
	}

	pub fn is_success(&self) -> bool {
		self.success
	}

	pub fn get_document(&self) -> Document {
		self.package.as_document()
	}
}
