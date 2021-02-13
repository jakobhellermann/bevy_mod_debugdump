pub fn short_name(type_name: &str) -> String {
    match type_name.find('<') {
        // no generics
        None => type_name.rsplit("::").next().unwrap_or(type_name).into(),
        // generics a::b::c<d>
        Some(angle_open) => {
            let angle_close = type_name.rfind('>').unwrap();

            let before_generics = &type_name[..angle_open];
            let after = &type_name[angle_close + 1..];
            let in_between = &type_name[angle_open + 1..angle_close];

            let before_generics = match before_generics.rfind("::") {
                None => before_generics,
                Some(i) => &before_generics[i + 2..],
            };

            let in_between = short_name(in_between);

            format!("{}<{}>{}", before_generics, in_between, after)
        }
    }
}
