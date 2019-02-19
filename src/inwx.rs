use std::fmt;
use cookie::CookieJar;
use sxd_xpath::{evaluate_xpath, Value};
use super::rpc::{RpcRequest, RpcResponse, RpcRequestParameter, RpcRequestParameterValue, RpcError};

const API_URL: &str = "https://api.domrobot.com/xmlrpc/";

#[derive(Debug)]
pub enum InwxError {
	RpcError(RpcError),
	ApiError {
		method: String,
		reason: String,
		msg: String
	}
}

impl fmt::Display for InwxError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			&InwxError::RpcError(ref e) => write!(f, "The inwx api call failed: {}", e),
			&InwxError::ApiError { ref method, ref msg, ref reason } => write!(f, "method={},msg={},reason={}", method, msg, reason)
		}
	}
}

pub struct Inwx {
	cookies: CookieJar
}

impl Inwx {
	fn send_request(&mut self, request: RpcRequest) -> Result<RpcResponse, InwxError> {
		let method = request.get_method();

		let response = request.send(API_URL, &mut self.cookies).map_err(|e| InwxError::RpcError(e))?;

		if response.is_success() {
			Ok(response)
		} else {
			let msg = match evaluate_xpath(
				&response.get_document(),
				"/methodResponse/params/param/value/struct/member[name/text()=\"msg\"]/value/string/text()"
			) {
				Ok(ref value) => value.string(),
				Err(_) => String::new()
			};

			let reason = match evaluate_xpath(
				&response.get_document(),
				"/methodResponse/params/param/value/struct/member[name/text()=\"reason\"]/value/string/text()"
			) {
				Ok(ref value) => value.string(),
				Err(_) => String::new()
			};

			Err(InwxError::ApiError {
				msg,
				reason,
				method
			})
		}
	}

	fn login(&mut self, user: &str, pass: &str) -> Result<(), InwxError> {
		let request = RpcRequest::new("account.login", &[
			RpcRequestParameter {
				name: "user",
				value: RpcRequestParameterValue::String(user.to_owned())
			},
			RpcRequestParameter {
				name: "pass",
				value: RpcRequestParameterValue::String(pass.to_owned())
			}
		]);

		self.send_request(request)?;

		Ok(())
	}

	pub fn new(user: &str, pass: &str) -> Result<Inwx, InwxError> {
		let mut api = Inwx {
			cookies: CookieJar::new()
		};

		api.login(user, pass)?;

		Ok(api)
	}

	fn split_domain(&mut self, domain: &str) -> Result<(String, String), InwxError> {
		let request = RpcRequest::new("nameserver.list", &[]);

		let response = self.send_request(request)?;

		if let Ok(Value::Nodeset(ref nodes)) = evaluate_xpath(&response.get_document(), "/methodResponse/params/param/value/struct/member[name/text()=\"resData\"]/value/struct/member[name/text()=\"domains\"]/value/array/data/value/struct/member[name/text()=\"domain\"]/value/string/text()") {
			for node in nodes {
				if let Some(ref text) = node.text() {
					let domain_root = text.text();

					if domain.ends_with(&format!(".{}", domain_root)) {
						let mut name = &domain[0..domain.len() - domain_root.len() - 1];

						return Ok((domain_root.to_owned(), name.to_owned()));
					} else if domain == domain_root {
						return Ok((domain_root.to_owned(), "".to_owned()));
					}
				}
			}
		}

		Err(InwxError::ApiError {
			method: "nameserver.list".to_owned(),
			msg: "Domain not found".to_owned(),
			reason: "".to_owned()
		})
	}

	pub fn create_txt_record(&mut self, domain: &str, content: &str) -> Result<(), InwxError> {
		let (domain, name) = self.split_domain(domain)?;

		let request = RpcRequest::new("nameserver.createRecord", &[
			RpcRequestParameter {
				name: "type",
				value: RpcRequestParameterValue::String("TXT".to_owned())
			},
			RpcRequestParameter {
				name: "name",
				value: RpcRequestParameterValue::String(name)
			},
			RpcRequestParameter {
				name: "content",
				value: RpcRequestParameterValue::String(content.to_owned())
			},
			RpcRequestParameter {
				name: "domain",
				value: RpcRequestParameterValue::String(domain)
			}
		]);

		self.send_request(request)?;

		Ok(())
	}

	pub fn get_record_id(&mut self, domain: &str) -> Result<i32, InwxError> {
		let (domain, name) = self.split_domain(domain)?;

		let request = RpcRequest::new("nameserver.info", &[
			RpcRequestParameter {
				name: "type",
				value: RpcRequestParameterValue::String("TXT".to_owned())
			},
			RpcRequestParameter {
				name: "name",
				value: RpcRequestParameterValue::String(name.to_owned())
			},
			RpcRequestParameter {
				name: "domain",
				value: RpcRequestParameterValue::String(domain.to_owned())
			}
		]);

		let response = self.send_request(request)?;

		let id = match evaluate_xpath(&response.get_document(), "/methodResponse/params/param/value/struct/member[name/text()=\"resData\"]/value/struct/member[name/text()=\"record\"]/value/array/data/value[1]/struct/member[name/text()=\"id\"]/value/int/text()") {
			Ok(ref id) => id.string().parse::<i32>().ok(),
			Err(_) => None
		};

		id.ok_or(InwxError::ApiError {
			method: "nameserver.info".to_owned(),
			msg: "Record not found".to_owned(),
			reason: "".to_owned()
		})
	}

	pub fn delete_txt_record(&mut self, domain: &str) -> Result<(), InwxError> {
		let id = self.get_record_id(domain)?;

		let request = RpcRequest::new("nameserver.deleteRecord", &[
			RpcRequestParameter {
				name: "id",
				value: RpcRequestParameterValue::Int(id)
			}
		]);

		self.send_request(request)?;

		Ok(())
	}

	pub fn logout(mut self) -> Result<(), InwxError> {
		let request = RpcRequest::new("account.logout", &[]);

		self.send_request(request)?;

		Ok(())
	}
}
