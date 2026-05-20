fn main() {
    println!("Placeholder для экспериментов с cli");

    let parsing_demo = r#"[UserBackets{"user_id":"Bob","backets":[Backet{"asset_id":"milk","count":3,},],},]"#.to_string();
    // just_parse_anouncements теперь возвращает (&str, T), берём .1
    let announcements = analysis::parse::just_parse_anouncements(&parsing_demo).unwrap().1;
    println!("demo-parsed: {:?}", announcements);

    let args = std::env::args().collect::<Vec<_>>();
    let filename = &args[1];
    println!("Trying opening file '{}' from directory '{}'", filename, std::env::current_dir().unwrap().to_string_lossy());
    
    // Прямая передача файла — никаких Rc, RefCell, clone
    let file = std::fs::File::open(filename).unwrap();
    let logs = analysis::read_log(file, analysis::READ_MODE_ALL, vec![]);
    
    println!("got logs:");
    logs.iter().for_each(|parsed| println!("  {:?}", parsed));
}