// for parsing the lambda calculus interpreter's output.
//

use sdl2::{
    rect::Rect,
};

enum Sign {
    Positive,
    Negative,
    Zero,
}

/// Convert a church boolean to a native boolean.
pub fn parse_church_bool(s: &str) -> Result<bool, String> {
    let (first_var_beg, first_var_end) = get_first_var_pos(&s)?;
    let first_var = &s[first_var_beg..first_var_end];
    let s = &s[first_var_end..];

    let mut sec_var_beg = 0;
    while let Some(' ') = s[sec_var_beg..].chars().next() {
        sec_var_beg += 1;
    }
    if sec_var_beg == 0 {
        if let Some(c) = s.chars().next() {
            return Err(format!("expected space after first variable, found '{}'", c));
        } else {
            return Err("expected space after first variable, but no character was found"
                       .to_string());
        }
    }
    let sec_var_end = get_var_end(s, sec_var_beg)?;

    let second_var = &s[sec_var_beg..sec_var_end];
    let s = &s[sec_var_end..];

    match s.chars().next() {
        Some('.') => {},
        Some(c) => return Err(format!("expected dot after second variable, found {}", c)),
        None => return Err("expected dot after second variable, but no character was found"
                           .to_string()),
    }
    let s = &s[1..]; // skip dot
    let s = s.trim();
    let mut body_end = s.len();
    while let Some(')') = s[..body_end].chars().next_back() {
        body_end -= 1;
    }
    let s = &s[..body_end];

    if s == first_var {
        Ok(true)
    } else if s == second_var {
        Ok(false)
    } else {
        Err(format!("lambda body {} is not equal to either first {} or second {} variable",
                    s, first_var, second_var,
        ))
    }
}

/// Parse a list of rectangles; the list must be made of chained church pairs
/// where the last element is false or nil, and each rectangle must be a
/// 4-tuple containing the integers (x, y, width, height) as SDL2 uses them.
/// Each integer must be encoded in CLNI, our lambda calculus integer encoding.
///
/// All of the rectangles' dimensions will be multiplied by scaling_factor.
/// The x and y coordinates will also be added with their respective offsets
/// afterwards.
///
/// The list is parsed backwards, but that shouldn't make a difference.
///
pub fn parse_rect_list(s: &str,
                       scaling_factor: i32,
                       x_offset: i32,
                       y_offset:i32) -> Result<Vec<Rect>, String> {

    // recursion base
    if is_list_end(s) {
        return Ok(Vec::new());
    }

    match s.chars().next() {
        Some('(') => {},
        _ => return Err("List of rectangles should begin with open paren".to_string()),
    };
    let first_num_beg = match s[1..].chars().position(|c| c == '(') {
        None => return Err("List of rectangles doesn't have a second open paren".to_string()),
        Some(i) => i,
    };

    let s = &s[first_num_beg..];
    let s = s.trim();

    let (rect, s) = parse_rect(s, scaling_factor, x_offset, y_offset)?;
    let s = s.trim();

    let mut rect_list = parse_rect_list(s, scaling_factor, x_offset, y_offset)?;
    rect_list.push(rect);
    return Ok(rect_list);
}

fn parse_rect(s: &str,
              scaling_factor: i32,
              x_offset: i32,
              y_offset: i32) -> Result<(Rect, &str), String> {
    match s.chars().next() {
        Some('(') => {},
        None => return Err("Empty rectangle expression".to_string()),
        Some(c) => return Err(format!("Rectangle expression doesn't start with open paren, but with `{}`", c)),
    };
    let first_num_beg = match s[1..].chars().position(|c| c == '(') {
        None => return Err("Rectangle expression doesn't have a second open paren".to_string()),
        Some(i) => i,
    };
    let s = &s[first_num_beg..];
    let s = s.trim();

    let (x, s) = clni_prefix_to_int(s)?;
    let s = s.trim();

    let (y, s) = clni_prefix_to_int(s)?;
    let s = s.trim();

    let (width, s) = clni_prefix_to_int(s)?;
    let s = s.trim();

    let (height, mut s) = clni_prefix_to_int(s)?;
    if let Some(')') = s.chars().next() {
        s = &s[1..];
    }

    let x = x * scaling_factor + x_offset;
    let y = y * scaling_factor + y_offset;
    let width = (width * scaling_factor) as u32;
    let height = (height * scaling_factor) as u32;

    Ok((Rect::new(x, y, width, height), s))
}

/// For non-negative values only.
pub fn clni_to_int(s: &str) -> Result<i32, String> {
    let (num, _) = clni_prefix_to_int(&s)?;
    Ok(num)
}

