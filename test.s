.start
    ldc %regA 0x8
    ldc %regB 0x0
.blubb
    jrcon 1
.label
    add %regB %regA %regB
    jrcon -2
    jrcon .start
