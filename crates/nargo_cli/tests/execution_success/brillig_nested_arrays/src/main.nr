struct Header {
    params: [Field; 3],
}

struct MyNote {
    plain: Field,
    array: [Field; 2],
    header: Header,
}

unconstrained fn access_nested(notes: [MyNote; 2], x: Field, y: Field) -> Field {
    notes[x].array[y] + notes[y].array[x] + notes[x].plain + notes[y].header.params[x]
}

unconstrained fn create_inside_brillig(x: Field, y: Field) {
    let header = Header { params: [1, 2, 3]};
    let note0 = MyNote { array: [1, 2], plain : 3, header };
    let note1 = MyNote { array: [4, 5], plain : 6, header };
    assert(access_nested([note0, note1], x, y) == (2 + 4 + 3 + 1));
}

fn main(x: Field, y: Field) {
    let header = Header { params: [1, 2, 3]};
    let note0 = MyNote { array: [1, 2], plain : 3, header };
    let note1 = MyNote { array: [4, 5], plain : 6, header };

    create_inside_brillig(x, y);
    assert(access_nested([note0, note1], x, y) == (2 + 4 + 3 + 1));
}

