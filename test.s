.start
    ldc %regA 0x42
    tst %regA %regA
    jr .start
    jzr -2
    nop
