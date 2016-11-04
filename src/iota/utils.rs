use std::collections::HashMap;

use regex::Regex;

lazy_static! {
    static ref MAPPINGS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // Primary 3-bit (8 colors). Unique representation!
        m.insert("000000", "00");
        m.insert("800000", "01");
        m.insert("008000", "02");
        m.insert("808000", "03");
        m.insert("000080", "04");
        m.insert("800080", "05");
        m.insert("008080", "06");
        m.insert("c0c0c0", "07");

        // Equivalent "bright" versions of original 8 colors.
        m.insert("808080", "08");
        m.insert("ff0000", "09");
        m.insert("00ff00", "10");
        m.insert("ffff00", "11");
        m.insert("0000ff", "12");
        m.insert("ff00ff", "13");
        m.insert("00ffff", "14");
        m.insert("ffffff", "15");

        // Strictly ascending.
        m.insert("000000", "16");
        m.insert("00005f", "17");
        m.insert("000087", "18");
        m.insert("0000af", "19");
        m.insert("0000d7", "20");
        m.insert("0000ff", "21");
        m.insert("005f00", "22");
        m.insert("005f5f", "23");
        m.insert("005f87", "24");
        m.insert("005faf", "25");
        m.insert("005fd7", "26");
        m.insert("005fff", "27");
        m.insert("008700", "28");
        m.insert("00875f", "29");
        m.insert("008787", "30");
        m.insert("0087af", "31");
        m.insert("0087d7", "32");
        m.insert("0087ff", "33");
        m.insert("00af00", "34");
        m.insert("00af5f", "35");
        m.insert("00af87", "36");
        m.insert("00afaf", "37");
        m.insert("00afd7", "38");
        m.insert("00afff", "39");
        m.insert("00d700", "40");
        m.insert("00d75f", "41");
        m.insert("00d787", "42");
        m.insert("00d7af", "43");
        m.insert("00d7d7", "44");
        m.insert("00d7ff", "45");
        m.insert("00ff00", "46");
        m.insert("00ff5f", "47");
        m.insert("00ff87", "48");
        m.insert("00ffaf", "49");
        m.insert("00ffd7", "50");
        m.insert("00ffff", "51");
        m.insert("5f0000", "52");
        m.insert("5f005f", "53");
        m.insert("5f0087", "54");
        m.insert("5f00af", "55");
        m.insert("5f00d7", "56");
        m.insert("5f00ff", "57");
        m.insert("5f5f00", "58");
        m.insert("5f5f5f", "59");
        m.insert("5f5f87", "60");
        m.insert("5f5faf", "61");
        m.insert("5f5fd7", "62");
        m.insert("5f5fff", "63");
        m.insert("5f8700", "64");
        m.insert("5f875f", "65");
        m.insert("5f8787", "66");
        m.insert("5f87af", "67");
        m.insert("5f87d7", "68");
        m.insert("5f87ff", "69");
        m.insert("5faf00", "70");
        m.insert("5faf5f", "71");
        m.insert("5faf87", "72");
        m.insert("5fafaf", "73");
        m.insert("5fafd7", "74");
        m.insert("5fafff", "75");
        m.insert("5fd700", "76");
        m.insert("5fd75f", "77");
        m.insert("5fd787", "78");
        m.insert("5fd7af", "79");
        m.insert("5fd7d7", "80");
        m.insert("5fd7ff", "81");
        m.insert("5fff00", "82");
        m.insert("5fff5f", "83");
        m.insert("5fff87", "84");
        m.insert("5fffaf", "85");
        m.insert("5fffd7", "86");
        m.insert("5fffff", "87");
        m.insert("870000", "88");
        m.insert("87005f", "89");
        m.insert("870087", "90");
        m.insert("8700af", "91");
        m.insert("8700d7", "92");
        m.insert("8700ff", "93");
        m.insert("875f00", "94");
        m.insert("875f5f", "95");
        m.insert("875f87", "96");
        m.insert("875faf", "97");
        m.insert("875fd7", "98");
        m.insert("875fff", "99");
        m.insert("878700", "100");
        m.insert("87875f", "101");
        m.insert("878787", "102");
        m.insert("8787af", "103");
        m.insert("8787d7", "104");
        m.insert("8787ff", "105");
        m.insert("87af00", "106");
        m.insert("87af5f", "107");
        m.insert("87af87", "108");
        m.insert("87afaf", "109");
        m.insert("87afd7", "110");
        m.insert("87afff", "111");
        m.insert("87d700", "112");
        m.insert("87d75f", "113");
        m.insert("87d787", "114");
        m.insert("87d7af", "115");
        m.insert("87d7d7", "116");
        m.insert("87d7ff", "117");
        m.insert("87ff00", "118");
        m.insert("87ff5f", "119");
        m.insert("87ff87", "120");
        m.insert("87ffaf", "121");
        m.insert("87ffd7", "122");
        m.insert("87ffff", "123");
        m.insert("af0000", "124");
        m.insert("af005f", "125");
        m.insert("af0087", "126");
        m.insert("af00af", "127");
        m.insert("af00d7", "128");
        m.insert("af00ff", "129");
        m.insert("af5f00", "130");
        m.insert("af5f5f", "131");
        m.insert("af5f87", "132");
        m.insert("af5faf", "133");
        m.insert("af5fd7", "134");
        m.insert("af5fff", "135");
        m.insert("af8700", "136");
        m.insert("af875f", "137");
        m.insert("af8787", "138");
        m.insert("af87af", "139");
        m.insert("af87d7", "140");
        m.insert("af87ff", "141");
        m.insert("afaf00", "142");
        m.insert("afaf5f", "143");
        m.insert("afaf87", "144");
        m.insert("afafaf", "145");
        m.insert("afafd7", "146");
        m.insert("afafff", "147");
        m.insert("afd700", "148");
        m.insert("afd75f", "149");
        m.insert("afd787", "150");
        m.insert("afd7af", "151");
        m.insert("afd7d7", "152");
        m.insert("afd7ff", "153");
        m.insert("afff00", "154");
        m.insert("afff5f", "155");
        m.insert("afff87", "156");
        m.insert("afffaf", "157");
        m.insert("afffd7", "158");
        m.insert("afffff", "159");
        m.insert("d70000", "160");
        m.insert("d7005f", "161");
        m.insert("d70087", "162");
        m.insert("d700af", "163");
        m.insert("d700d7", "164");
        m.insert("d700ff", "165");
        m.insert("d75f00", "166");
        m.insert("d75f5f", "167");
        m.insert("d75f87", "168");
        m.insert("d75faf", "169");
        m.insert("d75fd7", "170");
        m.insert("d75fff", "171");
        m.insert("d78700", "172");
        m.insert("d7875f", "173");
        m.insert("d78787", "174");
        m.insert("d787af", "175");
        m.insert("d787d7", "176");
        m.insert("d787ff", "177");
        m.insert("d7af00", "178");
        m.insert("d7af5f", "179");
        m.insert("d7af87", "180");
        m.insert("d7afaf", "181");
        m.insert("d7afd7", "182");
        m.insert("d7afff", "183");
        m.insert("d7d700", "184");
        m.insert("d7d75f", "185");
        m.insert("d7d787", "186");
        m.insert("d7d7af", "187");
        m.insert("d7d7d7", "188");
        m.insert("d7d7ff", "189");
        m.insert("d7ff00", "190");
        m.insert("d7ff5f", "191");
        m.insert("d7ff87", "192");
        m.insert("d7ffaf", "193");
        m.insert("d7ffd7", "194");
        m.insert("d7ffff", "195");
        m.insert("ff0000", "196");
        m.insert("ff005f", "197");
        m.insert("ff0087", "198");
        m.insert("ff00af", "199");
        m.insert("ff00d7", "200");
        m.insert("ff00ff", "201");
        m.insert("ff5f00", "202");
        m.insert("ff5f5f", "203");
        m.insert("ff5f87", "204");
        m.insert("ff5faf", "205");
        m.insert("ff5fd7", "206");
        m.insert("ff5fff", "207");
        m.insert("ff8700", "208");
        m.insert("ff875f", "209");
        m.insert("ff8787", "210");
        m.insert("ff87af", "211");
        m.insert("ff87d7", "212");
        m.insert("ff87ff", "213");
        m.insert("ffaf00", "214");
        m.insert("ffaf5f", "215");
        m.insert("ffaf87", "216");
        m.insert("ffafaf", "217");
        m.insert("ffafd7", "218");
        m.insert("ffafff", "219");
        m.insert("ffd700", "220");
        m.insert("ffd75f", "221");
        m.insert("ffd787", "222");
        m.insert("ffd7af", "223");
        m.insert("ffd7d7", "224");
        m.insert("ffd7ff", "225");
        m.insert("ffff00", "226");
        m.insert("ffff5f", "227");
        m.insert("ffff87", "228");
        m.insert("ffffaf", "229");
        m.insert("ffffd7", "230");
        m.insert("ffffff", "231");

        // Gray-scale range.
        m.insert("080808", "232");
        m.insert("121212", "233");
        m.insert("1c1c1c", "234");
        m.insert("262626", "235");
        m.insert("303030", "236");
        m.insert("3a3a3a", "237");
        m.insert("444444", "238");
        m.insert("4e4e4e", "239");
        m.insert("585858", "240");
        m.insert("626262", "241");
        m.insert("6c6c6c", "242");
        m.insert("767676", "243");
        m.insert("808080", "244");
        m.insert("8a8a8a", "245");
        m.insert("949494", "246");
        m.insert("9e9e9e", "247");
        m.insert("a8a8a8", "248");
        m.insert("b2b2b2", "249");
        m.insert("bcbcbc", "250");
        m.insert("c6c6c6", "251");
        m.insert("d0d0d0", "252");
        m.insert("dadada", "253");
        m.insert("e4e4e4", "254");
        m.insert("eeeeee", "255");
        m
    };

    static ref RE: Regex = Regex::new("(..)(..)(..)").unwrap();
}

