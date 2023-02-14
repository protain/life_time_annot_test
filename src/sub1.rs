use std::io::Read;

#[derive(Debug)]
pub struct PicoTts<'a> {
    sys: Vec<u8>,
    sub: Vec<&'a [u8]>,
}

impl<'a> PicoTts<'a> {

    pub fn new() -> Self {
        let x = PicoTts {
            sys: vec![1, 2, 3, 4, 5],
            sub: Vec::new(),
        };
        x
    }

    pub unsafe fn new_unsafe() -> Self {
        let mut x = PicoTts {
            sys: vec![1, 2, 3, 4, 5],
            sub: Vec::new(),
        };
        for i in 0..(x.sys.len() - 1) {
            // unsafe below.
            x.sub.push(&*(x.sys[i..=i+1].as_ref() as *const _));
        }
        x
    }

    pub fn from_path(path: &std::ffi::OsStr) -> std::io::Result<Self> {
        let mut v = std::fs::File::open(path)?;
        let mut buf = Vec::new();
        let _ = v.read_to_end(&mut buf)?;
        Ok(PicoTts {
            sys: buf,
            sub: Vec::new(),
        })
    }

    pub fn init(self: &'a mut PicoTts<'a>) -> &Self {

        for i in 0..(self.sys.len() - 1) {
            let v = self.sys[i..=i+1].as_ref();
            self.sub.push(v);
        }
        self
    }

    pub fn get_sub(self: &'a PicoTts<'a>) -> Option<&[u8]> {
        if self.sub.len() == 0 {
            None
        } else {
            Some(self.sub[0])
        }
    }

    pub fn update_sys(self: &'a mut PicoTts<'a>, dat: &[u8]) -> &Self {
        self.sys = Vec::from(dat);
        self.sub.clear();
        self.init()
    }

}