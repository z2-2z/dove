
pub fn error_header<S: AsRef<str>>(msg: S) {
    eprintln!("---==== {} ====---", msg.as_ref());
}

pub fn error_footer<S: AsRef<str>>(msg: S) {
    eprintln!("---====={1:=<0$}=====---", msg.as_ref().len() , "");
}
