use dep::std;

fn main(mut x: Field) {
    let one = 1;
    let add1 = |z| {
        *z = *z + one;
    };

    let two = 2;
    let add2 = |z| {
        *z = *z + two;
    };

    add1(&mut x);
    assert(x == 1);

    add2(&mut x);
    assert(x == 3);

}
