use derive_more::derive::Display;

#[allow(non_camel_case_types)]
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
    //401,
    //402,
    //403,
    Not_Found = 404,
    Method_Not_Allowed = 405,
    //406,
    //407,
    //408,
    //409,
    //410,
    //411,
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

    //500,
    //501,
    //502,
    //503,
    //504,
    //505,
    //506,
    //507,
    //508,
    //510,
    //511,
}

impl ResponseCode {
    pub fn pretty_string(self) -> String {
        format!("{} {}", self as i32, self.to_string().replace('_', " "))
    }
}
