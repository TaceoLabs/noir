use dep::std::encryption::gmimc;

fn main(k: Field, x1: [Field; 2], y1: pub [Field; 2], x2: [Field; 16], y2: pub [Field; 16])
{
    let cipher2 = gmimc::bn254::enc::x5_2(x1, k);
    assert(cipher2 == y1);

    let cipher16 = gmimc::bn254::enc::x5_16(x2, k);
    assert(cipher16 == y2);
}
