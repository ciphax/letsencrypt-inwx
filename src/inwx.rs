use reqwest::header::Cookie;
use sxd_xpath::{evaluate_xpath, Value};
use super::rpc::{RpcRequest, RpcResponse, RpcRequestParameter, RpcRequestParameterValue};
use std::fmt;

const API_URL: &str = "https://api.domrobot.com/xmlrpc/";

#[derive(Debug)]
pub enum InwxError {
    ConnectError,
    ApiError {
        method: String,
        msg: String
    }
}

impl fmt::Display for InwxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &InwxError::ConnectError => write!(f, "Could not connect to the inwx api"),
            &InwxError::ApiError { ref method, ref msg } => write!(f, "{}: {}", method, msg)
        }
    }
}

pub struct Inwx {
    cookie: Cookie
}

impl Inwx {
    fn send_request(request: RpcRequest) -> Result<RpcResponse, InwxError> {
        let method = request.get_method();

        if let Some(response) = request.send(API_URL) {
            if response.is_success() {
                Ok(response)
            } else {
                let msg = match evaluate_xpath(&response.get_document(), "/methodResponse/params/param/value/struct/member[name/text()=\"msg\"]/value/string/text()") {
                    Ok(ref value) => value.string(),
                    Err(_) => String::new()
                };

                Err(InwxError::ApiError {
                    msg,
                    method
                })
            }
        } else {
            Err(InwxError::ConnectError)
        }
    }

    fn login(user: &str, pass: &str) -> Result<Cookie, InwxError> {
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

        let response = Inwx::send_request(request)?;

        Ok(response.get_cookie())
    }

    pub fn new(user: &str, pass: &str) -> Result<Inwx, InwxError> {
        let cookie = Inwx::login(user, pass)?;

        Ok(Inwx {
            cookie
        })
    }

    fn split_domain(&self, domain: &str) -> Result<(String, String), InwxError> {
        let mut request = RpcRequest::new("nameserver.list", &[
            RpcRequestParameter {
                name: "pagelimit",
                value: RpcRequestParameterValue::Int(1000)
            }
        ]);
        request.set_cookie(self.cookie.clone());

        let response = Inwx::send_request(request)?;

        if let Ok(Value::Nodeset(ref nodes)) = evaluate_xpath(&response.get_document(), "/methodResponse/params/param/value/struct/member[name/text()=\"resData\"]/value/struct/member[name/text()=\"domains\"]/value/array/data/value/struct/member[name/text()=\"domain\"]/value/string/text()") {
            for node in nodes {
                if let Some(ref text) = node.text() {
                    let domain_root = text.text();

                    if domain.ends_with(domain_root) {
                        let mut name = &domain[0..domain.len() - domain_root.len()];

                        if name.ends_with(".") {
                            name = &name[0..name.len() - 1];
                        }

                        return Ok((domain_root.to_owned(), name.to_owned()));
                    }
                }
            }
        }
        
        Err(InwxError::ApiError {
            method: "nameserver.list".to_owned(),
            msg: "Domain not found".to_owned()
        })
    }

    pub fn create_txt_record(&self, domain: &str, content: &str) -> Result<(), InwxError> {
        let (domain, name) = self.split_domain(domain)?;
        
        let mut request = RpcRequest::new("nameserver.createRecord", &[
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
            },
            RpcRequestParameter {
                name: "ttl",
                value: RpcRequestParameterValue::Int(300)
            }
        ]);
        request.set_cookie(self.cookie.clone());

        Inwx::send_request(request)?;

        Ok(())
    }

    pub fn get_record_id(&self, domain: &str) -> Result<i32, InwxError> {
        let (domain, name) = self.split_domain(domain)?;
        
        let mut request = RpcRequest::new("nameserver.info", &[
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
        request.set_cookie(self.cookie.clone());

        let response = Inwx::send_request(request)?;

        let id = match evaluate_xpath(&response.get_document(), "/methodResponse/params/param/value/struct/member[name/text()=\"resData\"]/value/struct/member[name/text()=\"record\"]/value/array/data/value[1]/struct/member[name/text()=\"id\"]/value/int/text()") {
            Ok(ref id) => id.string().parse::<i32>().ok(),
            Err(_) => None
        };
        
        id.ok_or(InwxError::ApiError {
            method: "nameserver.info".to_owned(),
            msg: "Record not found".to_owned()
        })
    }

    pub fn delete_txt_record(&self, domain: &str) -> Result<(), InwxError> {
        let id = self.get_record_id(domain)?;

        let mut request = RpcRequest::new("nameserver.deleteRecord", &[
            RpcRequestParameter {
                name: "id",
                value: RpcRequestParameterValue::Int(id)
            }
        ]);
        request.set_cookie(self.cookie.clone());

        Inwx::send_request(request)?;

        Ok(())
    }

    pub fn logout(self) -> Result<(), InwxError> {
        let mut request = RpcRequest::new("account.logout", &[]);
        request.set_cookie(self.cookie);

        Inwx::send_request(request)?;

        Ok(())
    }
}