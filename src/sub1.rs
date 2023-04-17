use std::{io::Read};
use anyhow::{Context, Result};

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

    pub fn from_path(path: &std::ffi::OsStr) -> Result<PicoTts<'_, u8>> {
        let mut v = std::fs::File::open(path)?;
        let mut buf = vec![];
        let _ = v.read_to_end(&mut buf).context("unexpect read to file.")?;
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

#[cfg(test)]
mod tests {
    use std::vec;
    use crate::sub1::*;

    #[test]
    // 目的：PicoTts::new()関数が正しく動作するかを確認する
    fn test_pico_tts_new() {
        let pico_tts: PicoTts<u8> = PicoTts::new();
        assert_eq!(pico_tts.sys, vec![]);
        assert_eq!(pico_tts.sub, Vec::<&[u8]>::new());
        assert_eq!(pico_tts.subsub, None);
    }
    
    #[test]
    // 目的：PicoTts::new_unsafe()関数が正しく動作するかを確認する
    fn test_pico_tts_new_unsafe() {
        let pico_tts = unsafe { PicoTts::<'_, u8>::new_unsafe() };
        assert_eq!(pico_tts.sys, vec![1, 2, 3, 4, 5]);
        assert_eq!(pico_tts.sub.len(), 4);
        assert_eq!(pico_tts.subsub, None);
    }
    
    #[test]
    // 目的：PicoTts::new_with_data()関数が正しく動作するかを確認する
    fn test_pico_tts_new_with_data() {
        let data = vec![1, 2, 3, 4, 5];
        let pico_tts = PicoTts::new_with_data(&data);
        assert_eq!(pico_tts.sys, vec![1, 2, 3, 4, 5]);
        assert_eq!(pico_tts.sub.len(), 4);
        assert_eq!(pico_tts.subsub.unwrap().len(), 4);
    }
    
    #[test]
    // 目的：PicoTts::from_path()関数が正しく動作するかを確認する
    fn test_pico_tts_from_path() {
        let path = std::ffi::OsStr::new("Cargo.toml");
        let pico_tts = PicoTts::<'_, u8>::from_path(&path).unwrap();
        assert_eq!(pico_tts.sys.len(), 226);
        assert_eq!(pico_tts.sub.len(), 0);
        assert_eq!(pico_tts.subsub, None);
    }
    
    #[test]
    // 目的：PicoTts::init()関数が正しく動作するかを確認する
    fn test_pico_tts_init() {
        let mut pico_tts = PicoTts::new_with_data(&vec![1, 2, 3, 4, 5]);
        let pico_tts = pico_tts.init();
        assert_eq!(pico_tts.sys, vec![1, 2, 3, 4, 5]);
        assert_eq!(pico_tts.sub.len(), 8);
        assert_eq!(pico_tts.subsub.as_ref().unwrap().len(), 8);
    }
    
    #[test]
    // 目的：PicoTts::get_sub()関数が正しく動作するかを確認する
    fn test_pico_tts_get_sub() {
        let pico_tts = PicoTts::new_with_data(&vec![1, 2, 3, 4, 5]);
        let sub = pico_tts.get_sub(2);
        let test: &[i32] = &[3, 4];
        assert_eq!(sub, Some(test));
    }
    
    #[test]
    // 目的：PicoTts::update_sub()関数が正しく動作するかを確認する
    fn test_pico_tts_update_sub() {
        let mut pico_tts = PicoTts::new_with_data(&vec![1, 2, 3, 4, 5]);
        let dat: Vec<&[i32]> = vec![&[1, 2], &[3, 4], &[5, 6]];
        pico_tts.update_sub(dat);
        let test: Vec<&[i32]> = vec![&[1, 2], &[3, 4], &[5, 6]];
        assert_eq!(pico_tts.subsub.as_ref().unwrap(), &test);
    }
    
    #[test]
    // 目的：PicoTts::update_sys()関数が正しく動作するかを確認する
    fn test_pico_tts_update_sys() {
        let mut pico_tts = PicoTts::new_with_data(&vec![1, 2, 3, 4, 5]);
        let data = vec![6, 7, 8, 9, 10];
        let pico_tts = pico_tts.update_sys(&data);
        assert_eq!(pico_tts.sys, vec![6, 7, 8, 9, 10]);
        assert_eq!(pico_tts.sub.len(), 4);
        assert_eq!(pico_tts.subsub.as_ref().unwrap().len(), 4);
    }
}