// Convert a non-negative number in CLNI at the beginning of the string s,
// returning its value and the slice containing the rest of the string.
//
fn clni_prefix_to_int(s: &str) -> Result<(i32, &str), String> {
    match s.chars().next() {
        Some(c) if c != '(' => {
            return Err(format!("CLNI int at `{}` doesn't begin with open paren, but `{}`", s, c));
        }
        None => return Err("CLNI integer is empty'".to_string()),
        _ => {},
    };

    let sign = clni_prefix_sign(s)?;

    let mut backslash_count:i32 = 0;
    let mut paren_levelcount:i32 = 0;
    let mut chars = s.chars();
    loop {
        match chars.next() {
            Some('\\') => backslash_count += 1,
            Some('(') => paren_levelcount += 1,
            Some(')') => paren_levelcount -= 1,
            None => return Err("unfinished CLNI integer!".to_string()),
            _ => {},
        };
        if paren_levelcount == 0 {
            break;
        }
    };

    let num = match sign {
        Sign::Positive  => backslash_count,
        Sign::Negative  => -(backslash_count - 1),
        Sign::Zero      => 0,
    };
    Ok((num, chars.as_str()))
}

fn clni_prefix_sign(s: &str) -> Result<Sign, String> {
    // Any positive number will be of form
    //      (\x u. u x ...)
    //      (\x1 u. u x1 ...)
    //      etc
    // Whereas zero will be
    //      (\x. x)
    //      (\x1. x1)
    //      etc
    // And negative numbers will be
    //      (\x. x (\u. u) (\u. u) ...)
    // So we count the number of spaces between the backslash and the dot.
    //
    let mut chars = s.chars();
    loop {
        match chars.next() {
            Some('\\') => break,
            None => return Err("CLNI integer has no lambda terms in it!".to_string()),
            _ => {},
        };
    }
    let mut space_count:u32 = 0;
    loop {
        match chars.next() {
            Some(' ') => space_count += 1,
            Some('.') => break,
            None => return Err("CLNI integer has unfinished lambda term in it!".to_string()),
            _ => {},
        };
    };
    let sign = if space_count > 0 {
        Sign::Positive
    } else {
        let is_zero;
        loop {
            match chars.next() {
                None | Some(')') => {
                    is_zero = true;
                    break;
                },
                Some('(') => {
                    is_zero = false;
                    break;
                },
                _ => {},
            };
        };
        if is_zero {
            Sign::Zero
        } else {
            Sign::Negative
        }
    };
    Ok(sign)
}

fn is_list_end(s: &str) -> bool {
    if s.starts_with("nil") || s.starts_with("false") {
        return true;
    }
    if let Ok(b) = parse_church_bool(s) {
        if !b {
            return true;
        }
    }
    false
}

fn get_first_var_pos(s: &str) -> Result<(usize, usize), String> {
    let var_beg = match s.chars().position(|c| c == '\\') {
        None => return Err("no backslash (lambda symbol) found".to_string()),
        Some(i) => i + 1,
    };
    let var_end = get_var_end(s, var_beg)?;
    Ok((var_beg, var_end))
}

