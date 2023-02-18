use std::{io::Read};

struct PicoEntry<'a, T> {
    dat: Vec<&'a [T]>,
}

#[derive(Debug)]
pub struct PicoTts<'a, T: 'static> {
    sys: Vec<T>,
    sub: Vec<&'a [T]>,
    subsub: Option<Vec<&'a [T]>>,
}

impl<'a, T> PicoTts<'a, T>
    where T: 'static + Clone {

    pub fn new() -> Self {
        let x = PicoTts {
            sys: vec![],
            sub: Vec::new(),
            subsub: None,
        };
        x
    }

    // sysを可変にさせなければ安全性は担保されるので、このモジュール内で生合成を担保すればよく、
    // 外部モジュールからは、敢えてこの関数をunsafeにする必要はないか・・・・
    pub unsafe fn new_unsafe() -> PicoTts<'a, u8> {
        let mut x = PicoTts {
            sys: vec![1, 2, 3, 4, 5],
            sub: Vec::new(),
            subsub: None,
        };
        for i in 0..(x.sys.len() - 1) {
            // unsafe below.
            // main.rsでやってるのと同様、anonymousなポインタを介す事でライフタイムを再設定する
            x.sub.push(&*(x.sys[i..=i+1].as_ref() as *const _));
        }
        x
    }

    pub fn new_with_data(dat: &[T]) -> Self {
        let mut x = PicoTts {
            sys: Vec::from(dat),
            sub: Vec::new(),
            subsub: None,
        };
        for i in 0..(x.sys.len() - 1) {
            // unsafe below.
            // main.rsでやってるのと同様、anonymousなポインタを介す事でライフタイムを再設定する
            x.sub.push(unsafe { &*(x.sys[i..=i+1].as_ref() as *const _) });
        }
        x.subsub = Some(Vec::from(unsafe { &*(x.sub.as_ref() as *const _) }));
        x
    }

    pub fn from_path<'b>(path: &std::ffi::OsStr) -> std::io::Result<PicoTts<'b, u8>> {
        let mut v = std::fs::File::open(path)?;
        let mut buf = vec![];
        let _ = v.read_to_end(&mut buf)?;
        Ok(PicoTts {
            sys: buf,
            sub: Vec::new(),
            subsub: None,
        })
    }

    pub fn init(self: &'a mut PicoTts<'a, T>) -> &Self {

        if self.sys.len() > 0 {
            for i in 0..(self.sys.len() - 1) {
                let v = self.sys[i..=i+1].as_ref();
                self.sub.push(v);
            }
            self.subsub = Some(Vec::from(unsafe { &*(self.sub.as_ref() as *const _) }));
        }
        self
    }

    pub fn get_sub(self: &'a PicoTts<'a, T>, idx: usize) -> Option<&'a [T]> {
        if self.sub.len() == 0 {
            None
        } else {
            match self.sub.get(idx) {
                Some(v) => Some(*v),
                None => None
            }
        }
    }

    pub fn update_sub(&mut self, dat: Vec<&'a [T]>) -> &mut Self {
        self.subsub = Some(dat);
        self
    }

    pub fn update_sys(self: &'a mut PicoTts<'a, T>, dat: &[T]) -> &Self {
        self.sys = Vec::from(dat);
        self.sub.clear();
        self.subsub = None;
        self.init()
    }

}