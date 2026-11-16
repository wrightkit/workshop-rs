variables {
    global:
        0: probe
        1: t
}

rule ("slot acceptance: string-slice-player") {
    event {
        Ongoing - Global;
    }
    actions {
        Set Global Variable(probe, String Slice(Host Player, 1, 2));
    }
}
