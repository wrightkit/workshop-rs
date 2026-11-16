variables {
    global:
        0: probe
        1: t
}

rule ("slot acceptance: control-string-slice-string") {
    event {
        Ongoing - Global;
    }
    actions {
        Set Global Variable(probe, String Slice(Custom String("abcdef"), 1, 2));
    }
}
