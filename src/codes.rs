use derive_more::derive::Display;

// We allow dead code here because these still need to be here even if they're not currently used
#[allow(non_camel_case_types, dead_code)]
#[derive(Display, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResponseCode {
    //100,
    //101,
    //102,
    //103,
    
    Ok = 200,
    //201,
    //202,
    //203,
    //204,
    //205,
    //206,
    //207,
    //226,

    //300,
    //301,
    //302,
    //303,
    //304,
    //305,
    //306,
    //307,
    //308,

    Bad_Request = 400,
    Unauthorized = 401,
    Payment_Required = 402,
    Forbidden = 403,
    Not_Found = 404,
    Method_Not_Allowed = 405,
    Not_Acceptable = 406,
    Proxy_Authentication_Required = 407,
    Request_Timeout = 408,
    Conflict = 409,
    Gone = 410,
    Length_Required = 411,
    //412,
    //413,
    //414,
    //415,
    //416,
    //417,
    //418,
    //421,
    //422,
    //423,
    //424,
    //425,
    //426,
    //428,
    //429,
    //431,
    //451,

    Internal_Server_Error = 500,
    Not_Implemented = 501,
    Bad_Gateway = 502,
    Service_Unavailable = 503,
    Gateway_Timeout = 504,
    HTTP_Version_Not_Supported = 505,
    Variant_Also_Negotiates = 506,
    Insufficient_Storage = 507,
    Loop_Detected = 508,
    Not_Extended = 510,
    Network_Authentication_Required = 511,
}

impl ResponseCode {
    pub fn pretty_string(self) -> String {
        format!("{} {}", self as i32, self.to_string().replace('_', " "))
    }
}
