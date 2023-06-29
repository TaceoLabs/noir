mod const; // Instantiations of GMiMC for prime field of the same order as BN254

use crate::field::modulus_num_bits;

struct GmimcConfig<R> {
    alpha: comptime Field, // S-box power; depends on the underlying field
    arc: [Field; R], // round constants
}

fn config<R> (alpha: Field, arc: [Field; R]) -> GmimcConfig<R> {
    GmimcConfig {
        alpha: alpha,
        arc: arc,
    }
}


fn round<T>(state: &mut [Field; T], key: Field, alpha: Field, arc: Field) {
    let sum = state[0] + key + arc;
    let power = sum.pow_32(alpha);
    for i in 1..T {
        state[i] += power
    }
}

fn feistl_shift<T>(state: &mut [Field; T]){
    let tmp = state[T - 1];
    for i in (1..T).rev() {
        state[i] = state[i - 1];
    }
    state[0] = tmp;
}

// General GMiMC block cipher
fn encrypt<T, R>(gmimc_conf: GmimcConfig<R>, mut x: [Field; T] k: Field) -> [Field; T] {
    assert(modulus_num_bits() >= 128) // otherwise key is too small

    let GmimcConfig {alpha, ark, mds} = gmimc_conf;

    for r in 0..R {
        round(&mut x, k, alpha, ark[r]);
        feistl_shift(&mut x);
    }
    x
}
