variables {
    global:
        0: probe
        1: t
}

rule ("slot acceptance: square-root-player") {
    event {
        Ongoing - Global;
    }
    actions {
        Set Global Variable(probe, Square Root(Host Player));
    }
}
