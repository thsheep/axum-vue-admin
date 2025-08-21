use askama::Template;

#[derive(Template)]
#[template(path = "cn/reset_password.html")]
pub struct CNPasswordResetTemplate<'a> {
    pub reset_url: &'a str,
}


#[derive(Template)]
#[template(path = "en/reset_password.html")]
pub struct ENPasswordResetTemplate<'a> {
    pub reset_url: &'a str,
}