use std::{
    rc::Rc, cell::RefCell, str::FromStr,
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
    let mut pt: sub1::PicoTts<u8> = sub1::PicoTts::new();
    let pt = pt.init();
    println!("main_6: pt(1) => {:?}", pt);

    let mut pt = sub1::PicoTts::new();
    let v = vec![1, 4, 5, 7, 9, 11];
    let pt = pt.update_sys(v.as_slice());
    println!("main_6: pt(2) => {:?}", pt);

    let mut pt = sub1::PicoTts::<u8>::from_path(
        std::ffi::OsStr::new("Cargo.toml")).unwrap();
    let pt = pt.init();
    println!("main_6: pt(3) => {:?}", pt);

    let mut pt = unsafe {
        sub1::PicoTts::<u8>::new_unsafe()
    };
    let v = vec![1, 1, 2, 3, 5, 8, 13, 21, 34, 55];
    let pt = pt.update_sys(v.as_slice());
    println!("main_6: pt(4) => {:?}", pt);
    let pt_sub = pt.get_sub(0);
    println!("main_6: pt_sub(4) => {:?}", pt_sub);

    let pt = sub1::PicoTts::new_with_data(&[1, 2, 3, 4, 5]);
    println!("main_6: pt(5) => {:?}", pt);
    let pt_sub = pt.get_sub(3);
    println!("main_6: pt_sub(5) => {:?}", pt_sub);

    let mut pt = sub1::PicoTts::new_with_data(&[1, 2, 3, 4, 5]);

    {
        let v: Vec<&[i32]> = vec![&[11, 12], &[22, 33], &[33, 44, 55]];
        pt.update_sub(v);
        println!("main_6: pt(6) => {:?}", pt);
    }

    println!("main_6: pt(6) => {:?}", pt);
    let pt_sub = pt.get_sub(3);
    println!("main_6: pt_sub(6) => {:?}", pt_sub);



}

// ---------------------------------------------------------------------------------
// 仮想的なPDFのデータ構造

#[derive(Debug)]
struct XRefTableEntry<'doc> {
    entry_type: i32,
    entry_data: &'doc [u8],
}

impl<'doc> XRefTableEntry<'doc> {
    fn new(entry_type: i32, data: &'doc [u8]) -> Self {
        XRefTableEntry { entry_type: 0, entry_data: data }
    }
}

#[derive(Debug)]
struct XRefTable<'doc> {
    doc: &'doc Doc<'doc>,
    table_data: Vec<XRefTableEntry<'doc>>
}

impl <'doc> XRefTable<'doc> {
    fn new(doc: &'doc Doc) -> Self {
        let mut xtbl = XRefTable {
            doc,
            table_data: Vec::new(),
        };
        xtbl.table_data.push(XRefTableEntry::new(1, doc.dat));
        xtbl
    }
}

#[derive(Debug)]
struct Doc<'a> {
    dat: &'a [u8],
    table: Option<XRefTable<'a>>,
}

impl <'a> Doc<'a> {
    fn new(dat: &'a [u8]) -> Self {
        let mut d = Doc {
            dat,
            table: None
        };
        let tbl = Some(XRefTable::new(unsafe {
            &*(&d as *const _)
        }));
        d.table = tbl;
        d
    }
}

fn main_7() {
    let v = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let d = Doc::new(v.as_slice());

    // docが再帰的にプリントされる事により以下は落ちる。
    //println!("main_7: {:?}", d);
}

// -------------------------------------------------------------------------------------------
// Typeパラメータ付き関数のテスト

struct Integer(i32);

impl AsRef<i32> for Integer {
    fn as_ref(&self) -> &i32 {
        &self.0
    }
}

fn fn_01<T>(cnt: T)
    where T: AsRef<i32>     // Tはtraitじゃないとだめ
{
    println!("cnt: => {}", cnt.as_ref());
}

fn fn_01_2(cnt: &dyn AsRef<i32>) {
    println!("cnt2: => {}", cnt.as_ref());
}

fn fn_01_3<T>(cnt: &T)
    where T: AsRef<i32>     // Tはtraitじゃないとだめ
{
    println!("cnt3: => {}", cnt.as_ref());
}

trait IBase {
    fn get_age(&self) -> i32;
    fn get_name(&self) -> &str;
    fn print(&self);
}

struct XYZ {
    age: i32,
    name: String,
    role: i32,
}

impl FromStr for XYZ {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v: Vec<&str> = s.split(' ').collect();

        // let else 秀逸かも
        let Some(age) = v.get(0) else { return Err("unexpected age".to_owned()) };
        let age = age.parse::<i32>().map_err(|v| format!("{:?}", v))?;
        let Some(name) = v.get(1) else { return Err("unexpected name".to_owned()) };
        let name = format!("{}", name);
        let Some(role) = v.get(2) else { return Err("unexpected role".to_owned()) };
        let role = role.parse::<i32>().map_err(|v| format!("{:?}", v))?;

