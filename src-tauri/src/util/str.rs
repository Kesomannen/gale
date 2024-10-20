pub trait TrimInPlace {
    fn trim_in_place(self: &mut Self) -> &str;
}

impl TrimInPlace for String {
    fn trim_in_place(self: &mut Self) -> &str {
        let trim = self.trim();
        let (trim_ptr, trim_len) = (trim.as_ptr(), trim.len());

        if trim_len != self.len() {
            unsafe {
                let vec = self.as_mut_vec();

                std::intrinsics::copy(trim_ptr, vec.as_mut_ptr(), trim_len);

                vec.set_len(trim_len);
            }
        }

        self.as_str()
    }
}