// This function assumes var_beg is a valid index and the beginning of the
// variable whose end we seek.
//
fn get_var_end(s: &str, var_beg: usize) -> Result<usize, String> {
    match s[var_beg..].chars().position(|c| !c.is_alphanumeric()) {
        None => return Err("unfinished lambda term".to_string()),
        Some(i) => Ok(i + var_beg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clni_test_pos1() {
        let (num, _) = clni_prefix_to_int("(\\x u. u (\\u1. u1 (\\u2. u2 x)))")
            .unwrap();
        assert_eq!(num, 3);
    }

    #[test]
    fn clni_test_pos2() {
        let (num, _) = clni_prefix_to_int("(\\x u. u x)")
            .unwrap();
        assert_eq!(num, 1);
    }

    #[test]
    fn clni_test_zero() {
        let (num, _) = clni_prefix_to_int("(\\x. x)")
            .unwrap();
        assert_eq!(num, 0);
    }

    #[test]
    fn clni_test_neg1() {
        let (num, _) = clni_prefix_to_int("(\\x. x (\\u. u))")
            .unwrap();
        assert_eq!(num, -1);
    }

    #[test]
    fn clni_test_neg2() {
        let (num, _) = clni_prefix_to_int("(\\x. x (\\u. u) (\\u. u))")
            .unwrap();
        assert_eq!(num, -2);
    }

    #[test]
    fn church_bool_true1() {
        assert_eq!(parse_church_bool("(\\x y. x)"), Ok(true));
    }
    #[test]
    fn church_bool_true2() {
        assert_eq!(parse_church_bool("(\\apple orange. apple)"), Ok(true));
    }
    #[test]
    fn church_bool_true3() {
        assert_eq!(parse_church_bool("(\\x1 y1. x1)"), Ok(true));
    }

    #[test]
    fn church_bool_false1() {
        assert_eq!(parse_church_bool("(\\x y. y)"), Ok(false));
    }
    #[test]
    fn church_bool_false2() {
        assert_eq!(parse_church_bool("(\\blue green. green)"), Ok(false));
    }
    #[test]
    fn church_bool_false3() {
        assert_eq!(parse_church_bool("(\\x2 y2. y2)"), Ok(false));
    }

    #[test]
    fn church_bool_invalid1() {
        if let Ok(_) = parse_church_bool("\\x y z. y") {
            panic!("parse_church_bool should have returned Err");
        }
    }

    #[test]
    fn church_bool_invalid2() {
        if let Ok(_) = parse_church_bool("\\y. y") {
            panic!("parse_church_bool should have returned Err");
        }
    }

    #[test]
    fn church_bool_invalid3() {
        if let Ok(_) = parse_church_bool("\\a b. c") {
            panic!("parse_church_bool should have returned Err");
        }
    }

    #[test]
    fn test_rect1() {
        let s = "(\\f. f (\\x u. u x) (\\x1 u1. u1 (\\u2. u2 (\\u3. u3 x1))) (\\x2 u4. u4 (\\u5. u5 x2)) (\\x3 u6. u6 (\\u7. u7 (\\u8. u8 (\\u9. u9 (\\u10. u10 x3))))))";
        let (rect, _) = parse_rect(s, 1, 0, 0).unwrap();
        assert_eq!(rect, Rect::new(1, 3, 2, 5));
    }

    #[test]
    fn test_rect2() {
        let s = "(\\f. f (\\x. x) (\\x1. x1) (\\x2 u. u (\\u1. u1 (\\u2. u2 (\\u3. u3 x2)))) (\\x3 u4. u4 (\\u5. u5 (\\u6. u6 (\\u7. u7 (\\u8. u8 (\\u9. u9 (\\u10. u10 x3))))))))";
        let (rect, _) = parse_rect(s, 1, 0, 0).unwrap();
        assert_eq!(rect, Rect::new(0, 0, 4, 7));
    }

    #[test]
    fn test_rect_list1() {
        let s = "(\\z. z (\\f. f (\\x. x) (\\x1. x1) (\\x2 u. u (\\u1. u1 (\\u2. u2 (\\u3. u3 x2)))) (\\x3 u4. u4 (\\u5. u5 (\\u6. u6 (\\u7. u7 (\\u8. u8 (\\u9. u9 (\\u10. u10 x3)))))))) (\\z1. z1 (\\f1. f1 (\\x4 u11. u11 (\\u12. u12 (\\u13. u13 (\\u14. u14 (\\u15. u15 (\\u16. u16 (\\u17. u17 (\\u18. u18 x4)))))))) (\\x5 u19. u19 (\\u20. u20 (\\u21. u21 (\\u22. u22 (\\u23. u23 (\\u24. u24 (\\u25. u25 x5))))))) (\\x6 u26. u26 (\\u27. u27 (\\u28. u28 x6))) (\\x7 u29. u29 (\\u30. u30 x7))) nil))";
        let mut expected = Vec::new();
        let rect1 = Rect::new(0, 0, 4, 7);
        let rect2 = Rect::new(8, 7, 3, 2);
        expected.push(rect2);
        expected.push(rect1);
        assert_eq!(parse_rect_list(s, 1, 0, 0), Ok(expected));
    }

    #[test]
    fn test_rect_list2() {
        let s = "(\\z. z (\\f. f (\\x u. u x) (\\x1. x1) (\\x2 u1. u1 (\\u2. u2 x2)) (\\x3 u3. u3 (\\u4. u4 (\\u5. u5 (\\u6. u6 (\\u7. u7 x3)))))) (\\z1. z1 (\\f1. f1 (\\x4 u8. u8 (\\u9. u9 (\\u10. u10 (\\u11. u11 x4)))) (\\x5 u12. u12 (\\u13. u13 (\\u14. u14 x5))) (\\x6 u15. u15 x6) (\\x7 u16. u16 (\\u17. u17 (\\u18. u18 (\\u19. u19 (\\u20. u20 (\\u21. u21 x7))))))) (\\z2. z2 (\\f2. f2 (\\x8. x8) (\\x9 u22. u22 (\\u23. u23 (\\u24. u24 x9))) (\\x10 u25. u25 (\\u26. u26 (\\u27. u27 (\\u28. u28 (\\u29. u29 (\\u30. u30 (\\u31. u31 x10))))))) (\\x11 u32. u32 (\\u33. u33 (\\u34. u34 (\\u35. u35 (\\u36. u36 (\\u37. u37 (\\u38. u38 (\\u39. u39 (\\u40. u40 x11)))))))))) nil)))";

        let rect1 = Rect::new(1, 0, 2, 5);
        let rect2 = Rect::new(4, 3, 1, 6);
        let rect3 = Rect::new(0, 3, 7, 9);

        let mut expected = Vec::new();
        expected.push(rect3);
        expected.push(rect2);
        expected.push(rect1);
        assert_eq!(parse_rect_list(s, 1, 0, 0), Ok(expected));
    }
}
