variables {
    global:
        0: probe
        1: t
}

rule ("slot acceptance: string-slice-vector") {
    event {
        Ongoing - Global;
    }
    actions {
        Set Global Variable(probe, String Slice(Vector(1, 2, 3), 1, 2));
    }
}
