variables {
    global:
        0: probe
        1: t
}

rule ("slot acceptance: string-slice-number-expression") {
    event {
        Ongoing - Global;
    }
    actions {
        Set Global Variable(probe, String Slice(Add(Modulo(Global.t, 60), 100), 1, 2));
    }
}
