from math import ceil, log, gcd
import sys

from Crypto.Hash import SHAKE128

p = 21888242871839275222246405745257275088548364400416034343698204186575808495617 # BN254
if len(sys.argv) == 2:
   p = int(sys.argv[1])

def get_d(p):
    for d in range(3, p):
        if gcd(d, p-1) == 1:
            break
    return d

def get_gmimc_rounds(t, d, p):
    r1 = 2 * (1 + t + t^2)
    r2 = ceil(2 * log(p, d) + 2 * t)
    if r2 > r1:
        return r2
    return r1

def field_element_from_shake(shake, p):
    bitlen = ceil(log(p, 2))
    byte = ceil(bitlen / 8)

    while True:
        buf = shake.read(int(byte))
        element_candidate = int.from_bytes(buf, "little")
        if element_candidate < p:
            return element_candidate

def get_rc(r, d, p):
    shake = SHAKE128.new()
    p_bytes = ceil(ceil(log(p, 2)) / 8.0)
    init_data = b'GMiMC' + int(d).to_bytes(1, 'little') + int(p).to_bytes(p_bytes, 'little')
    shake.update(init_data)

    round_constants = []
    for i in range(r):
        field_element = field_element_from_shake(shake, p)
        round_constants.append(field_element)

    return round_constants

for t in range(2, 17):
    d = get_d(p)
    r = get_gmimc_rounds(t, d, p)
    rc = get_rc(r, d, p)

    print("fn x{}_{}_config() -> GmimcConfig<{}> ".format(d, t, r), end="{\n")
    print("    config(")
    print("       alpha(),")
    print("       [")
    for r in rc:
        print("           {}, ".format(r))
    print("       ]")
    print("    )")
    print("}")
    print("")