        Ok(XYZ {
            age,
            name,
            role
        })
    }
}

impl IBase for XYZ {
    fn get_age(&self) -> i32 {
        self.age
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn print(&self) {
        println!("print from XYZ: {}", self.name);
    }
}

struct YSD<'a> {
    age: i32,
    display: String,
    speed: f32,
    hoge: &'a str,
}

impl YSD<'_> {
    fn new<'a, 'b>(age: i32, disp: String, speed: f32, hoge: &'a str) -> YSD<'b>
        where 'a: 'b    // 'bより'aの方が同じかより長生きするあのちゃん -> brrowチェッカーによってこの制約は担保され得る
    {
        YSD {
            age,
            display: disp,
            speed,
            hoge,
        }
    }

    fn print(&self) {
        print!("{{age: {}, display: {}, seppd: {}, hoge: {}}}",
            self.age, self.display, self.speed, self.hoge);
    }
}

impl IBase for YSD<'_> {
    fn get_age(&self) -> i32 {
        self.age
    }

    fn get_name(&self) -> &str {
        &self.display
    }

    fn print(&self) {
        self.print();
    }
}

fn fn_02<T>(disp: &[Box<T>])
    where T: IBase + ?Sized //+ 'static (<-'staticをつけると怒られる事が分かる/trait境界外や) // traitをよく扱う場合は、この記法の方が望ましいかも
{
    disp.into_iter().for_each(|v| {
        v.print();
        //println!("age: {}, name: {}", v.get_age(), v.get_name());
    });
}

// fn fn_03(v: &[Option<Box<dyn IBase>>])
fn fn_03<T>(v: &[Option<Box<T>>])   // この記法もできる: 違いはなんだろう？ -> where句にdynキーワードはつけられない
    where T: IBase + ?Sized         // dynなtraitの場合、?Sizedを付与しないとコンパイル時に怒られる。
{
    v.into_iter().for_each(|v| {
        if let Some(v) = v {
            v.print();
        } else {
            println!("none");
        }
    })
}

fn main_8() {
    // 以下は、fnの記法によって前者は静的ディスパッチ、後者は動的ディスパッチとなる
    fn_01(Integer(45));
    let int = Integer(32);
    fn_01_2(&int);
    // このfnの記法では借用でtraitを渡すが、dynすると怒られる。
    fn_01_3(&int);

    // 同じtraitを実装する奴らを同じく扱うためには参照もしくはBoxに突っ込むかのどっちかだ
    // →でも、予め具象型がわかっている場合は、traitではなくてenumにするな。動的ディスパッチにする必要ないもん。
    // 中身の所有権をVecに任せるならば、Boxに入れておく必要がある。
    let mut disp: Vec<Box<dyn IBase>> = vec![];
    disp.push(Box::new(YSD::new(12, "YSD_01".to_owned(), 12.0, "hoge01")));
    disp.push(Box::new(YSD::new(15, "YSD_02".to_owned(), 2.0, "hoge02")));
    disp.push(Box::new(XYZ { age: 93, name: "XYZ_01".to_owned(), role: 1032 }));
    fn_02(disp.as_slice());

    // Boxで包むことで、ポリモーフィズムなオブジェクトの配列は実現できる。
    let strstr = format!("XYZじゃなく123");
    let disp: [Box<dyn IBase>; 4] = [
        Box::new(YSD::new(112, "2:YSD_01".to_owned(), 12.0, &strstr)),
        Box::new(XYZ::from_str("33 HOGEHOE-XYZ-02 1021").unwrap()),
        Box::new(YSD::new(115, "2:YSD_02".to_owned(), 2.0, "123")),
        Box::new(XYZ { age: 31, name: "2:XYZ_01".to_owned(), role: 1032 }),
    ];
    fn_02(&disp);

    let mut disp: Vec<Option<Box<dyn IBase>>> = vec![];
    disp.push(Some(Box::new(YSD::new(12, "YSD_01".to_owned(), 12.0, "hoge01"))));
    let v: Option<Box<dyn IBase>> = if let Ok(v) = XYZ::from_str("33 HOGEHOE-XYZ-02 1021") {
        Some(Box::new(v))
    } else {
        None
    };
    disp.push(v);
    let v: Option<Box<dyn IBase>> = if let Ok(v) = XYZ::from_str("3.3 HOGEHOE-XYZ-03 748") {
        Some(Box::new(v))
    } else {
        None
    };
    disp.push(v);
    fn_03(disp.as_slice())

}

// -------------------------------------------------------------------------------------------


// -------------------------------------------------------------------------------------------
fn main() {
    main_1();
    main_2();
    main_3();
    main_4();
    main_5();
    main_6();
    main_7();
    main_8();
}