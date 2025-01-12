use libactionkv::ActionKV;

#[cfg(target_os = "windows")]
const USAGE: &str = "
USAGE: 
    akv_mem.exe FILE get KEY
    akv_mem.exe FILE delete KEY
    akv_mem.exe FILE insert KEY VALUE
    akv_mem.exe FILE update KEY VALUE
";

#[cfg(not(target_os = "windows"))]
const USAGE: &str = "
USAGE: 
    akv_mem FILE get KEY
    akv_mem FILE delete KEY
    akv_mem FILE insert KEY VALUE
    akv_mem FILE update KEY VALUE
";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let fname = args.get(1).expect(&USAGE);
    let action = args.get(2).expect(&USAGE).as_ref();
    let key = args.get(3).expect(&USAGE).as_ref();
    let maybe_value = args.get(4);

    let path = std::path::Path::new(fname);
    let mut store = ActionKV::open(path).expect(&format!("Unable to open {}", fname));
    store.load().expect("Unable to load data.");

    match action {
        "get" => match store.get(key).unwrap() {
            None => eprintln!(
                "key '{}' {:?} not found",
                std::str::from_utf8(key).unwrap(),
                key
            ),
            Some(value) => println!("({}) {:?}", std::str::from_utf8(&value).unwrap(), value),
        },
        "delete" => store.delete(key).unwrap(),
        "insert" => {
            let value = maybe_value.expect(&USAGE).as_ref();
            store.insert(key, &value).unwrap();
        }
        "update" => {
            let value = maybe_value.expect(&USAGE).as_ref();
            store.update(key, value).unwrap();
        }
        _ => eprintln!("{}", &USAGE),
    }
}
