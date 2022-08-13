pub struct ResponseInfo {
    pub status: u16,
    pub time: u128
}

impl ResponseInfo {

    pub fn new(status: u16, time: u128) -> ResponseInfo {
        ResponseInfo { status, time }
    }

}