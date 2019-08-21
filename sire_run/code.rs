fn main () {
    // bar(1,2);
    // foo(1,1);
    // distance(0,0);
    // alt_dist(0,0);
    // rec_dist(0,0);
    bound_a(0);
    // bound_b(0);
}

fn bar(n: usize, m: usize) -> usize {
    if n >= m { n } else { m }
}

fn foo(x: i32, y: i32) -> i32 {

    let z = x + 1;
    let w = y - 1;
    if false {
        0
    } else {
        z - w + 4
    }
}

fn distance(x: i32, y: i32) -> i32 {
    if x > y {
        x - y
    } else {
        y - x
    }
}

fn alt_dist(x: i32, y: i32) -> i32 {
    let sign: i32;
    if x > y {
        sign = 1;
    } else {
        sign = -1;
    }
    sign * (x - y)
}

fn rec_dist(x: i32, y: i32) -> i32 {
    if x > y {
        x - y
    } else {
        rec_dist(y, x)
    }
}
//
fn bound_a(x: usize) -> bool {
    x > 0
}
//
// fn bound_b(x: usize) -> bool {
//     x + 1 > 1
// }
