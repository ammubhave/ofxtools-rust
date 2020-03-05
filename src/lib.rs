// pub struct OFXClientInitStruct<'a, 'b> {
//     // userid: Optional[str] = None,
//     //     clientuid: Optional[str] = None,
//     org: Option<&'a str>,
//     fid: Option<&'b str>,
//     //     version: Optional[int] = None,
//     //     appid: Optional[str] = None,
//     //     appver: Optional[str] = None,
//     //     language: Optional[str] = None,
//     //     prettyprint: Optional[bool] = None,
//     //     close_elements: Optional[bool] = None,
//     //     bankid: Optional[str] = None,
//     //     brokerid: Optional[str] = None
// }

pub struct OFXClient {
    url: String,
    org: Option<String>,
    fid: Option<String>,
}

impl OFXClient {
    pub fn new(url: String, org: Option<String>, fid: Option<String>) -> Self {
        OFXClient { url, org, fid }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
