use std::{
    rc::Rc, cell::RefCell,
};
use with_locals::with;

#[derive(Debug)]
struct Test<'a> {
    f1: &'a String
}

#[derive(Debug)]
struct Parent<'a> {
    name: String,
    test: &'a Test<'a>
}

impl<'a> Parent<'a> {
    fn new(test: &'a Test) -> Parent<'a> {
        Parent {
            name: "hoge".to_owned(),
            test: test
        }
    }
}

fn main_1() {
    let f1 = "f1f1".to_owned();
    let t = Test { f1: &f1 };
    let p = Parent::new(&t);

    println!("Hello, world! {:?}", p);
}

#[derive(Debug)]
struct SliceHolder<'a> {
    s: Option<&'a [u8]>,
}

impl<'a> SliceHolder<'a> {
    fn print_s(&self) {
        println!("my data: {:?}", self.s);
    }
}

#[derive(Debug)]
struct SliceArray<'a> {
    ss: Vec<SliceHolder<'a>>,
}

impl<'a> SliceArray<'a> {
    fn new(data: &'a [u8]) -> SliceArray<'a> {
        let mut sa = SliceArray { ss: vec![] };
        sa.ss.push(SliceHolder { s: data.get(..=3) });
        sa.ss.push(SliceHolder { s: data.get(2..=3) });
        sa.ss.push(SliceHolder { s: data.get(9..=13) });
        sa
    }
}

fn main_2() {
    // 外で全体のデータを保持するパターン
    let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    let sa = SliceArray::new(v.as_slice());
    // vとsaの生存期間が一致するのでOKOK。
    // 本音としては、SliceArrayにデータをすべて押し込みたい
    println!("slice array => {:?}", sa);
    sa.ss.into_iter().for_each(|v| v.print_s());
}


#[derive(Debug)]
struct SliceSelfArray<'a> {
    // Boxにする必要は必ずしもない？→
    dat: Box<Vec<u8>>,
    ss: Vec<SliceHolder<'a>>,
}

impl<'a> SliceSelfArray<'a> {
    fn new() -> Self {
        let mut sa = SliceSelfArray {
            dat: Box::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]),
            ss: vec![]
        };
        // 自己メンバー参照するケースはunsafeが必要となる？？
        // 参考：https://medium.com/@reduls/refers-other-field-in-the-same-struct-in-rust-777bb2075b8c
        // unsafeでポインタの参照をとることによって、ライフタイムを新たに設定することをコンパイラ(Borrow checker??)に伝えている
        let data: &'a [u8] = unsafe { &*(sa.dat.as_slice() as *const _) };

        // 関数化してもNG
        //let data: &'a [u8] = sa._self_data();
        sa.ss.push(SliceHolder { s: data.get(..=3) });
        sa.ss.push(SliceHolder { s: data.get(2..=3) });
        sa.ss.push(SliceHolder { s: data.get(9..=13) });
        sa
    }

    fn _self_data(&self) -> &[u8] {
        unsafe { &*(self.dat.as_slice() as *const _) }
    }
}

fn main_3() {
    let ssa = SliceSelfArray::new();
    println!("slice self array => {:?}", ssa);
    ssa.ss.into_iter().for_each(|v| v.print_s());
}

type RcRefCell<T> = std::rc::Rc<std::cell::RefCell<T>>;
type RcCell<T> = std::rc::Rc<std::cell::Cell<T>>;

#[derive(Debug)]
struct SliceRcArray<'a> {
    dat: RcRefCell<Vec<u8>>,
    ss: Vec<SliceHolder<'a>>,
}

impl<'a> SliceRcArray<'a> {
    fn new() -> SliceRcArray<'a> {
        let mut sra = SliceRcArray {
            dat: Rc::new(RefCell::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9])),
            ss: vec![],
        };
        // Rc<RefCell>で保持しても、結局はunsafeが必要になるぽい
        let data: &'a [u8] = unsafe { &*(sra.dat.borrow().as_slice() as *const _) };
        sra.ss.push(SliceHolder { s: data.get(..=3) });
        sra
    }
}