pub fn rgb_to_short(rgb: &str) -> usize {
    let incs = vec!(0x00, 0x5f, 0x87, 0xaf, 0xd7, 0xff);

	let matches = RE.captures(rgb).unwrap();
	let parts = vec!(
		i32::from_str_radix(matches.at(1).unwrap(), 16).unwrap(),
		i32::from_str_radix(matches.at(2).unwrap(), 16).unwrap(),
		i32::from_str_radix(matches.at(3).unwrap(), 16).unwrap(),
	);

    let mut result = Vec::new();

    for part in parts {
        let mut i = 0;
        while i < incs.len() - 1 {
            let (s, b) = (incs[i], incs[i+1]);
            if s <= part && part <= b {
                let s1 = (s - part).abs();
                let b1 = (b - part).abs();
                let closest = if s1 < b1 {
                    s
                } else {
                    b
                };
                result.push(closest);
                break;
            }
            i += 1;
        }
    }
    let mut res = String::new();
    for r in result {
        res.push_str(&*format!("{0:02.x}", r));
    }
    let equiv = MAPPINGS.get(&*res);

    return i32::from_str_radix(equiv.unwrap(), 10).unwrap() as usize;
}

pub fn char_width(c: char, is_cjk: bool, tab_width: usize, position: usize) -> Option<usize> {
    use unicode_width::UnicodeWidthChar;

    if c == '\t' {
        Some(tab_width - position%tab_width)
    } else if c == '\n' {
        Some(1)
    } else if is_cjk {
        UnicodeWidthChar::width_cjk(c)
    } else {
        UnicodeWidthChar::width(c)
    }
}
