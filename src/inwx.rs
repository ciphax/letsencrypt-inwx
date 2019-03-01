use std::fmt;
use cookie::CookieJar;
use sxd_xpath::{evaluate_xpath, Value};
use super::rpc::{RpcRequest, RpcResponse, RpcRequestParameter, RpcRequestParameterValue, RpcError};
use super::config::Account;

const API_URL: &str = "https://api.domrobot.com/xmlrpc/";
const OTE_API_URL: &str = "https://api.ote.domrobot.com/xmlrpc/";

#[derive(Debug)]
pub enum InwxError {
    RpcError(RpcError),
    DomainNotFound,
    RecordNotFound
}

impl fmt::Display for InwxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &InwxError::RpcError(ref e) => write!(f, "An inwx api call failed: {}", e),
            &InwxError::DomainNotFound => write!(f, "There is no nameserver for the specified domain"),
            &InwxError::RecordNotFound => write!(f, "The specified record does not exist")
        }
    }
}

impl From<RpcError> for InwxError {
    fn from(rpc_error: RpcError) -> InwxError {
        InwxError::RpcError(rpc_error)
    }
}

pub struct Inwx<'a> {
    cookies: CookieJar,
    account: &'a Account
}

impl<'a> Inwx<'a> {
    fn send_request(&mut self, request: RpcRequest) -> Result<RpcResponse, InwxError> {
        let url = match self.account.ote {
            true => OTE_API_URL,
            false => API_URL
        };
        let response = request.send(url, &mut self.cookies)?;

        Ok(response)
    }

    fn login(&mut self) -> Result<(), InwxError> {
        let request = RpcRequest::new("account.login", &[
            RpcRequestParameter {
                name: "user",
                value: RpcRequestParameterValue::String(self.account.username.to_owned())
            },
            RpcRequestParameter {
                name: "pass",
                value: RpcRequestParameterValue::String(self.account.password.to_owned())
            }
        ]);

        self.send_request(request)?;

        Ok(())
    }

    pub fn new(account: &'a Account) -> Result<Inwx<'a>, InwxError> {
        let mut api = Inwx {
            cookies: CookieJar::new(),
            account
        };

        api.login()?;

        Ok(api)
    }

    fn split_domain(&mut self, domain: &str) -> Result<(String, String), InwxError> {
        let page_size = 20;
        let mut page = 1;

        loop {
            let request = RpcRequest::new("nameserver.list", &[
                RpcRequestParameter {
                    name: "pagelimit",
                    value: RpcRequestParameterValue::Int(page_size)
                },
                RpcRequestParameter {
                    name: "page",
                    value: RpcRequestParameterValue::Int(page)
                }
            ]);

            let response = self.send_request(request)?;

            let total: i32 = evaluate_xpath(
                &response.get_document(),
                "/methodResponse/params/param/value/struct/member[name/text()=\"resData\"]/value/struct/member[name/text()=\"count\"]/value/int"
            )
                .ok()
                .and_then(|value| value.string().parse().ok())
                .ok_or_else(|| InwxError::DomainNotFound)?;

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

            if total > page * page_size {
                page += 1;
            } else {
                return Err(InwxError::DomainNotFound);
            }
        }
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

        id.ok_or_else(|| InwxError::RecordNotFound)
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