fn main_4() {
    let mut sra = SliceRcArray::new();
    println!("1: slice rc array => {:?}", sra);

    // 参照元を変更しちゃうと、SliceHolderが無効になっちゃう
    sra.dat.as_ref().borrow_mut().push(10);
    sra.dat.as_ref().borrow_mut().push(12);
    println!("2: slice rc array => {:?}", sra);
}

// 参考ページ: https://arunanshub.hashnode.dev/self-referential-structs-in-rust
// 自己参照構造体 (親と子で互いに参照を持ち合う的な話)

#[derive(Debug)]
struct Person<'me> {
    full_name: String,
    name: &'me str,
    surname: &'me str,
}

impl<'me> Person<'me> {
    fn new(full_name: String) -> Option<Self> {
        if full_name.split(" ").count() != 2 {
            None
        } else {
            Some(Self {
                full_name,
                name: "".into(),
                surname: "".into(),
            })
        }
    }

    fn new_and_init(full_name: String) -> Option<Self> {
        let p = Person::new(full_name);
        if p.is_none() {
            None
        } else {
            let p = p.unwrap();
            let p = p.init3();
            //let p: &'me Person = unsafe { &*(p.init() as *const _) };
            Some(p)
        }
    }

    fn init3(mut self) -> Person<'me> {
        let ref mut words = self.full_name.split(" ");
        // unsafeであれば、
        self.name = unsafe { &*(words.next().unwrap() as *const _) };
        self.surname = unsafe { &*(words.next().unwrap() as *const _) };
        self
    }

    // newと初期化を別々に呼び出す必要があるが、full_nameを書き換えるとボローチェッカーに怒られるようになるのでありっちゃーありか！？
    //  自己参照のシグニチャーを移動させる。
    fn init(self: &'me mut Person<'me>) -> &Self {
        let ref mut words = self.full_name.split(" ");
        self.name = words.next().unwrap();
        self.surname = words.next().unwrap();
        self
    }
}

#[with]
fn main_5() {
    let mut p = Person::new("Ryuta Hayashi".into()).unwrap();
    {
        let pp = p.init();
        println!("p => {:?}", pp);
    }
    // 以下のコードはコンパイルできない。
    // →init()でppのライフタイム = pのライフタイムとなって、ppがスコープを抜けて無効となっているため
    //p.full_name = "Haruka Hayashi".into();

    let mut p2 = (Person::new_and_init("Aiko Hayashi".into())).unwrap();
    // 以下で、nameとsurnameは無効化される。
    p2.full_name = "Fumika Hayashi".into();

    println!("p2 => {:?}", p2);
}

pub mod sub1;

fn main_6() {
    let mut pt = sub1::PicoTts::new();
    let pt = pt.init();
    println!("main_6: pt(1) => {:?}", pt);

    let mut pt = sub1::PicoTts::new();
    let v = vec![1, 4, 5, 7, 9, 11];
    let pt = pt.update_sys(v.as_slice());
    println!("main_6: pt(2) => {:?}", pt);

    let mut pt = sub1::PicoTts::from_path(
        std::ffi::OsStr::new("Cargo.toml")).unwrap();
    let pt = pt.init();
    println!("main_6: pt(3) => {:?}", pt);

    let mut pt = unsafe {
        sub1::PicoTts::new_unsafe()
    };
    let v = vec![1, 1, 2, 3, 5, 8, 13, 21, 34, 55];
    let pt = pt.update_sys(v.as_slice());
    println!("main_6: pt(4) => {:?}", pt);
    let pt_sub = pt.get_sub();
    println!("main_6: pt_sub(4) => {:?}", pt_sub);
}

fn main() {
    main_1();
    main_2();
    main_3();
    main_4();
    main_5();
    main_6();
}